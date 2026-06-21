//! ONNX inference (orientation, line removal, enhancement) via `ort`.
//!
//! NOTE: the model URLs and tensor I/O assumptions below are implemented to the
//! best documented behaviour of these exports but have NOT been validated at
//! runtime in this environment. The first real run is the "validation spike":
//! expect to adjust URLs / normalisation / output scaling per the actual models.
//! Face restoration (GFPGAN/CodeFormer) additionally needs a face-detection
//! stage and is intentionally not wired here yet.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use anyhow::{anyhow, Result};
use image::{imageops::FilterType, GrayImage, Luma, Rgb, RgbImage};
use ort::execution_providers::{CPUExecutionProvider, ExecutionProviderDispatch};
use ort::inputs;
use ort::session::Session;
use ort::value::Tensor;
use tauri::ipc::Channel;

use crate::imaging::{open_rgb, save_rgb};
use crate::models::{ModelEvent, ModelStatus};

/// A downloadable ONNX model. `key` is the stable id the UI/API refer to.
pub struct Model {
    pub key: &'static str,
    pub file: &'static str,
    pub url: &'static str,
    pub label: &'static str,
    pub approx_mb: u32,
}

// Community ONNX exports. Treat URLs as defaults that may need updating after
// the first validation run.
pub const RESNET: Model = Model {
    key: "resnet50",
    file: "resnet50.onnx",
    url: "https://github.com/onnx/models/raw/main/validated/vision/classification/resnet/model/resnet50-v1-7.onnx",
    label: "Smart orient (ResNet-50)",
    approx_mb: 98,
};
pub const LAMA: Model = Model {
    key: "lama",
    file: "lama.onnx",
    url: "https://huggingface.co/Carve/LaMa-ONNX/resolve/main/lama_fp32.onnx",
    label: "Scan-line removal (LaMa)",
    approx_mb: 206,
};
pub const ESRGAN: Model = Model {
    key: "realesrgan",
    file: "realesrgan.onnx",
    url: "https://huggingface.co/wide-video/real-esrgan-v1.0.0/resolve/main/real_esrgan_x4.onnx",
    label: "Enhance (Real-ESRGAN ×4)",
    approx_mb: 66,
};

/// Every model the pipeline can download, in UI display order.
pub const MODELS: &[&Model] = &[&RESNET, &LAMA, &ESRGAN];

static SESSIONS: OnceLock<Mutex<HashMap<&'static str, Session>>> = OnceLock::new();
// Serialises downloads so two callers never race on the same `.part` file.
static DOWNLOADS: OnceLock<Mutex<()>> = OnceLock::new();

/// Execution providers tried in order; ORT falls back to the next if one fails
/// to load. GPU backends are opt-in build features and always fall back to CPU.
fn execution_providers() -> Vec<ExecutionProviderDispatch> {
    #[allow(unused_mut)]
    let mut providers = Vec::new();
    #[cfg(feature = "cuda")]
    providers.push(ort::execution_providers::CUDAExecutionProvider::default().build());
    #[cfg(feature = "directml")]
    providers.push(ort::execution_providers::DirectMLExecutionProvider::default().build());
    providers.push(CPUExecutionProvider::default().build());
    providers
}

/// Stream `model` to disk if absent, reporting `(downloaded, total)` as it goes.
/// Writes to a `.part` file and atomically renames only on a complete download.
fn ensure_model<P: FnMut(u64, Option<u64>)>(
    model: &Model,
    model_dir: &Path,
    mut progress: P,
) -> Result<PathBuf> {
    let path = model_dir.join(model.file);
    if path.exists() {
        return Ok(path);
    }
    let _guard = DOWNLOADS.get_or_init(|| Mutex::new(())).lock().unwrap();
    // Another caller may have finished while we waited for the lock.
    if path.exists() {
        return Ok(path);
    }
    std::fs::create_dir_all(model_dir)?;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .build();
    let resp = agent
        .get(model.url)
        .call()
        .map_err(|e| anyhow!("failed to download model {}: {e}", model.file))?;
    let total = resp
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok());

    let tmp = path.with_extension("part");
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(&tmp)?;
    let mut buf = [0u8; 64 * 1024];
    let mut downloaded = 0u64;
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                return Err(anyhow!("download of {} interrupted: {e}", model.file));
            }
        };
        if let Err(e) = file.write_all(&buf[..n]) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e.into());
        }
        downloaded += n as u64;
        progress(downloaded, total);
    }
    file.flush()?;
    drop(file);

    // A short read against a known length means a truncated download — don't
    // promote it to the final path, or it would poison every later run.
    if let Some(t) = total {
        if downloaded < t {
            let _ = std::fs::remove_file(&tmp);
            return Err(anyhow!(
                "incomplete download for {} ({downloaded}/{t} bytes)",
                model.file
            ));
        }
    }
    std::fs::rename(&tmp, &path)?;
    Ok(path)
}

/// Run `f` with a (lazily created, cached) session for `model`.
fn with_session<F, R>(model: &Model, model_dir: &Path, f: F) -> Result<R>
where
    F: FnOnce(&mut Session) -> Result<R>,
{
    // Download outside the sessions lock so a slow first fetch doesn't block
    // inference on already-loaded models.
    let path = ensure_model(model, model_dir, |_, _| {})?;
    let map = SESSIONS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().unwrap();
    if !guard.contains_key(model.key) {
        let session = Session::builder()?
            .with_execution_providers(execution_providers())?
            .commit_from_file(&path)?;
        guard.insert(model.key, session);
    }
    f(guard.get_mut(model.key).unwrap())
}

/// Snapshot of each model's on-disk state for the settings UI.
pub fn statuses(model_dir: &Path) -> Vec<ModelStatus> {
    MODELS
        .iter()
        .map(|m| {
            let path = model_dir.join(m.file);
            let size_bytes = std::fs::metadata(&path).ok().map(|md| md.len());
            ModelStatus {
                key: m.key.to_string(),
                file: m.file.to_string(),
                label: m.label.to_string(),
                approx_mb: m.approx_mb,
                downloaded: path.exists(),
                size_bytes,
            }
        })
        .collect()
}

/// Pre-fetch models (all, or those whose key is in `keys`), streaming progress
/// over `channel`. Already-present models report `Finished` immediately.
pub fn download_models(model_dir: &Path, keys: Option<&[String]>, channel: &Channel<ModelEvent>) {
    for model in MODELS {
        if let Some(keys) = keys {
            if !keys.iter().any(|k| k == model.key) {
                continue;
            }
        }
        if model_dir.join(model.file).exists() {
            let _ = channel.send(ModelEvent::Finished {
                key: model.key.to_string(),
            });
            continue;
        }
        let _ = channel.send(ModelEvent::Started {
            key: model.key.to_string(),
            label: model.label.to_string(),
        });
        let key = model.key.to_string();
        let result = ensure_model(model, model_dir, |downloaded, total| {
            let _ = channel.send(ModelEvent::Progress {
                key: key.clone(),
                downloaded,
                total,
            });
        });
        match result {
            Ok(_) => {
                let _ = channel.send(ModelEvent::Finished {
                    key: model.key.to_string(),
                });
            }
            Err(e) => {
                let _ = channel.send(ModelEvent::Failed {
                    key: model.key.to_string(),
                    error: e.to_string(),
                });
            }
        }
    }
    let _ = channel.send(ModelEvent::AllDone);
}

fn rotate_k(img: &RgbImage, k: usize) -> RgbImage {
    match k % 4 {
        1 => image::imageops::rotate90(img),
        2 => image::imageops::rotate180(img),
        3 => image::imageops::rotate270(img),
        _ => img.clone(),
    }
}

// ─── Auto-orient (ResNet RotNet heuristic) ────────────────────────────────────

const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

fn preprocess_imagenet(img: &RgbImage) -> Vec<f32> {
    let resized = image::imageops::resize(img, 224, 224, FilterType::Triangle);
    let mut data = vec![0f32; 3 * 224 * 224];
    let hw = 224 * 224;
    for (i, p) in resized.pixels().enumerate() {
        for c in 0..3 {
            let v = p[c] as f32 / 255.0;
            data[c * hw + i] = (v - IMAGENET_MEAN[c]) / IMAGENET_STD[c];
        }
    }
    data
}

fn max_softmax(logits: &[f32]) -> f32 {
    let max = logits.iter().cloned().fold(f32::MIN, f32::max);
    let sum: f32 = logits.iter().map(|&l| (l - max).exp()).sum();
    // softmax of the argmax == exp(0) / sum
    1.0 / sum
}

/// Pick the rotation under which a generic classifier is most confident
/// (assumed upright), and bake it in. Operates in place.
pub fn orient(path: &Path, model_dir: &Path) -> Result<bool> {
    let img = open_rgb(path)?;
    let mut best = (0usize, f32::MIN);
    for k in 0..4 {
        let rotated = rotate_k(&img, k);
        let data = preprocess_imagenet(&rotated);
        let conf = with_session(&RESNET, model_dir, |s| {
            let t = Tensor::from_array((vec![1i64, 3, 224, 224], data))?;
            let out = s.run(inputs![t])?;
            let (_, logits) = out[0].try_extract_tensor::<f32>()?;
            Ok(max_softmax(logits))
        })?;
        if conf > best.1 {
            best = (k, conf);
        }
    }
    if best.0 != 0 {
        save_rgb(&rotate_k(&img, best.0), path)?;
    }
    Ok(true)
}

// ─── Scan-line removal (LaMa inpainting) ──────────────────────────────────────

/// Per-row deviation detector (window 5, sensitivity 2.0), dilated vertically.
fn detect_scan_lines(img: &RgbImage) -> GrayImage {
    let gray = image::imageops::grayscale(img);
    let (w, h) = gray.dimensions();
    let half = 2i32; // window 5
    let mut dev = vec![0f32; h as usize];
    for y in half..(h as i32 - half) {
        let mut acc = 0f64;
        for x in 0..w {
            let mut neigh: Vec<f32> = Vec::with_capacity(4);
            for dy in -half..=half {
                if dy == 0 {
                    continue;
                }
                neigh.push(gray.get_pixel(x, (y + dy) as u32)[0] as f32);
            }
            neigh.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = (neigh[1] + neigh[2]) / 2.0;
            acc += (gray.get_pixel(x, y as u32)[0] as f32 - median).abs() as f64;
        }
        dev[y as usize] = (acc / w as f64) as f32;
    }
    let inner: Vec<f32> = dev[half as usize..(h as usize - half as usize)].to_vec();
    let n = inner.len().max(1) as f32;
    let mean = inner.iter().sum::<f32>() / n;
    let var = inner.iter().map(|d| (d - mean).powi(2)).sum::<f32>() / n;
    let threshold = mean + 2.0 * var.sqrt();

    let mut mask = GrayImage::new(w, h);
    for y in 0..h {
        if dev[y as usize] > threshold {
            for x in 0..w {
                mask.put_pixel(x, y, Luma([255]));
            }
        }
    }
    imageproc::morphology::dilate(&mask, imageproc::distance_transform::Norm::LInf, 1)
}

fn round_down_8(v: u32) -> u32 {
    (v / 8 * 8).max(8)
}

pub fn remove_lines(path: &Path, model_dir: &Path) -> Result<bool> {
    let img = open_rgb(path)?;
    let (w, h) = img.dimensions();
    let mask = detect_scan_lines(&img);
    if mask.pixels().all(|p| p[0] == 0) {
        return Ok(true);
    }

    // LaMa expects dims divisible by 8.
    let (tw, th) = (round_down_8(w), round_down_8(h));
    let img_r = image::imageops::resize(&img, tw, th, FilterType::Triangle);
    let mask_r = image::imageops::resize(&mask, tw, th, FilterType::Nearest);

    let hw = (tw * th) as usize;
    let mut img_data = vec![0f32; 3 * hw];
    for (i, p) in img_r.pixels().enumerate() {
        for c in 0..3 {
            img_data[c * hw + i] = p[c] as f32 / 255.0;
        }
    }
    let mut mask_data = vec![0f32; hw];
    for (i, p) in mask_r.pixels().enumerate() {
        mask_data[i] = if p[0] > 127 { 1.0 } else { 0.0 };
    }

    let out = with_session(&LAMA, model_dir, |s| {
        let image_t = Tensor::from_array((vec![1i64, 3, th as i64, tw as i64], img_data))?;
        let mask_t = Tensor::from_array((vec![1i64, 1, th as i64, tw as i64], mask_data))?;
        let outputs = s.run(inputs![image_t, mask_t])?;
        let (_, data) = outputs[0].try_extract_tensor::<f32>()?;
        Ok(data.to_vec())
    })?;

    // Output assumed (1,3,th,tw) in 0..1 (scaled to 0..255 if it looks normalised).
    let scale = if out.iter().cloned().fold(0f32, f32::max) <= 1.5 { 255.0 } else { 1.0 };
    let mut result = RgbImage::new(tw, th);
    for (i, p) in result.pixels_mut().enumerate() {
        let r = (out[i] * scale).clamp(0.0, 255.0) as u8;
        let g = (out[hw + i] * scale).clamp(0.0, 255.0) as u8;
        let b = (out[2 * hw + i] * scale).clamp(0.0, 255.0) as u8;
        *p = Rgb([r, g, b]);
    }
    let result = image::imageops::resize(&result, w, h, FilterType::Triangle);

    // Composite: take inpainted pixels only where the mask was set.
    let mut composed = img.clone();
    for y in 0..h {
        for x in 0..w {
            if mask.get_pixel(x, y)[0] > 127 {
                composed.put_pixel(x, y, *result.get_pixel(x, y));
            }
        }
    }
    save_rgb(&composed, path)?;
    Ok(true)
}

// ─── Enhancement (Real-ESRGAN x4 upscale) ─────────────────────────────────────

/// Upscale `src` into `dest` via Real-ESRGAN. Face restoration is a follow-up
/// (needs a face-detection + alignment stage).
pub fn enhance(src: &Path, dest: &Path, model_dir: &Path) -> Result<()> {
    let mut img = open_rgb(src)?;
    // Cap input size like the original pipeline to bound memory.
    let (w, h) = img.dimensions();
    let max_dim = 1024u32;
    if w.max(h) > max_dim {
        let scale = max_dim as f32 / w.max(h) as f32;
        img = image::imageops::resize(
            &img,
            (w as f32 * scale) as u32,
            (h as f32 * scale) as u32,
            FilterType::Triangle,
        );
    }
    let (iw, ih) = img.dimensions();
    let hw = (iw * ih) as usize;
    let mut data = vec![0f32; 3 * hw];
    for (i, p) in img.pixels().enumerate() {
        for c in 0..3 {
            data[c * hw + i] = p[c] as f32 / 255.0;
        }
    }

    let (out, ow, oh) = with_session(&ESRGAN, model_dir, |s| {
        let t = Tensor::from_array((vec![1i64, 3, ih as i64, iw as i64], data))?;
        let outputs = s.run(inputs![t])?;
        let (shape, vals) = outputs[0].try_extract_tensor::<f32>()?;
        // shape = [1,3,OH,OW]
        let oh = shape[2] as u32;
        let ow = shape[3] as u32;
        Ok((vals.to_vec(), ow, oh))
    })?;

    let ohw = (ow * oh) as usize;
    let scale = if out.iter().cloned().fold(0f32, f32::max) <= 1.5 { 255.0 } else { 1.0 };
    let mut result = RgbImage::new(ow, oh);
    for (i, p) in result.pixels_mut().enumerate() {
        let r = (out[i] * scale).clamp(0.0, 255.0) as u8;
        let g = (out[ohw + i] * scale).clamp(0.0, 255.0) as u8;
        let b = (out[2 * ohw + i] * scale).clamp(0.0, 255.0) as u8;
        *p = Rgb([r, g, b]);
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    save_rgb(&result, dest)?;
    Ok(())
}

//! ONNX inference (orientation, line removal, enhancement) via `ort`.
//!
//! NOTE: the model URLs and tensor I/O assumptions below are implemented to the
//! best documented behaviour of these exports but have NOT been validated at
//! runtime in this environment. The first real run is the "validation spike":
//! expect to adjust URLs / normalisation / output scaling per the actual models.
//! Face restoration is wired as SCRFD (detection) → square crop → GFPGAN
//! (restore) → feathered composite; landmark alignment is approximated by a
//! centred square crop, which is good enough for scanned snapshots.

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
pub const SCRFD: Model = Model {
    key: "scrfd",
    file: "scrfd.onnx",
    url: "https://huggingface.co/RuteNL/SCRFD-face-detection-ONNX/resolve/main/2.5g_bnkps.onnx",
    label: "Face detect (SCRFD)",
    approx_mb: 4,
};
pub const GFPGAN: Model = Model {
    key: "gfpgan",
    file: "gfpgan.onnx",
    url: "https://huggingface.co/facefusion/models-3.0.0/resolve/main/gfpgan_1.4.onnx",
    label: "Face restore (GFPGAN)",
    approx_mb: 333,
};

/// Every model the pipeline can download, in UI display order.
pub const MODELS: &[&Model] = &[&RESNET, &LAMA, &SCRFD, &GFPGAN, &ESRGAN];

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

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(30)))
        .build()
        .into();
    let resp = agent
        .get(model.url)
        .call()
        .map_err(|e| anyhow!("failed to download model {}: {e}", model.file))?;
    let total = resp
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    let tmp = path.with_extension("part");
    let mut reader = resp.into_body().into_reader();
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
            .with_execution_providers(execution_providers())
            .map_err(|e| anyhow!("{e}"))?
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

// ─── Face restoration (SCRFD detect → GFPGAN restore) ──────────────────────────

/// A detected face box in original-image pixel coordinates.
struct Face {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    score: f32,
}

fn iou(a: &Face, b: &Face) -> f32 {
    let xx1 = a.x1.max(b.x1);
    let yy1 = a.y1.max(b.y1);
    let xx2 = a.x2.min(b.x2);
    let yy2 = a.y2.min(b.y2);
    let inter = (xx2 - xx1).max(0.0) * (yy2 - yy1).max(0.0);
    let area_a = (a.x2 - a.x1).max(0.0) * (a.y2 - a.y1).max(0.0);
    let area_b = (b.x2 - b.x1).max(0.0) * (b.y2 - b.y1).max(0.0);
    let union = area_a + area_b - inter;
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
}

fn nms(mut faces: Vec<Face>, iou_thresh: f32) -> Vec<Face> {
    faces.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let mut keep: Vec<Face> = Vec::new();
    'outer: for f in faces {
        for k in &keep {
            if iou(&f, k) > iou_thresh {
                continue 'outer;
            }
        }
        keep.push(f);
    }
    keep
}

/// Detect faces with SCRFD. Letterboxes the image into 640×640 (top-left
/// aligned), decodes the per-stride score/bbox heads, runs NMS, and maps boxes
/// back to original coordinates. Reads only the score+bbox outputs so it works
/// with both the keypoint (9-output) and plain (6-output) SCRFD exports.
fn detect_faces(img: &RgbImage, model_dir: &Path) -> Result<Vec<Face>> {
    const INPUT: u32 = 640;
    const STRIDES: [usize; 3] = [8, 16, 32];
    const NUM_ANCHORS: usize = 2;
    const SCORE_THRESH: f32 = 0.5;

    let (w, h) = img.dimensions();
    let scale = INPUT as f32 / w.max(h).max(1) as f32;
    let nw = ((w as f32 * scale).round() as u32).clamp(1, INPUT);
    let nh = ((h as f32 * scale).round() as u32).clamp(1, INPUT);
    let resized = image::imageops::resize(img, nw, nh, FilterType::Triangle);
    let mut canvas = RgbImage::from_pixel(INPUT, INPUT, Rgb([0, 0, 0]));
    image::imageops::overlay(&mut canvas, &resized, 0, 0);

    let hw = (INPUT * INPUT) as usize;
    let mut data = vec![0f32; 3 * hw];
    for (i, p) in canvas.pixels().enumerate() {
        for c in 0..3 {
            data[c * hw + i] = (p[c] as f32 - 127.5) / 128.0;
        }
    }

    let mut faces = with_session(&SCRFD, model_dir, |s| {
        let t = Tensor::from_array((vec![1i64, 3, INPUT as i64, INPUT as i64], data))?;
        let out = s.run(inputs![t])?;
        let mut faces: Vec<Face> = Vec::new();
        for (si, &stride) in STRIDES.iter().enumerate() {
            let (_, scores) = out[si].try_extract_tensor::<f32>()?;
            let (_, bboxes) = out[STRIDES.len() + si].try_extract_tensor::<f32>()?;
            let grid = INPUT as usize / stride;
            for i in 0..scores.len() {
                let sc = scores[i];
                if sc < SCORE_THRESH {
                    continue;
                }
                let cell = i / NUM_ANCHORS;
                let cx = (cell % grid) as f32 * stride as f32;
                let cy = (cell / grid) as f32 * stride as f32;
                let l = bboxes[i * 4] * stride as f32;
                let t = bboxes[i * 4 + 1] * stride as f32;
                let r = bboxes[i * 4 + 2] * stride as f32;
                let b = bboxes[i * 4 + 3] * stride as f32;
                faces.push(Face {
                    x1: cx - l,
                    y1: cy - t,
                    x2: cx + r,
                    y2: cy + b,
                    score: sc,
                });
            }
        }
        Ok(faces)
    })?;

    faces = nms(faces, 0.4);
    let inv = 1.0 / scale;
    for f in &mut faces {
        f.x1 *= inv;
        f.y1 *= inv;
        f.x2 *= inv;
        f.y2 *= inv;
    }
    Ok(faces)
}

/// Restore a 512×512 RGB face crop with GFPGAN. Input/output are normalised to
/// [-1, 1] (RGB, NCHW), per the canonical ONNX export.
fn gfpgan_restore(face: &RgbImage, model_dir: &Path) -> Result<RgbImage> {
    let hw = 512 * 512usize;
    let mut data = vec![0f32; 3 * hw];
    for (i, p) in face.pixels().enumerate() {
        for c in 0..3 {
            data[c * hw + i] = (p[c] as f32 / 255.0 - 0.5) / 0.5;
        }
    }
    let out = with_session(&GFPGAN, model_dir, |s| {
        let t = Tensor::from_array((vec![1i64, 3, 512, 512], data))?;
        let outputs = s.run(inputs![t])?;
        let (_, vals) = outputs[0].try_extract_tensor::<f32>()?;
        Ok(vals.to_vec())
    })?;
    let denorm = |v: f32| (((v.clamp(-1.0, 1.0) + 1.0) / 2.0) * 255.0).round() as u8;
    let mut img = RgbImage::new(512, 512);
    for (i, p) in img.pixels_mut().enumerate() {
        *p = Rgb([denorm(out[i]), denorm(out[hw + i]), denorm(out[2 * hw + i])]);
    }
    Ok(img)
}

/// Composite `patch` (size×size) into `dst` at `(ox, oy)`, feathering the edges
/// so restored faces blend into the original photo without a hard seam.
fn blend_in(dst: &mut RgbImage, patch: &RgbImage, ox: u32, oy: u32, size: u32) {
    let feather = (size as f32 * 0.12).max(1.0);
    for j in 0..size {
        for i in 0..size {
            let edge = i.min(size - 1 - i).min(j).min(size - 1 - j) as f32;
            let wgt = (edge / feather).clamp(0.0, 1.0);
            let o = *dst.get_pixel(ox + i, oy + j);
            let q = *patch.get_pixel(i, j);
            let mix = |a: u8, b: u8| (a as f32 * (1.0 - wgt) + b as f32 * wgt).round() as u8;
            dst.put_pixel(
                ox + i,
                oy + j,
                Rgb([mix(o[0], q[0]), mix(o[1], q[1]), mix(o[2], q[2])]),
            );
        }
    }
}

/// Detect faces and restore each one in place. A no-op (Ok) when no faces are
/// found, so it is safe to include in any preset.
pub fn restore_faces(path: &Path, model_dir: &Path) -> Result<bool> {
    let mut img = open_rgb(path)?;
    let (w, h) = img.dimensions();
    // Too small to crop a meaningful face from — nothing to do.
    if w < 16 || h < 16 {
        return Ok(true);
    }
    let faces = detect_faces(&img, model_dir)?;
    if faces.is_empty() {
        return Ok(true);
    }
    let max_side = w.min(h) as i32;
    for f in faces {
        let cx = (f.x1 + f.x2) / 2.0;
        let cy = (f.y1 + f.y2) / 2.0;
        // Square crop around the face with a 40% margin for context.
        let side = ((f.x2 - f.x1).max(f.y2 - f.y1) * 1.4).round() as i32;
        let size = side.clamp(16, max_side);
        if size < 16 {
            continue;
        }
        let sx = ((cx - size as f32 / 2.0).round() as i32).clamp(0, w as i32 - size) as u32;
        let sy = ((cy - size as f32 / 2.0).round() as i32).clamp(0, h as i32 - size) as u32;
        let size = size as u32;

        let crop = image::imageops::crop_imm(&img, sx, sy, size, size).to_image();
        let face512 = image::imageops::resize(&crop, 512, 512, FilterType::Triangle);
        let restored512 = match gfpgan_restore(&face512, model_dir) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("gfpgan restore failed: {e}");
                continue;
            }
        };
        let restored = image::imageops::resize(&restored512, size, size, FilterType::Triangle);
        blend_in(&mut img, &restored, sx, sy, size);
    }
    save_rgb(&img, path)?;
    Ok(true)
}

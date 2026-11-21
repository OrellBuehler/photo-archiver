export interface Image {
  id: number
  source_path: string
  filename: string
  scan_id: string | null
  file_size: number | null
  width: number | null
  height: number | null
  year: number | null
  month: number | null
  title: string | null
  status: string
  organized_path: string | null
  enhanced_path: string | null
  thumbnail_path: string | null
  phash: string | null
  folder: string | null
  history_pos: number
  error: string | null
  created_at: string
  updated_at: string
}

export interface ImageFilters {
  year?: number | null
  month?: number | null
  status?: string | null
  step?: string | null
  folder?: string | null
  search?: string | null
  year_unknown: boolean
}

export interface TimelineStep {
  seq: number
  label: string
  created_at: string
}

/// A photo's undo stack; steps with `seq > pos` are currently undone.
export interface ImageTimeline {
  pos: number
  steps: TimelineStep[]
}

export interface ImageListResponse {
  images: Image[]
  total: number
  page: number
  per_page: number
}

export interface FilterCountItem {
  value: string
  count: number
}

export interface FilterCounts {
  years: FilterCountItem[]
  months: FilterCountItem[]
  statuses: FilterCountItem[]
  steps: FilterCountItem[]
  folders: FilterCountItem[]
  total: number
}

export interface AppSettings {
  source_dir: string | null
  output_dir: string | null
  thumbnail_size: number
}

export interface Task {
  id: number
  status: string
  steps: string[]
  total_images: number
  completed_images: number
  failed_images: number
  error_message: string | null
  created_at: string
  started_at: string | null
  completed_at: string | null
}

export interface DuplicateGroup {
  /// Best quality first.
  images: Image[]
  distance: number
}

export type ProgressEvent =
  | { type: 'task_started'; task_id: number; total: number }
  | { type: 'image_started'; task_id: number; image_id: number }
  | { type: 'step_started'; task_id: number; image_id: number; step: string }
  | { type: 'step_completed'; task_id: number; image_id: number; step: string }
  | { type: 'progress'; task_id: number; completed: number; failed: number; total: number }
  | { type: 'image_failed'; task_id: number; image_id: number; error: string }
  | { type: 'task_completed'; task_id: number; status: string }

export interface ModelStatus {
  key: string
  file: string
  label: string
  approx_mb: number
  downloaded: boolean
  size_bytes: number | null
}

export type ModelEvent =
  | { type: 'started'; key: string; label: string }
  | { type: 'progress'; key: string; downloaded: number; total: number | null }
  | { type: 'finished'; key: string }
  | { type: 'failed'; key: string; error: string }
  | { type: 'all_done' }

export interface ModelDownload {
  downloaded: number
  total: number | null
  done: boolean
  error: string | null
}

export interface PipelineStep {
  key: string
  label: string
  hint: string
  icon: string
  model?: boolean
  modelKey?: string
  modelKeys?: string[]
}

// Every photo is copied into the archive first ("organize", always on);
// these are the optional clean-up steps on that copy, in run order.
export const PIPELINE_STEPS: PipelineStep[] = [
  { key: 'orient', label: 'Auto-rotate', hint: 'Apply the rotation the scanner recorded', icon: 'rotateSquare' },
  { key: 'crop', label: 'Crop borders', hint: 'Trim blank scan margins', icon: 'crop' },
  { key: 'auto_orient', label: 'Smart orient', hint: 'Detect which way is up', icon: 'image', model: true, modelKey: 'resnet50' },
  { key: 'deskew', label: 'Straighten', hint: 'Level tilted scans', icon: 'scanLine' },
  { key: 'restore_color', label: 'Restore colour', hint: 'Fix fading and contrast', icon: 'palette' },
  { key: 'remove_dust', label: 'Remove dust', hint: 'Paint out specks and scratches', icon: 'sparkles' },
  { key: 'remove_lines', label: 'Remove scan lines', hint: 'Repair scanner streaks', icon: 'layers', model: true, modelKey: 'lama' },
  { key: 'restore_faces', label: 'Restore faces', hint: 'Sharpen blurry faces', icon: 'user', model: true, modelKeys: ['scrfd', 'gfpgan'] },
  { key: 'enhance', label: 'Upscale 4×', hint: 'Slow — adds detail for printing', icon: 'wand', model: true, modelKey: 'realesrgan' },
]

const STEP_LABELS: Record<string, string> = {
  organize: 'Filed',
  rotate_left: 'Rotated left',
  rotate_right: 'Rotated right',
}

export function stepLabel(key: string): string {
  return STEP_LABELS[key] ?? PIPELINE_STEPS.find((s) => s.key === key)?.label ?? key
}

// The status language: where a photo is on its way into the archive.
export const STATUS: Record<string, { label: string; color: string; hint: string }> = {
  new: { label: 'New', color: 'var(--status-new)', hint: 'Not in the archive yet' },
  organized: { label: 'Filed', color: 'var(--status-organized)', hint: 'Copied into the archive as scanned' },
  restored: { label: 'Restored', color: 'var(--status-restored)', hint: 'Cleaned up and in the archive' },
  failed: { label: 'Failed', color: 'var(--status-failed)', hint: 'The last restore did not finish' },
}

/// The model keys a step needs downloaded before it can run.
export function stepModelKeys(s: PipelineStep): string[] {
  return s.modelKeys ?? (s.modelKey ? [s.modelKey] : [])
}

export interface Preset {
  key: string
  label: string
  steps: string[]
  desc: string
}

// Curated bundles — a friendlier entry than the flat step checklist.
export const PRESETS: Preset[] = [
  { key: 'file', label: 'Just file', steps: [], desc: 'Copy into folders, untouched' },
  { key: 'tidy', label: 'Tidy', steps: ['orient', 'crop', 'deskew'], desc: 'Crop and straighten' },
  { key: 'restore', label: 'Restore', steps: ['crop', 'deskew', 'restore_color', 'remove_dust'], desc: 'Tidy, colour and dust' },
  { key: 'max', label: 'Everything', steps: PIPELINE_STEPS.map((s) => s.key), desc: 'All steps incl. AI — slow' },
]

export const MONTHS = [
  '',
  'Januar',
  'Februar',
  'März',
  'April',
  'Mai',
  'Juni',
  'Juli',
  'August',
  'September',
  'Oktober',
  'November',
  'Dezember',
]

/// "06.1978", "1978" or null when undated.
export function dateLabel(img: { year: number | null; month: number | null }): string | null {
  if (img.year == null) return null
  return img.month ? `${String(img.month).padStart(2, '0')}.${img.year}` : String(img.year)
}

export function plural(n: number, one: string, many = `${one}s`): string {
  return `${n} ${n === 1 ? one : many}`
}

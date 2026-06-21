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
  created_at: string
  updated_at: string
}

export interface ImageFilters {
  year?: number | null
  month?: number | null
  status?: string | null
  step?: string | null
  year_unknown: boolean
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

export interface HistoryItem {
  id: number
  image_id: number
  step: string
  created_at: string
}

export interface DuplicateGroup {
  image_ids: number[]
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
}

export const PIPELINE_STEPS: PipelineStep[] = [
  { key: 'organize', label: 'Organize', hint: 'Copy into year/month folders', icon: 'folderTree' },
  { key: 'orient', label: 'Auto-rotate', hint: 'Apply EXIF orientation', icon: 'rotateSquare' },
  { key: 'crop', label: 'Crop borders', hint: 'Trim blank scan margins', icon: 'crop' },
  { key: 'auto_orient', label: 'Smart orient', hint: 'ML upright detection', icon: 'image', model: true, modelKey: 'resnet50' },
  { key: 'deskew', label: 'Deskew', hint: 'Straighten tilted scans', icon: 'scanLine' },
  { key: 'restore_color', label: 'Restore color', hint: 'Fix fading & contrast', icon: 'palette' },
  { key: 'remove_dust', label: 'Remove dust', hint: 'Inpaint specks & scratches', icon: 'sparkles' },
  { key: 'remove_lines', label: 'Remove scan lines', hint: 'LaMa inpaint', icon: 'layers', model: true, modelKey: 'lama' },
  { key: 'enhance', label: 'Enhance', hint: 'Real-ESRGAN upscale', icon: 'wand', model: true, modelKey: 'realesrgan' },
]

export interface Preset {
  key: string
  label: string
  steps: string[]
  desc: string
}

// Curated bundles — a friendlier entry than the flat step checklist.
export const PRESETS: Preset[] = [
  { key: 'file', label: 'Just file it', steps: ['organize'], desc: 'Sort into folders, no edits' },
  { key: 'tidy', label: 'Quick tidy', steps: ['organize', 'orient', 'crop', 'deskew'], desc: 'Straighten, crop, organize' },
  {
    key: 'restore',
    label: 'Full restore',
    steps: ['organize', 'crop', 'deskew', 'restore_color', 'remove_dust'],
    desc: 'Recommended for faded photos',
  },
  { key: 'max', label: 'Everything', steps: PIPELINE_STEPS.map((s) => s.key), desc: 'All steps incl. ML upscale' },
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

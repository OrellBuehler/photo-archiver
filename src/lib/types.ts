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

export type ProgressEvent =
  | { type: 'task_started'; task_id: number; total: number }
  | { type: 'image_started'; task_id: number; image_id: number }
  | { type: 'step_started'; task_id: number; image_id: number; step: string }
  | { type: 'step_completed'; task_id: number; image_id: number; step: string }
  | { type: 'progress'; task_id: number; completed: number; failed: number; total: number }
  | { type: 'image_failed'; task_id: number; image_id: number; error: string }
  | { type: 'task_completed'; task_id: number; status: string }

export interface PipelineStep {
  key: string
  label: string
  hint: string
}

export const PIPELINE_STEPS: PipelineStep[] = [
  { key: 'organize', label: 'Organize', hint: 'Copy into year/month folders' },
  { key: 'orient', label: 'Auto-rotate', hint: 'Apply EXIF orientation' },
  { key: 'crop', label: 'Crop borders', hint: 'Trim white scan borders' },
  { key: 'deskew', label: 'Deskew', hint: 'Straighten tilted scans' },
  { key: 'restore_color', label: 'Restore color', hint: 'Fix fading & contrast' },
  { key: 'remove_dust', label: 'Remove dust', hint: 'Inpaint specks & scratches' },
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

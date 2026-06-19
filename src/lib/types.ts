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

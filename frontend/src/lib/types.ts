export interface Image {
  id: number;
  source_path: string;
  filename: string;
  scan_id: string | null;
  file_size: number | null;
  width: number | null;
  height: number | null;
  year: number | null;
  month: number | null;
  title: string | null;
  status: string;
  organized_path: string | null;
  enhanced_path: string | null;
  thumbnail_path: string | null;
  created_at: string | null;
  updated_at: string | null;
}

export interface ImageListResponse {
  images: Image[];
  total: number;
  page: number;
  per_page: number;
}

export interface ImageStats {
  year: number | null;
  status: string;
  count: number;
}

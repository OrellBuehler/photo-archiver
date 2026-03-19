import type { ImageListResponse, ImageStats, Image } from './types';

const BASE = '/api';

async function fetchJSON<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, init);
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}

export async function getImages(params: {
  year?: number | null;
  month?: number | null;
  status?: string | null;
  page?: number;
  per_page?: number;
} = {}): Promise<ImageListResponse> {
  const sp = new URLSearchParams();
  if (params.year != null) sp.set('year', String(params.year));
  if (params.month != null) sp.set('month', String(params.month));
  if (params.status) sp.set('status', params.status);
  if (params.page) sp.set('page', String(params.page));
  if (params.per_page) sp.set('per_page', String(params.per_page));
  return fetchJSON<ImageListResponse>(`${BASE}/images?${sp}`);
}

export async function getImageStats(): Promise<ImageStats[]> {
  return fetchJSON<ImageStats[]>(`${BASE}/images/stats`);
}

export async function getImage(id: number): Promise<Image> {
  return fetchJSON<Image>(`${BASE}/images/${id}`);
}

export async function updateImage(id: number, data: { year?: number; month?: number; title?: string }): Promise<Image> {
  return fetchJSON<Image>(`${BASE}/images/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export function thumbnailUrl(id: number): string {
  return `${BASE}/images/${id}/thumbnail`;
}

export function imageFileUrl(id: number, variant: string = 'source'): string {
  return `${BASE}/images/${id}/file?variant=${variant}`;
}

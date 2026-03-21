import type { ImageListResponse, ImageStats, Image, Task, AppSettings, DuplicateGroup } from './types';

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

export async function rotateImage(id: number, direction: 'left' | 'right'): Promise<Image> {
  return fetchJSON<Image>(`${BASE}/images/${id}/rotate?direction=${direction}`, { method: 'POST' });
}

export async function createBatchTask(imageIds: number[] | 'all', steps: string[]): Promise<Task> {
  return fetchJSON<Task>(`${BASE}/processing/batch`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ image_ids: imageIds, steps }),
  });
}

export async function cancelTask(taskId: number): Promise<void> {
  await fetch(`${BASE}/processing/${taskId}`, { method: 'DELETE' });
}

export async function getTasks(): Promise<Task[]> {
  return fetchJSON<Task[]>(`${BASE}/tasks`);
}

export async function getTask(taskId: number): Promise<Task> {
  return fetchJSON<Task>(`${BASE}/tasks/${taskId}`);
}

export async function bulkDeleteImages(imageIds: number[]): Promise<{deleted: number}> {
  return fetchJSON(`${BASE}/images/bulk-delete`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ image_ids: imageIds }),
  });
}

export async function scanDuplicates(): Promise<{status: string}> {
  return fetchJSON(`${BASE}/duplicates/scan`, { method: 'POST' });
}

export async function getDuplicates(threshold: number = 6): Promise<DuplicateGroup[]> {
  return fetchJSON<DuplicateGroup[]>(`${BASE}/duplicates?threshold=${threshold}`);
}

export async function getSettings(): Promise<AppSettings> {
  return fetchJSON<AppSettings>(`${BASE}/settings`);
}

export async function updateSettings(data: { thumbnail_size?: number; device?: string }): Promise<AppSettings> {
  return fetchJSON<AppSettings>(`${BASE}/settings`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function bulkUpdateImages(imageIds: number[], data: { year?: number; month?: number; title?: string }): Promise<{updated: number}> {
  return fetchJSON(`${BASE}/images/bulk`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ image_ids: imageIds, ...data }),
  });
}

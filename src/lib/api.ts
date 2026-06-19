import { Channel, invoke } from '@tauri-apps/api/core'
import type {
  AppSettings,
  DuplicateGroup,
  FilterCounts,
  HistoryItem,
  Image,
  ImageFilters,
  ImageListResponse,
  ProgressEvent,
  Task,
} from './types'

export const getSettings = () => invoke<AppSettings>('get_settings')

export const pickSourceFolder = () =>
  invoke<AppSettings | null>('pick_source_folder')

export const scanSource = () => invoke<number>('scan_source')

export const listImages = (filters: ImageFilters, page: number, perPage: number) =>
  invoke<ImageListResponse>('list_images', { filters, page, perPage })

export const imageStats = (filters: ImageFilters) =>
  invoke<FilterCounts>('image_stats', { filters })

export const getImage = (id: number) => invoke<Image | null>('get_image', { id })

export function startBatch(
  imageIds: number[],
  all: boolean,
  steps: string[],
  onEvent: Channel<ProgressEvent>,
) {
  return invoke<Task>('start_batch', { imageIds, all, steps, onEvent })
}

export const cancelTask = (taskId: number) => invoke<boolean>('cancel_task', { taskId })
export const listTasks = () => invoke<Task[]>('list_tasks')
export const getTask = (id: number) => invoke<Task | null>('get_task', { id })
export const imageHistory = (id: number) => invoke<HistoryItem[]>('image_history', { id })

export const rotateImage = (id: number, clockwise: boolean) =>
  invoke<Image>('rotate_image', { id, clockwise })

export const updateImage = (
  id: number,
  year: number | null,
  month: number | null,
  title: string | null,
) => invoke<Image>('update_image', { id, year, month, title })

export const bulkUpdate = (
  ids: number[],
  year: number | null,
  month: number | null,
  title: string | null,
) => invoke<number>('bulk_update', { ids, year, month, title })

export const bulkDelete = (ids: number[]) => invoke<number>('bulk_delete', { ids })

export async function getVariant(id: number, variant: string): Promise<string> {
  const buf = await invoke<ArrayBuffer>('get_variant', { id, variant })
  return URL.createObjectURL(new Blob([buf], { type: 'image/jpeg' }))
}

export const scanDuplicates = () => invoke<number>('scan_duplicates')
export const findDuplicates = (threshold: number) =>
  invoke<DuplicateGroup[]>('find_duplicates', { threshold })
export const pickOutputFolder = () => invoke<AppSettings | null>('pick_output_folder')
export const updateSettings = (thumbnailSize: number) =>
  invoke<AppSettings>('update_settings', { thumbnailSize })

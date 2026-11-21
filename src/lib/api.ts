import { Channel, invoke } from '@tauri-apps/api/core'
import type {
  AppSettings,
  DuplicateGroup,
  FilterCounts,
  Image,
  ImageFilters,
  ImageListResponse,
  ImageTimeline,
  ModelEvent,
  ModelStatus,
  ProgressEvent,
  Task,
} from './types'

export const getSettings = () => invoke<AppSettings>('get_settings')

export const pickSourceFolder = () =>
  invoke<AppSettings | null>('pick_source_folder')

export const setSourceFolder = (path: string) =>
  invoke<AppSettings | null>('set_source_folder', { path })

export const scanSource = () => invoke<number>('scan_source')

export const listImages = (filters: ImageFilters, page: number, perPage: number) =>
  invoke<ImageListResponse>('list_images', { filters, page, perPage })

export const imageStats = (filters: ImageFilters) =>
  invoke<FilterCounts>('image_stats', { filters })

export const listImageIds = (filters: ImageFilters) => invoke<number[]>('list_image_ids', { filters })

export function startBatch(imageIds: number[], steps: string[], force: boolean, onEvent: Channel<ProgressEvent>) {
  return invoke<Task>('start_batch', { imageIds, steps, force, onEvent })
}

export const cancelTask = (taskId: number) => invoke<boolean>('cancel_task', { taskId })
export const listTasks = () => invoke<Task[]>('list_tasks')
export const imageTimeline = (id: number) => invoke<ImageTimeline>('image_timeline', { id })

export const bulkRotate = (ids: number[], clockwise: boolean) =>
  invoke<number>('bulk_rotate', { ids, clockwise })

export const setFolder = (ids: number[], folder: string | null) =>
  invoke<number>('set_folder', { ids, folder })

export const undoImage = (id: number) => invoke<Image>('undo_image', { id })
export const redoImage = (id: number) => invoke<Image>('redo_image', { id })
export const resetImages = (ids: number[]) => invoke<number>('reset_images', { ids })

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

export const listModels = () => invoke<ModelStatus[]>('list_models')
export const modelsDir = () => invoke<string>('models_dir')
export const logDir = () => invoke<string>('log_dir')
export const openLogDir = () => invoke<void>('open_log_dir')
export const openOutputDir = () => invoke<void>('open_output_dir')
export const revealImage = (id: number) => invoke<void>('reveal_image', { id })
export function downloadModels(keys: string[] | null, onEvent: Channel<ModelEvent>) {
  return invoke<void>('download_models', { keys, onEvent })
}

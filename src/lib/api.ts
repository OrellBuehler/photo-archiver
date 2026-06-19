import { invoke } from '@tauri-apps/api/core'
import type {
  AppSettings,
  FilterCounts,
  Image,
  ImageFilters,
  ImageListResponse,
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

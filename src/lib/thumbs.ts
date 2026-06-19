import { invoke } from '@tauri-apps/api/core'

const cache = new Map<number, string>()

export async function thumbUrl(id: number): Promise<string> {
  const hit = cache.get(id)
  if (hit) return hit
  const buf = await invoke<ArrayBuffer>('get_thumbnail', { id })
  const url = URL.createObjectURL(new Blob([buf], { type: 'image/jpeg' }))
  cache.set(id, url)
  return url
}

export function invalidateThumb(id: number) {
  const url = cache.get(id)
  if (url) {
    URL.revokeObjectURL(url)
    cache.delete(id)
  }
}

export function clearThumbCache() {
  for (const url of cache.values()) URL.revokeObjectURL(url)
  cache.clear()
}

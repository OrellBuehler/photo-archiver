import { Channel } from '@tauri-apps/api/core'
import type { DockviewApi } from 'dockview-core'
import { openPanel } from './workspace'
import {
  bulkDelete,
  bulkUpdate,
  cancelTask,
  downloadModels,
  getSettings,
  imageStats,
  listImages,
  listTasks,
  pickOutputFolder,
  pickSourceFolder,
  scanSource,
  startBatch,
  updateSettings,
} from './api'
import { clearThumbCache } from './thumbs'
import type {
  AppSettings,
  FilterCounts,
  Image,
  ImageFilters,
  ModelDownload,
  ModelEvent,
  ProgressEvent,
  Task,
} from './types'

const emptyCounts: FilterCounts = {
  years: [],
  months: [],
  statuses: [],
  steps: [],
  total: 0,
}

interface ActiveTask {
  id: number
  total: number
  completed: number
  failed: number
  status: string
  currentImage: number | null
  currentStep: string | null
}

class AppStore {
  settings = $state<AppSettings | null>(null)
  filters = $state<ImageFilters>({ year_unknown: false })
  images = $state<Image[]>([])
  total = $state(0)
  page = $state(1)
  readonly perPage = 60
  counts = $state<FilterCounts>(emptyCounts)
  selected = $state<Set<number>>(new Set())
  loading = $state(false)
  scanning = $state(false)
  lastScanCount = $state<number | null>(null)
  error = $state<string | null>(null)

  tasks = $state<Task[]>([])
  activeTask = $state<ActiveTask | null>(null)
  processingIds = $state<Set<number>>(new Set())
  thumbVersion = $state(0)
  focusedImageId = $state<number | null>(null)

  modelProgress = $state<Record<string, ModelDownload>>({})
  downloadingModels = $state(false)

  // Not reactive — a handle to the dockview workspace for activating panels.
  dockApi: DockviewApi | null = null

  get pages() {
    return Math.max(1, Math.ceil(this.total / this.perPage))
  }

  get busy() {
    return this.activeTask?.status === 'running'
  }

  async init() {
    this.settings = await getSettings()
    await this.loadTasks()
    if (this.settings?.source_dir) await this.refresh()
  }

  async pickFolder() {
    this.error = null
    try {
      const next = await pickSourceFolder()
      if (next) {
        this.settings = next
        await this.scan()
      }
    } catch (e) {
      this.error = String(e)
    }
  }

  async scan() {
    if (!this.settings?.source_dir) return
    this.scanning = true
    this.error = null
    try {
      this.lastScanCount = await scanSource()
      this.page = 1
      await this.refresh()
    } catch (e) {
      this.error = String(e)
    } finally {
      this.scanning = false
    }
  }

  async refresh() {
    this.loading = true
    try {
      const [list, counts] = await Promise.all([
        listImages(this.filters, this.page, this.perPage),
        imageStats(this.filters),
      ])
      this.images = list.images
      this.total = list.total
      this.counts = counts
    } finally {
      this.loading = false
    }
  }

  setFilter(patch: Partial<ImageFilters>) {
    this.filters = { ...this.filters, ...patch }
    this.page = 1
    this.refresh()
  }

  clearFilters() {
    this.filters = { year_unknown: false }
    this.page = 1
    this.refresh()
  }

  goToPage(p: number) {
    this.page = Math.min(Math.max(1, p), this.pages)
    this.refresh()
  }

  toggleSelect(id: number) {
    const next = new Set(this.selected)
    if (next.has(id)) next.delete(id)
    else next.add(id)
    this.selected = next
  }

  selectAllOnPage() {
    const next = new Set(this.selected)
    for (const img of this.images) next.add(img.id)
    this.selected = next
  }

  clearSelection() {
    this.selected = new Set()
  }

  async loadTasks() {
    this.tasks = await listTasks()
  }

  private onProgress(e: ProgressEvent) {
    switch (e.type) {
      case 'task_started':
        this.activeTask = {
          id: e.task_id,
          total: e.total,
          completed: 0,
          failed: 0,
          status: 'running',
          currentImage: null,
          currentStep: null,
        }
        break
      case 'image_started':
        this.processingIds = new Set(this.processingIds).add(e.image_id)
        if (this.activeTask) this.activeTask.currentImage = e.image_id
        break
      case 'step_started':
        if (this.activeTask) this.activeTask.currentStep = e.step
        break
      case 'progress':
        if (this.activeTask) {
          this.activeTask.completed = e.completed
          this.activeTask.failed = e.failed
        }
        break
      case 'task_completed':
        if (this.activeTask) this.activeTask.status = e.status
        break
    }
  }

  async startBatch(steps: string[], all: boolean) {
    if (this.busy || steps.length === 0) return
    const imageIds = all ? [] : [...this.selected]
    if (!all && imageIds.length === 0) return

    const channel = new Channel<ProgressEvent>()
    channel.onmessage = (e) => this.onProgress(e)
    try {
      await startBatch(imageIds, all, steps, channel)
    } finally {
      clearThumbCache()
      this.thumbVersion++
      this.processingIds = new Set()
      this.activeTask = null
      await Promise.all([this.loadTasks(), this.refresh()])
    }
  }

  async cancelActive() {
    if (this.activeTask) await cancelTask(this.activeTask.id)
  }

  openImage(id: number) {
    this.focusedImageId = id
    this.openPanel('viewer')
  }

  // Focus a panel (StageBar, titlebar menu), re-creating it if it was closed.
  openPanel(id: string) {
    if (this.dockApi) openPanel(this.dockApi, id)
  }

  statusCount(value: string) {
    return this.counts.statuses.find((s) => s.value === value)?.count ?? 0
  }

  async deleteSelected() {
    if (this.selected.size === 0) return
    const n = await bulkDelete([...this.selected])
    this.clearSelection()
    await this.refresh()
    return n
  }

  async editSelected(year: number | null, month: number | null, title: string | null) {
    if (this.selected.size === 0) return
    await bulkUpdate([...this.selected], year, month, title)
    await this.refresh()
  }

  async pickOutput() {
    const next = await pickOutputFolder()
    if (next) this.settings = next
  }

  /// Download the given model keys (or all missing when null), streaming
  /// progress into `modelProgress`. Returns true if none failed.
  async downloadModelFiles(keys: string[] | null): Promise<boolean> {
    if (this.downloadingModels) return false
    this.downloadingModels = true
    this.modelProgress = {}
    let ok = true
    const prev = (key: string): ModelDownload =>
      this.modelProgress[key] ?? { downloaded: 0, total: null, done: false, error: null }

    const channel = new Channel<ModelEvent>()
    channel.onmessage = (e: ModelEvent) => {
      switch (e.type) {
        case 'started':
          this.modelProgress = { ...this.modelProgress, [e.key]: { downloaded: 0, total: null, done: false, error: null } }
          break
        case 'progress':
          this.modelProgress = {
            ...this.modelProgress,
            [e.key]: { downloaded: e.downloaded, total: e.total, done: false, error: null },
          }
          break
        case 'finished':
          this.modelProgress = { ...this.modelProgress, [e.key]: { ...prev(e.key), done: true, error: null } }
          break
        case 'failed':
          ok = false
          this.modelProgress = { ...this.modelProgress, [e.key]: { ...prev(e.key), done: false, error: e.error } }
          break
      }
    }
    try {
      await downloadModels(keys, channel)
    } catch (e) {
      ok = false
      this.error = String(e)
    } finally {
      this.downloadingModels = false
    }
    return ok
  }

  async setThumbnailSize(size: number) {
    this.settings = await updateSettings(size)
    clearThumbCache()
    this.thumbVersion++
  }
}

export const store = new AppStore()

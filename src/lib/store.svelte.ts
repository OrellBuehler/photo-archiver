import { Channel } from '@tauri-apps/api/core'
import {
  bulkDelete,
  bulkRotate,
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
  setFolder,
  setSourceFolder,
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
  folders: [],
  total: 0,
}

const PAGE = 120

export type View = 'grid' | 'loupe' | 'duplicates'
export type InspectorTab = 'info' | 'restore'

interface ActiveTask {
  id: number
  total: number
  completed: number
  failed: number
  status: string
  currentImage: number | null
  currentStep: string | null
  startedAt: number
}

export interface Toast {
  id: number
  kind: 'info' | 'success' | 'error'
  text: string
}

export interface ConfirmRequest {
  title: string
  message: string
  confirmLabel: string
  danger: boolean
  resolve: (ok: boolean) => void
}

function loadPref<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(`photo-archiver:${key}`)
    return raw == null ? fallback : (JSON.parse(raw) as T)
  } catch {
    return fallback
  }
}

function savePref(key: string, value: unknown) {
  try {
    localStorage.setItem(`photo-archiver:${key}`, JSON.stringify(value))
  } catch {}
}

class AppStore {
  settings = $state<AppSettings | null>(null)
  ready = $state(false)
  filters = $state<ImageFilters>({ year_unknown: false })
  images = $state<Image[]>([])
  total = $state(0)
  private loadedPages = 1
  counts = $state<FilterCounts>(emptyCounts)
  loading = $state(false)
  loadingMore = $state(false)
  scanning = $state(false)

  view = $state<View>('grid')
  selected = $state<Set<number>>(new Set())
  focusedId = $state<number | null>(null)
  private anchorId: number | null = null

  sidebarOpen = $state(loadPref('sidebar', true))
  inspectorOpen = $state(loadPref('inspector', true))
  inspectorTab = $state<InspectorTab>('info')
  gridSize = $state(loadPref('gridSize', 150))
  settingsOpen = $state(false)
  shortcutsOpen = $state(false)

  tasks = $state<Task[]>([])
  activeTask = $state<ActiveTask | null>(null)
  processingIds = $state<Set<number>>(new Set())
  thumbVersion = $state(0)

  modelProgress = $state<Record<string, ModelDownload>>({})
  downloadingModels = $state(false)

  toasts = $state<Toast[]>([])
  confirmRequest = $state<ConfirmRequest | null>(null)
  private toastSeq = 0

  get hasMore() {
    return this.images.length < this.total
  }

  get busy() {
    return this.activeTask?.status === 'running'
  }

  get libraryTotal() {
    return this.counts.statuses.reduce((n, s) => n + s.count, 0)
  }

  get focused(): Image | null {
    if (this.focusedId == null) return null
    return this.images.find((i) => i.id === this.focusedId) ?? null
  }

  get hasFilters() {
    const f = this.filters
    return f.year != null || f.month != null || f.status != null || f.folder != null || f.year_unknown || !!f.search
  }

  statusCount(value: string) {
    return this.counts.statuses.find((s) => s.value === value)?.count ?? 0
  }

  async init() {
    try {
      this.settings = await getSettings()
      await this.loadTasks()
      if (this.settings?.source_dir) await this.refresh()
    } catch (e) {
      this.fail(e)
    } finally {
      this.ready = true
    }
  }

  // ── Layout prefs ────────────────────────────────────────────────────────────

  toggleSidebar() {
    this.sidebarOpen = !this.sidebarOpen
    savePref('sidebar', this.sidebarOpen)
  }

  toggleInspector() {
    this.inspectorOpen = !this.inspectorOpen
    savePref('inspector', this.inspectorOpen)
  }

  setGridSize(px: number) {
    this.gridSize = Math.min(320, Math.max(90, Math.round(px)))
    savePref('gridSize', this.gridSize)
  }

  openInspector(tab: InspectorTab) {
    this.inspectorTab = tab
    if (!this.inspectorOpen) this.toggleInspector()
  }

  // ── Feedback ────────────────────────────────────────────────────────────────

  toast(text: string, kind: Toast['kind'] = 'info') {
    const id = ++this.toastSeq
    this.toasts = [...this.toasts, { id, kind, text }]
    setTimeout(() => this.dismissToast(id), kind === 'error' ? 8000 : 4000)
  }

  dismissToast(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id)
  }

  fail(e: unknown) {
    this.toast(String(e), 'error')
  }

  confirm(opts: { title: string; message: string; confirmLabel?: string; danger?: boolean }): Promise<boolean> {
    this.confirmRequest?.resolve(false)
    return new Promise((resolve) => {
      this.confirmRequest = {
        title: opts.title,
        message: opts.message,
        confirmLabel: opts.confirmLabel ?? 'Confirm',
        danger: opts.danger ?? false,
        resolve: (ok) => {
          this.confirmRequest = null
          resolve(ok)
        },
      }
    })
  }

  // ── Source folder ───────────────────────────────────────────────────────────

  async pickFolder() {
    try {
      const next = await pickSourceFolder()
      if (next) {
        this.settings = next
        this.resetView()
        await this.scan()
      }
    } catch (e) {
      this.fail(e)
    }
  }

  async setSourcePath(path: string) {
    try {
      const next = await setSourceFolder(path)
      if (next) {
        this.settings = next
        this.resetView()
        await this.scan()
      }
    } catch (e) {
      this.fail(e)
    }
  }

  private resetView() {
    this.filters = { year_unknown: false }
    this.clearSelection()
    this.focusedId = null
    this.view = 'grid'
  }

  async scan() {
    if (!this.settings?.source_dir) return
    this.scanning = true
    try {
      const n = await scanSource()
      this.loadedPages = 1
      await this.refresh()
      this.toast(n > 0 ? `Found ${n} new photo${n === 1 ? '' : 's'}` : 'No new photos found', n > 0 ? 'success' : 'info')
    } catch (e) {
      this.fail(e)
    } finally {
      this.scanning = false
    }
  }

  async pickOutput() {
    try {
      const next = await pickOutputFolder()
      if (next) this.settings = next
    } catch (e) {
      this.fail(e)
    }
  }

  async setThumbnailSize(size: number) {
    this.settings = await updateSettings(size)
    this.bustThumbs()
  }

  // ── Listing ─────────────────────────────────────────────────────────────────

  async refresh() {
    this.loading = true
    try {
      const [list, counts] = await Promise.all([
        listImages(this.filters, 1, this.loadedPages * PAGE),
        imageStats(this.filters),
      ])
      this.images = list.images
      this.total = list.total
      this.counts = counts
      const present = new Set(list.images.map((i) => i.id))
      if (this.focusedId != null && !present.has(this.focusedId)) this.focusedId = null
      if ([...this.selected].some((id) => !present.has(id))) {
        this.selected = new Set([...this.selected].filter((id) => present.has(id)))
      }
    } catch (e) {
      this.fail(e)
    } finally {
      this.loading = false
    }
  }

  async loadMore() {
    if (this.loadingMore || this.loading || !this.hasMore) return
    this.loadingMore = true
    try {
      const list = await listImages(this.filters, this.loadedPages + 1, PAGE)
      this.loadedPages++
      const seen = new Set(this.images.map((i) => i.id))
      this.images = [...this.images, ...list.images.filter((i) => !seen.has(i.id))]
      this.total = list.total
    } catch (e) {
      this.fail(e)
    } finally {
      this.loadingMore = false
    }
  }

  setFilter(patch: Partial<ImageFilters>) {
    this.filters = { ...this.filters, ...patch }
    this.loadedPages = 1
    if (this.view === 'duplicates') this.view = 'grid'
    this.refresh()
  }

  private searchTimer: ReturnType<typeof setTimeout> | undefined
  setSearch(q: string) {
    this.filters = { ...this.filters, search: q }
    this.loadedPages = 1
    if (this.view === 'duplicates') this.view = 'grid'
    if (this.searchTimer) clearTimeout(this.searchTimer)
    this.searchTimer = setTimeout(() => this.refresh(), 220)
  }

  clearFilters() {
    this.filters = { year_unknown: false }
    this.loadedPages = 1
    this.refresh()
  }

  // ── Selection & focus ───────────────────────────────────────────────────────

  /// Click semantics: plain = select only this, toggle (ctrl/cmd) = add/remove,
  /// range (shift) = everything between the anchor and this image.
  click(id: number, mode: 'single' | 'toggle' | 'range' = 'single') {
    if (mode === 'range' && this.anchorId != null) {
      const a = this.images.findIndex((i) => i.id === this.anchorId)
      const b = this.images.findIndex((i) => i.id === id)
      if (a >= 0 && b >= 0) {
        const [lo, hi] = a < b ? [a, b] : [b, a]
        this.selected = new Set(this.images.slice(lo, hi + 1).map((i) => i.id))
      }
    } else if (mode === 'toggle') {
      const next = new Set(this.selected)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      this.selected = next
      this.anchorId = id
    } else {
      this.selected = new Set([id])
      this.anchorId = id
    }
    this.focusedId = id
  }

  focus(id: number, extend = false) {
    if (extend) this.click(id, 'range')
    else this.click(id)
  }

  selectAll() {
    this.selected = new Set(this.images.map((i) => i.id))
  }

  clearSelection() {
    this.selected = new Set()
    this.anchorId = null
  }

  /// The images an action applies to: the selection, or the focused image.
  get targetIds(): number[] {
    if (this.selected.size) return [...this.selected]
    return this.focusedId != null ? [this.focusedId] : []
  }

  /// The large view always acts on the one photo shown, so the selection
  /// collapses to it.
  openLoupe(id?: number) {
    const target = id ?? this.focusedId ?? this.images[0]?.id
    if (target == null) return
    this.click(target)
    this.view = 'loupe'
  }

  step(delta: number) {
    if (!this.images.length) return
    const idx = this.images.findIndex((i) => i.id === this.focusedId)
    const next = Math.min(this.images.length - 1, Math.max(0, idx < 0 ? 0 : idx + delta))
    this.click(this.images[next].id)
    if (next >= this.images.length - 10) this.loadMore()
  }

  // ── Mutations ───────────────────────────────────────────────────────────────

  bustThumbs() {
    clearThumbCache()
    this.thumbVersion++
  }

  async deleteTargets() {
    const ids = this.targetIds
    if (!ids.length) return
    const ok = await this.confirm({
      title: `Delete ${ids.length} photo${ids.length === 1 ? '' : 's'}?`,
      message:
        'They are removed from the library along with their organized and restored copies. The original scans on disk are not touched.',
      confirmLabel: 'Delete',
      danger: true,
    })
    if (!ok) return
    try {
      const n = await bulkDelete(ids)
      this.clearSelection()
      if (this.view === 'loupe') this.view = 'grid'
      await this.refresh()
      this.toast(`Deleted ${n} photo${n === 1 ? '' : 's'}`)
    } catch (e) {
      this.fail(e)
    }
  }

  async editTargets(year: number | null, month: number | null, title: string | null) {
    const ids = this.targetIds
    if (!ids.length) return
    try {
      await bulkUpdate(ids, year, month, title)
      await this.refresh()
      this.toast(`Updated ${ids.length} photo${ids.length === 1 ? '' : 's'}`, 'success')
    } catch (e) {
      this.fail(e)
    }
  }

  async rotateTargets(clockwise: boolean) {
    const ids = this.targetIds
    if (!ids.length) return
    try {
      await bulkRotate(ids, clockwise)
      this.bustThumbs()
      await this.refresh()
    } catch (e) {
      this.fail(e)
    }
  }

  async setFolderTargets(folder: string | null) {
    const ids = this.targetIds
    if (!ids.length) return
    try {
      await setFolder(ids, folder)
      await this.refresh()
      this.toast(folder ? `Moved to “${folder}”` : 'Removed from folder', 'success')
    } catch (e) {
      this.fail(e)
    }
  }

  // ── Processing ──────────────────────────────────────────────────────────────

  async loadTasks() {
    try {
      this.tasks = await listTasks()
    } catch (e) {
      this.fail(e)
    }
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
          startedAt: Date.now(),
        }
        break
      case 'image_started':
        this.processingIds = new Set([e.image_id])
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
    const imageIds = all ? [] : this.targetIds
    if (!all && imageIds.length === 0) return

    const channel = new Channel<ProgressEvent>()
    channel.onmessage = (e) => this.onProgress(e)
    try {
      await startBatch(imageIds, all, steps, channel)
      const t = this.activeTask
      if (t) {
        const done = `Restored ${t.completed} photo${t.completed === 1 ? '' : 's'}`
        if (t.status === 'cancelled') this.toast(`Cancelled · ${t.completed} of ${t.total} done`)
        else if (t.failed) this.toast(`${done} · ${t.failed} failed`, 'error')
        else this.toast(done, 'success')
      }
    } catch (e) {
      this.fail(e)
    } finally {
      this.bustThumbs()
      this.processingIds = new Set()
      this.activeTask = null
      await Promise.all([this.loadTasks(), this.refresh()])
    }
  }

  async cancelActive() {
    if (this.activeTask) await cancelTask(this.activeTask.id)
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
      this.fail(e)
    } finally {
      this.downloadingModels = false
    }
    return ok
  }
}

export const store = new AppStore()

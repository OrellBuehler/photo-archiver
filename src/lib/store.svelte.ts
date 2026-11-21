import { Channel } from '@tauri-apps/api/core'
import {
  bulkDelete,
  bulkRotate,
  bulkUpdate,
  cancelTask,
  downloadModels,
  getSettings,
  imageStats,
  listImageIds,
  listImages,
  listTasks,
  openOutputDir,
  pickOutputFolder,
  pickSourceFolder,
  redoImage,
  resetImages,
  revealImage,
  scanSource,
  setFolder,
  setSourceFolder,
  startBatch,
  undoImage,
  updateImage,
  updateSettings,
} from './api'
import { clearThumbCache, invalidateThumb } from './thumbs'
import {
  MONTHS,
  STATUS,
  plural,
  type AppSettings,
  type FilterCounts,
  type Image,
  type ImageFilters,
  type ModelDownload,
  type ModelEvent,
  type ProgressEvent,
  type Task,
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
export type MetaField = 'title' | 'year' | 'folder'

/// One place in the sidebar: everything but the search text.
export type Place = Omit<Partial<ImageFilters>, 'search'>

export interface Meta {
  title: string | null
  year: number | null
  month: number | null
  folder: string | null
}

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

export interface MenuItem {
  label: string
  icon?: string
  hint?: string
  danger?: boolean
  separator?: boolean
  run: () => void
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
  gridFit = $state(loadPref('gridFit', true))
  settingsOpen = $state(false)
  shortcutsOpen = $state(false)

  /// The date/folder last typed in, for "same as last photo".
  lastMeta = $state<Omit<Meta, 'title'> | null>(loadPref('lastMeta', null))
  /// Which inspector field to focus once the next photo's form mounts.
  refocus = $state<MetaField | null>(null)

  tasks = $state<Task[]>([])
  activeTask = $state<ActiveTask | null>(null)
  processingIds = $state<Set<number>>(new Set())
  thumbVersion = $state(0)

  modelProgress = $state<Record<string, ModelDownload>>({})
  downloadingModels = $state(false)

  toasts = $state<Toast[]>([])
  confirmRequest = $state<ConfirmRequest | null>(null)
  menu = $state<{ x: number; y: number; items: MenuItem[] } | null>(null)
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

  get undated() {
    return this.counts.years.find((y) => y.value === 'unknown')?.count ?? 0
  }

  get focused(): Image | null {
    if (this.focusedId == null) return null
    return this.images.find((i) => i.id === this.focusedId) ?? null
  }

  get hasPlace() {
    const f = this.filters
    return f.year != null || f.month != null || f.status != null || f.folder != null || f.year_unknown
  }

  get hasFilters() {
    return this.hasPlace || !!this.filters.search
  }

  /// What the main view is showing, in words.
  get placeLabel() {
    const f = this.filters
    if (f.status) return f.status === 'new' ? 'Inbox' : (STATUS[f.status]?.label ?? f.status)
    if (f.folder) return f.folder
    if (f.year_unknown) return 'Needs a date'
    if (f.year != null) return f.month != null ? `${MONTHS[f.month]} ${f.year}` : String(f.year)
    return 'All photos'
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

  toggleGridFit() {
    this.gridFit = !this.gridFit
    savePref('gridFit', this.gridFit)
  }

  openInspector(tab: InspectorTab) {
    this.inspectorTab = tab
    if (!this.inspectorOpen) this.toggleInspector()
  }

  /// Jump into the date field of the focused photo.
  editDate() {
    if (this.focusedId == null && this.images.length) this.click(this.images[0].id)
    if (this.focusedId == null) return
    if (this.selected.size > 1) this.click(this.focusedId)
    this.openInspector('info')
    this.refocus = 'year'
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

  // ── Folders ─────────────────────────────────────────────────────────────────

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
      this.toast(n > 0 ? `Found ${plural(n, 'new photo')}` : 'No new photos — the library is up to date', n > 0 ? 'success' : 'info')
    } catch (e) {
      this.fail(e)
    } finally {
      this.scanning = false
    }
  }

  async pickOutput() {
    const filed = this.libraryTotal - this.statusCount('new')
    if (filed > 0) {
      const ok = await this.confirm({
        title: 'Change the archive folder?',
        message: `${plural(filed, 'photo')} already filed will not be moved. Move the contents of the current archive folder to the new one yourself, or those photos lose their copies.`,
        confirmLabel: 'Choose folder',
      })
      if (!ok) return
    }
    try {
      const next = await pickOutputFolder()
      if (next) {
        this.settings = next
        this.bustThumbs()
      }
    } catch (e) {
      this.fail(e)
    }
  }

  async openOutput() {
    try {
      await openOutputDir()
    } catch (e) {
      this.fail(e)
    }
  }

  async reveal(id: number) {
    try {
      await revealImage(id)
    } catch (e) {
      this.fail(e)
    }
  }

  async setThumbnailSize(size: number) {
    this.settings = await updateSettings(size)
    this.bustThumbs()
  }

  // ── Listing ─────────────────────────────────────────────────────────────────

  /// Reload the list. `keep` pins the photo being edited in place even if the
  /// edit just moved it out of this view, so its form doesn't vanish mid-typing.
  async refresh(keep?: Image) {
    this.loading = true
    const prevIdx = this.images.findIndex((i) => i.id === this.focusedId)
    try {
      const [list, counts, ids] = await Promise.all([
        listImages(this.filters, 1, this.loadedPages * PAGE),
        imageStats(this.filters),
        this.selected.size > 1 ? listImageIds(this.filters) : null,
      ])
      let images = list.images
      const present = new Set(images.map((i) => i.id))
      if (this.focusedId != null && !present.has(this.focusedId)) {
        if (keep && keep.id === this.focusedId) {
          images = [...images]
          images.splice(Math.min(Math.max(prevIdx, 0), images.length), 0, keep)
        } else if (this.view === 'loupe' && images.length) {
          this.focusedId = images[Math.min(Math.max(prevIdx, 0), images.length - 1)].id
          this.selected = new Set([this.focusedId])
        } else {
          this.focusedId = null
          if (this.view === 'loupe') this.view = 'grid'
        }
      }
      this.images = images
      this.total = list.total
      this.counts = counts
      const valid = ids ? new Set(ids) : new Set(images.map((i) => i.id))
      if ([...this.selected].some((id) => !valid.has(id))) {
        this.selected = new Set([...this.selected].filter((id) => valid.has(id)))
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

  private applyFilters(next: ImageFilters) {
    this.filters = next
    this.loadedPages = 1
    this.clearSelection()
    if (this.view === 'duplicates') this.view = 'grid'
  }

  /// Go to one place in the library (the search text carries over).
  show(place: Place = {}) {
    this.applyFilters({ year_unknown: false, search: this.filters.search, ...place })
    this.focusedId = null
    this.view = 'grid'
    return this.refresh()
  }

  private searchTimer: ReturnType<typeof setTimeout> | undefined
  setSearch(q: string) {
    this.applyFilters({ ...this.filters, search: q })
    if (this.searchTimer) clearTimeout(this.searchTimer)
    this.searchTimer = setTimeout(() => this.refresh(), 220)
  }

  clearFilters() {
    this.applyFilters({ year_unknown: false })
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

  /// Everything in the current view, including pages not loaded yet.
  async selectAll() {
    try {
      this.selected = new Set(this.hasMore ? await listImageIds(this.filters) : this.images.map((i) => i.id))
    } catch (e) {
      this.fail(e)
    }
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
      title: `Remove ${plural(ids.length, 'photo')}?`,
      message:
        'They leave the library and their copies in the archive are deleted. The original scans are not touched — a rescan brings them back.',
      confirmLabel: 'Remove',
      danger: true,
    })
    if (!ok) return
    try {
      const n = await bulkDelete(ids)
      this.clearSelection()
      await this.refresh()
      this.toast(`Removed ${plural(n, 'photo')}`)
    } catch (e) {
      this.fail(e)
    }
  }

  /// Throw away the archive copies and every edit; the photos are new again.
  async resetTargets() {
    const ids = this.targetIds
    if (!ids.length) return
    const ok = await this.confirm({
      title: `Start over with ${plural(ids.length, 'photo')}?`,
      message: 'Their copies in the archive and all restoration steps are discarded. They go back to the inbox as untouched scans.',
      confirmLabel: 'Start over',
      danger: true,
    })
    if (!ok) return
    try {
      await resetImages(ids)
      this.bustThumbs()
      await this.refresh()
    } catch (e) {
      this.fail(e)
    }
  }

  private remember(m: Omit<Meta, 'title'>) {
    if (m.year == null && !m.folder) return
    this.lastMeta = { year: m.year, month: m.month, folder: m.folder }
    savePref('lastMeta', this.lastMeta)
  }

  /// Save one photo's details. With `advance`, move on to the next photo and
  /// keep typing in the same field — the fast way through a stack of scans.
  async saveMeta(image: Image, m: Meta, advance: MetaField | null = null) {
    const idx = this.images.findIndex((i) => i.id === image.id)
    const nextId = (this.images[idx + 1] ?? this.images[idx - 1])?.id
    try {
      if (m.title !== image.title || m.year !== image.year || m.month !== image.month) {
        await updateImage(image.id, m.year, m.month, m.title)
      }
      if (m.folder !== image.folder) await setFolder([image.id], m.folder)
      this.remember(m)
      if (advance && nextId != null && this.focusedId === image.id) {
        this.click(nextId)
        this.refocus = advance
        await this.refresh()
      } else {
        await this.refresh({ ...image, ...m })
      }
    } catch (e) {
      this.fail(e)
    }
  }

  async editTargets(year: number | null, month: number | null, title: string | null) {
    const ids = this.targetIds
    if (!ids.length) return
    try {
      await bulkUpdate(ids, year, month, title)
      if (year != null) this.remember({ year, month, folder: null })
      await this.refresh()
      this.toast(`Updated ${plural(ids.length, 'photo')}`, 'success')
    } catch (e) {
      this.fail(e)
    }
  }

  /// Give the targets the same date (and folder) as the last photo edited.
  async applyLast(advance = false) {
    const last = this.lastMeta
    const ids = this.targetIds
    if (!last || !ids.length) return
    const idx = this.images.findIndex((i) => i.id === this.focusedId)
    const nextId = ids.length === 1 ? (this.images[idx + 1] ?? this.images[idx - 1])?.id : undefined
    try {
      if (last.year != null) await bulkUpdate(ids, last.year, last.month, null)
      if (last.folder) await setFolder(ids, last.folder)
      if (advance && nextId != null) this.click(nextId)
      await this.refresh()
      if (ids.length > 1) this.toast(`Dated ${plural(ids.length, 'photo')}`, 'success')
    } catch (e) {
      this.fail(e)
    }
  }

  async rotateTargets(clockwise: boolean) {
    const ids = this.targetIds
    if (!ids.length) return
    try {
      const n = await bulkRotate(ids, clockwise)
      this.bustThumbs()
      await this.refresh()
      if (n < ids.length) this.toast(`${plural(ids.length - n, 'photo')} could not be rotated`, 'error')
    } catch (e) {
      this.fail(e)
    }
  }

  async setFolderTargets(folder: string | null) {
    const ids = this.targetIds
    if (!ids.length) return
    try {
      await setFolder(ids, folder)
      if (folder) this.remember({ year: this.lastMeta?.year ?? null, month: this.lastMeta?.month ?? null, folder })
      await this.refresh()
      this.toast(folder ? `Moved to “${folder}”` : 'Removed from folder', 'success')
    } catch (e) {
      this.fail(e)
    }
  }

  async undoRedo(id: number, redo: boolean) {
    try {
      await (redo ? redoImage(id) : undoImage(id))
      invalidateThumb(id)
      this.thumbVersion++
      await this.refresh()
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

  /// Run the clean-up `steps` on `ids`. Photos are always filed first; steps a
  /// photo already has are skipped unless `force` starts it over.
  async restore(ids: number[], steps: string[], force = false) {
    if (this.busy || ids.length === 0) return
    const channel = new Channel<ProgressEvent>()
    channel.onmessage = (e) => this.onProgress(e)
    try {
      await startBatch(ids, ['organize', ...steps], force, channel)
      const t = this.activeTask
      if (t) {
        const done = `${plural(t.completed, 'photo')} done`
        if (t.status === 'cancelled') this.toast(`Stopped · ${t.completed} of ${t.total} done`)
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

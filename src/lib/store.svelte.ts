import { getSettings, imageStats, listImages, pickSourceFolder, scanSource } from './api'
import type { AppSettings, FilterCounts, Image, ImageFilters } from './types'

const emptyCounts: FilterCounts = {
  years: [],
  months: [],
  statuses: [],
  steps: [],
  total: 0,
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

  get pages() {
    return Math.max(1, Math.ceil(this.total / this.perPage))
  }

  async init() {
    this.settings = await getSettings()
    if (this.settings?.source_dir) await this.refresh()
  }

  async pickFolder() {
    const next = await pickSourceFolder()
    if (next) {
      this.settings = next
      await this.scan()
    }
  }

  async scan() {
    if (!this.settings?.source_dir) return
    this.scanning = true
    try {
      this.lastScanCount = await scanSource()
      this.page = 1
      await this.refresh()
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
}

export const store = new AppStore()

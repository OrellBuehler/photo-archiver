import {
  createDockview,
  themeVisualStudio,
  type DockviewApi,
  type IContentRenderer,
  type GroupPanelPartInitParameters,
} from 'dockview-core'
import { mount, unmount, type Component } from 'svelte'
import Library from './panels/Library.svelte'
import Filters from './panels/Filters.svelte'
import Processing from './panels/Processing.svelte'
import Tasks from './panels/Tasks.svelte'
import Viewer from './panels/Viewer.svelte'
import Duplicates from './panels/Duplicates.svelte'
import Settings from './panels/Settings.svelte'

// Maps a dockview component name to the Svelte component that renders it.
const registry: Record<string, Component<any>> = {
  library: Library,
  filters: Filters,
  processing: Processing,
  tasks: Tasks,
  viewer: Viewer,
  duplicates: Duplicates,
  settings: Settings,
}

function createRenderer(name: string): IContentRenderer {
  const element = document.createElement('div')
  element.style.height = '100%'
  let instance: Record<string, unknown> | undefined

  return {
    element,
    init(params: GroupPanelPartInitParameters) {
      const Comp = registry[name] ?? Library
      instance = mount(Comp, { target: element, props: { ...params.params } })
    },
    dispose() {
      if (instance) unmount(instance)
    },
  }
}

const LAYOUT_KEY = 'photo-archiver:layout'
// Bump when the default panel set OR default sizing changes so stale saved
// layouts are discarded.
const LAYOUT_VERSION = 3
// Panels that must exist for a restored layout to be considered valid. Settings
// is intentionally absent — it is opened on demand from the titlebar menu.
const EXPECTED_PANELS = ['library', 'viewer', 'duplicates', 'filters', 'processing', 'tasks']

// Title + how to re-create each panel when opened from the menu after being
// closed. Components share their id name (see `registry`).
const PANEL_META: Record<string, { title: string }> = {
  library: { title: 'Library' },
  filters: { title: 'Filters' },
  processing: { title: 'Processing' },
  tasks: { title: 'Tasks' },
  viewer: { title: 'Viewer' },
  duplicates: { title: 'Duplicates' },
  settings: { title: 'Settings' },
}

// Focus a panel, re-creating it as a tab in the centre group if it was closed.
export function openPanel(api: DockviewApi, id: string) {
  const existing = api.getPanel(id)
  if (existing) {
    existing.api.setActive()
    return
  }
  const meta = PANEL_META[id]
  if (!meta) return
  const ref = api.getPanel('library') ?? api.panels[0]
  api.addPanel({
    id,
    component: id,
    title: meta.title,
    position: ref ? { referencePanel: ref.id, direction: 'within' } : undefined,
  })
  api.getPanel(id)?.api.setActive()
}

// Default proportions for the three columns. Library keeps the lion's share;
// Filters and the Processing/Tasks column get enough room to be usable.
const FILTERS_FRACTION = 0.17
const PROCESSING_FRACTION = 0.28
const TASKS_HEIGHT_FRACTION = 0.34

function buildDefaultLayout(api: DockviewApi, parent: HTMLElement) {
  api.addPanel({ id: 'library', component: 'library', title: 'Library' })
  api.addPanel({ id: 'viewer', component: 'viewer', title: 'Viewer', inactive: true })
  api.addPanel({ id: 'duplicates', component: 'duplicates', title: 'Duplicates', inactive: true })
  api.addPanel({
    id: 'filters',
    component: 'filters',
    title: 'Filters',
    minimumWidth: 200,
    position: { referencePanel: 'library', direction: 'left' },
  })
  api.addPanel({
    id: 'processing',
    component: 'processing',
    title: 'Processing',
    minimumWidth: 300,
    minimumHeight: 160,
    position: { referencePanel: 'library', direction: 'right' },
  })
  api.addPanel({
    id: 'tasks',
    component: 'tasks',
    title: 'Tasks',
    minimumHeight: 140,
    position: { referencePanel: 'processing', direction: 'below' },
  })

  api.getPanel('library')?.api.setActive()
  applyDefaultSizes(api, parent)
}

// dockview applies `initialWidth`/`initialHeight` against the container's size
// at addPanel time, which is still ~zero during the initial mount — so the side
// columns collapse to their minimums. Instead we size them explicitly once the
// container reports real dimensions.
function applyDefaultSizes(api: DockviewApi, parent: HTMLElement, attempt = 0) {
  const width = parent.clientWidth
  const height = parent.clientHeight
  if ((width < 2 || height < 2) && attempt < 30) {
    requestAnimationFrame(() => applyDefaultSizes(api, parent, attempt + 1))
    return
  }

  api.layout(width, height)
  api.getPanel('filters')?.api.setSize({ width: Math.round(width * FILTERS_FRACTION) })
  api.getPanel('processing')?.api.setSize({ width: Math.round(width * PROCESSING_FRACTION) })
  api.getPanel('tasks')?.api.setSize({ height: Math.round(height * TASKS_HEIGHT_FRACTION) })
}

function restoreLayout(api: DockviewApi): boolean {
  let raw: string | null = null
  try {
    raw = localStorage.getItem(LAYOUT_KEY)
  } catch {
    return false
  }
  if (!raw) return false

  try {
    const saved = JSON.parse(raw)
    if (saved?.version !== LAYOUT_VERSION || !saved.layout) return false
    api.fromJSON(saved.layout)
    // Guard against a stale layout that is missing panels we now ship.
    const present = new Set(api.panels.map((p) => p.id))
    if (!EXPECTED_PANELS.every((id) => present.has(id))) {
      api.clear()
      return false
    }
    return true
  } catch {
    try {
      api.clear()
    } catch {}
    return false
  }
}

function saveLayout(api: DockviewApi) {
  try {
    const payload = { version: LAYOUT_VERSION, layout: api.toJSON() }
    localStorage.setItem(LAYOUT_KEY, JSON.stringify(payload))
  } catch {}
}

export function createWorkspace(parent: HTMLElement): DockviewApi {
  const api = createDockview(parent, {
    theme: themeVisualStudio,
    createComponent: (options) => createRenderer(options.name),
  })

  if (!restoreLayout(api)) buildDefaultLayout(api, parent)

  // Persist user layout changes (resize, move, tab reorder) after they settle.
  let saveTimer: ReturnType<typeof setTimeout> | undefined
  api.onDidLayoutChange(() => {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(() => saveLayout(api), 400)
  })

  return api
}

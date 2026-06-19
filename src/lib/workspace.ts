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

// Maps a dockview component name to the Svelte component that renders it.
const registry: Record<string, Component<any>> = {
  library: Library,
  filters: Filters,
  processing: Processing,
  tasks: Tasks,
  viewer: Viewer,
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

export function createWorkspace(parent: HTMLElement): DockviewApi {
  const api = createDockview(parent, {
    theme: themeVisualStudio,
    createComponent: (options) => createRenderer(options.name),
  })

  api.addPanel({ id: 'library', component: 'library', title: 'Library' })
  api.addPanel({ id: 'viewer', component: 'viewer', title: 'Viewer', inactive: true })
  api.addPanel({
    id: 'filters',
    component: 'filters',
    title: 'Filters',
    position: { referencePanel: 'library', direction: 'left' },
  })
  api.addPanel({
    id: 'processing',
    component: 'processing',
    title: 'Processing',
    position: { referencePanel: 'library', direction: 'right' },
  })
  api.addPanel({
    id: 'tasks',
    component: 'tasks',
    title: 'Tasks',
    position: { referencePanel: 'processing', direction: 'below' },
  })

  api.getPanel('library')?.api.setActive()
  return api
}

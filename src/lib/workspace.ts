import {
  createDockview,
  themeVisualStudio,
  type DockviewApi,
  type IContentRenderer,
  type GroupPanelPartInitParameters,
} from 'dockview-core'
import { mount, unmount, type Component } from 'svelte'
import Placeholder from './panels/Placeholder.svelte'

// Maps a dockview component name to the Svelte component that renders it.
const registry: Record<string, Component<any>> = {
  placeholder: Placeholder,
}

function createRenderer(name: string): IContentRenderer {
  const element = document.createElement('div')
  element.style.height = '100%'
  let instance: Record<string, unknown> | undefined

  return {
    element,
    init(params: GroupPanelPartInitParameters) {
      const Comp = registry[name] ?? Placeholder
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

  api.addPanel({
    id: 'library',
    component: 'placeholder',
    title: 'Library',
    params: { title: 'Library', hint: 'Your photo gallery will live here.' },
  })
  api.addPanel({
    id: 'filters',
    component: 'placeholder',
    title: 'Filters',
    params: { title: 'Filters', hint: 'Filter by year, month, status, and step.' },
    position: { referencePanel: 'library', direction: 'left' },
  })
  api.addPanel({
    id: 'processing',
    component: 'placeholder',
    title: 'Processing',
    params: { title: 'Processing', hint: 'Choose pipeline steps and run them.' },
    position: { referencePanel: 'library', direction: 'right' },
  })
  api.addPanel({
    id: 'tasks',
    component: 'placeholder',
    title: 'Tasks',
    params: { title: 'Tasks', hint: 'Live task progress and history.' },
    position: { referencePanel: 'library', direction: 'below' },
  })

  return api
}

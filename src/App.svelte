<script lang="ts">
  import { onMount } from 'svelte'
  import type { DockviewApi } from 'dockview-core'
  import Titlebar from './lib/Titlebar.svelte'
  import StageBar from './lib/StageBar.svelte'
  import { createWorkspace } from './lib/workspace'
  import { store } from './lib/store.svelte'

  let dockEl: HTMLDivElement
  let api: DockviewApi
  let activePanelId = $state('library')

  onMount(() => {
    api = createWorkspace(dockEl)
    store.dockApi = api
    store.init()
    const sub = api.onDidActivePanelChange((e) => {
      if (e.panel?.id) activePanelId = e.panel.id
    })
    return () => {
      sub.dispose()
      api?.dispose()
    }
  })
</script>

<Titlebar />

<div bind:this={dockEl} class="min-h-0 flex-1"></div>

<StageBar {activePanelId} />

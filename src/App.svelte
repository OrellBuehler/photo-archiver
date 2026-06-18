<script lang="ts">
  import { onMount } from 'svelte'
  import type { DockviewApi } from 'dockview-core'
  import Titlebar from './lib/Titlebar.svelte'
  import { createWorkspace } from './lib/workspace'

  let dockEl: HTMLDivElement
  let api: DockviewApi

  onMount(() => {
    api = createWorkspace(dockEl)
    return () => api?.dispose()
  })
</script>

<Titlebar />

<div bind:this={dockEl} class="min-h-0 flex-1"></div>

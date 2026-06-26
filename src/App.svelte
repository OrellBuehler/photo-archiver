<script lang="ts">
  import { onMount } from 'svelte'
  import { getCurrentWebview } from '@tauri-apps/api/webview'
  import type { DockviewApi } from 'dockview-core'
  import Titlebar from './lib/Titlebar.svelte'
  import StageBar from './lib/StageBar.svelte'
  import Icon from './lib/ui/Icon.svelte'
  import { createWorkspace } from './lib/workspace'
  import { store } from './lib/store.svelte'

  let dockEl: HTMLDivElement
  let api: DockviewApi
  let activePanelId = $state('library')
  let dropActive = $state(false)

  onMount(() => {
    api = createWorkspace(dockEl)
    store.dockApi = api
    store.init()
    const sub = api.onDidActivePanelChange((e) => {
      if (e.panel?.id) activePanelId = e.panel.id
    })

    // Drop a folder anywhere on the window to load it as the source.
    let unlistenDrop: (() => void) | undefined
    void getCurrentWebview()
      .onDragDropEvent((e) => {
        const p = e.payload
        if (p.type === 'enter' || p.type === 'over') dropActive = true
        else if (p.type === 'leave') dropActive = false
        else if (p.type === 'drop') {
          dropActive = false
          if (p.paths?.length) store.setSourcePath(p.paths[0])
        }
      })
      .then((un) => (unlistenDrop = un))

    return () => {
      sub.dispose()
      api?.dispose()
      unlistenDrop?.()
    }
  })
</script>

<Titlebar />

<div bind:this={dockEl} class="min-h-0 flex-1"></div>

<StageBar {activePanelId} />

{#if dropActive}
  <div class="pointer-events-none fixed inset-0 z-50 grid place-items-center" style="background: var(--bg-overlay);">
    <div class="flex flex-col items-center gap-3 rounded-[12px] border-2 border-dashed border-brand bg-surface px-10 py-8 shadow-[var(--shadow-lg)]">
      <Icon name="folderOpen" size={36} class="text-brand" />
      <p class="font-serif text-xl text-ink">Drop a folder to load it</p>
    </div>
  </div>
{/if}

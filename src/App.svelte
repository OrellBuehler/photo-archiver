<script lang="ts">
  import { onMount } from 'svelte'
  import { getCurrentWebview } from '@tauri-apps/api/webview'
  import { store } from './lib/store.svelte'
  import Titlebar from './lib/shell/Titlebar.svelte'
  import Sidebar from './lib/shell/Sidebar.svelte'
  import StatusBar from './lib/shell/StatusBar.svelte'
  import Inspector from './lib/inspector/Inspector.svelte'
  import Grid from './lib/views/Grid.svelte'
  import Loupe from './lib/views/Loupe.svelte'
  import Duplicates from './lib/views/Duplicates.svelte'
  import Welcome from './lib/views/Welcome.svelte'
  import Settings from './lib/overlays/Settings.svelte'
  import Shortcuts from './lib/overlays/Shortcuts.svelte'
  import Confirm from './lib/overlays/Confirm.svelte'
  import Toasts from './lib/overlays/Toasts.svelte'
  import Icon from './lib/ui/Icon.svelte'
  import Spinner from './lib/ui/Spinner.svelte'

  let dropActive = $state(false)
  const hasLib = $derived(!!store.settings?.source_dir)

  onMount(() => {
    store.init()

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
      .catch(() => {})

    return () => unlistenDrop?.()
  })

  // App-wide shortcuts. View-specific keys (arrows, zoom…) live in the views.
  function onKey(e: KeyboardEvent) {
    if (e.defaultPrevented || store.confirmRequest) return
    const t = e.target as HTMLElement
    const typing = t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable)
    const mod = e.ctrlKey || e.metaKey
    const k = e.key.toLowerCase()

    if (mod && k === ',') {
      e.preventDefault()
      store.settingsOpen = true
      return
    }
    if (!hasLib || store.settingsOpen || store.shortcutsOpen) return
    if (mod && k === 'b') {
      e.preventDefault()
      store.toggleSidebar()
    } else if (mod && k === 'i') {
      e.preventDefault()
      store.toggleInspector()
    } else if (mod && k === 'f') {
      e.preventDefault()
      document.getElementById('global-search')?.focus()
    }
    if (typing) return

    if (e.key === '/') {
      e.preventDefault()
      document.getElementById('global-search')?.focus()
    } else if (e.key === '?') {
      store.shortcutsOpen = true
    } else if (mod && k === 'a' && store.view === 'grid') {
      e.preventDefault()
      store.selectAll()
    } else if (e.key === 'Escape' && store.view === 'grid') {
      if (store.selected.size) store.clearSelection()
      else if (store.hasFilters) store.clearFilters()
    } else if (e.key === 'Escape' && store.view === 'duplicates') {
      store.view = 'grid'
    } else if (store.view === 'duplicates' || mod || e.altKey) {
      return
    } else if (e.key === '[') {
      store.rotateTargets(false)
    } else if (e.key === ']') {
      store.rotateTargets(true)
    } else if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault()
      store.deleteTargets()
    } else if (k === 'r') {
      store.openInspector('restore')
    } else if (k === 'i') {
      store.openInspector('info')
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<Titlebar />

<div class="flex min-h-0 flex-1">
  {#if !store.ready}
    <div class="grid flex-1 place-items-center"><Spinner size={20} /></div>
  {:else if !hasLib}
    <main class="min-w-0 flex-1"><Welcome /></main>
  {:else}
    {#if store.sidebarOpen}<Sidebar />{/if}
    <main class="min-w-0 flex-1 bg-base">
      {#if store.view === 'loupe'}
        <Loupe />
      {:else if store.view === 'duplicates'}
        <Duplicates />
      {:else}
        <Grid />
      {/if}
    </main>
    {#if store.inspectorOpen}<Inspector />{/if}
  {/if}
</div>

<StatusBar />

{#if store.settingsOpen}<Settings />{/if}
{#if store.shortcutsOpen}<Shortcuts />{/if}
<Confirm />
<Toasts />

{#if dropActive}
  <div class="scrim pointer-events-none fixed inset-0 z-[80] grid place-items-center">
    <div class="pop flex flex-col items-center gap-3 rounded-[14px] border-2 border-dashed border-brand bg-surface px-12 py-9 shadow-[var(--shadow-lg)]">
      <Icon name="folderOpen" size={36} class="text-brand" />
      <p class="font-serif text-xl text-ink">Drop to open this folder</p>
    </div>
  </div>
{/if}

<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { onMount } from 'svelte'
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'

  const appWindow = getCurrentWindow()
  let maximized = $state(false)
  const hasLib = $derived(!!store.settings?.source_dir)

  onMount(() => {
    appWindow.isMaximized().then((v) => (maximized = v))
    const un = appWindow.onResized(async () => {
      maximized = await appWindow.isMaximized()
    })
    return () => {
      un.then((f) => f())
    }
  })
</script>

<header
  class="grid h-10 shrink-0 grid-cols-[1fr_minmax(0,420px)_1fr] items-center border-b border-line bg-bar select-none"
  data-tauri-drag-region
>
  <div class="flex min-w-0 items-center gap-1 pl-3" data-tauri-drag-region>
    <svg width="18" height="18" viewBox="0 0 64 64" fill="none" aria-hidden="true" class="mr-1.5 shrink-0">
      <rect x="2" y="2" width="60" height="60" rx="14" fill="#e08a3c" />
      <rect x="16" y="20" width="32" height="26" rx="4" fill="#2a1a06" fill-opacity="0.25" />
      <rect x="18" y="16" width="28" height="24" rx="4" fill="#fdf6ec" />
      <circle cx="26" cy="24" r="3.2" fill="#e08a3c" />
      <path d="M19 39 L29 30 L34 34 L40 28 L45 34 L45 38 a2 2 0 0 1 -2 2 H21 a2 2 0 0 1 -2 -2 Z" fill="#c9772f" />
    </svg>
    <span class="mr-3 text-xs font-semibold text-ink" data-tauri-drag-region>Photo Archiver</span>
    {#if hasLib}
      <button
        class="icon-btn"
        class:icon-btn-active={store.sidebarOpen}
        title="Toggle sidebar (Ctrl+B)"
        aria-label="Toggle sidebar"
        onclick={() => store.toggleSidebar()}
      >
        <Icon name="panelLeft" size={15} />
      </button>
    {/if}
  </div>

  <div class="flex justify-center" data-tauri-drag-region>
    {#if hasLib}
      <label class="relative w-full">
        <Icon name="search" size={14} class="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-ink-faint" />
        <input
          id="global-search"
          class="input h-7 rounded-full bg-base pr-12 pl-8"
          placeholder="Search filename, title, scan id…"
          value={store.filters.search ?? ''}
          oninput={(e) => store.setSearch(e.currentTarget.value)}
          onkeydown={(e) => {
            if (e.key === 'Escape') {
              store.setSearch('')
              e.currentTarget.blur()
            } else if (e.key === 'Enter' || e.key === 'ArrowDown') {
              e.currentTarget.blur()
            }
          }}
        />
        {#if store.filters.search}
          <button
            class="absolute top-1/2 right-2 grid h-5 w-5 -translate-y-1/2 place-items-center rounded-full text-ink-faint hover:bg-surface-2 hover:text-ink"
            aria-label="Clear search"
            onclick={() => store.setSearch('')}
          >
            <Icon name="x" size={12} />
          </button>
        {:else}
          <span class="kbd pointer-events-none absolute top-1/2 right-2 -translate-y-1/2">/</span>
        {/if}
      </label>
    {/if}
  </div>

  <div class="flex h-full items-center justify-end" data-tauri-drag-region>
    {#if hasLib}
      <button
        class="icon-btn"
        class:icon-btn-active={store.inspectorOpen}
        title="Toggle inspector (Ctrl+I)"
        aria-label="Toggle inspector"
        onclick={() => store.toggleInspector()}
      >
        <Icon name="panelRight" size={15} />
      </button>
    {/if}
    <button class="icon-btn" title="Keyboard shortcuts (?)" aria-label="Keyboard shortcuts" onclick={() => (store.shortcutsOpen = true)}>
      <Icon name="keyboard" size={15} />
    </button>
    <button class="icon-btn mr-2" title="Settings (Ctrl+,)" aria-label="Settings" onclick={() => (store.settingsOpen = true)}>
      <Icon name="settings" size={15} />
    </button>

    <div class="flex h-full border-l border-line">
      <button
        class="grid h-full w-11 place-items-center text-ink-dim hover:bg-surface-2 hover:text-ink"
        title="Minimize"
        aria-label="Minimize"
        onclick={() => appWindow.minimize()}
      >
        <svg width="10" height="10" viewBox="0 0 10 10"><rect y="4.5" width="10" height="1" fill="currentColor" /></svg>
      </button>
      <button
        class="grid h-full w-11 place-items-center text-ink-dim hover:bg-surface-2 hover:text-ink"
        title={maximized ? 'Restore' : 'Maximize'}
        aria-label="Maximize"
        onclick={() => appWindow.toggleMaximize()}
      >
        {#if maximized}
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"><rect x="0.5" y="2.5" width="6" height="6" /><path d="M3 2.5V0.5H9.5V7" /></svg>
        {:else}
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"><rect x="0.5" y="0.5" width="9" height="9" /></svg>
        {/if}
      </button>
      <button
        class="grid h-full w-11 place-items-center text-ink-dim hover:bg-[#c42b1c] hover:text-white"
        title="Close"
        aria-label="Close"
        onclick={() => appWindow.close()}
      >
        <svg width="10" height="10" viewBox="0 0 10 10" stroke="currentColor"><path d="M0.5 0.5l9 9M9.5 0.5l-9 9" /></svg>
      </button>
    </div>
  </div>
</header>

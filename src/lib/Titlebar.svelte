<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { onMount } from 'svelte'
  import { store } from './store.svelte'
  import Icon from './ui/Icon.svelte'

  const appWindow = getCurrentWindow()
  let maximized = $state(false)
  let openMenu = $state<string | null>(null)

  // Panels reachable from the View menu (re-creates them if they were closed).
  const VIEW_ITEMS = [
    { id: 'library', label: 'Library', icon: 'image' },
    { id: 'filters', label: 'Filters', icon: 'sliders' },
    { id: 'processing', label: 'Processing', icon: 'wand' },
    { id: 'viewer', label: 'Viewer', icon: 'columns' },
    { id: 'duplicates', label: 'Duplicates', icon: 'copy' },
    { id: 'tasks', label: 'Tasks', icon: 'listChecks' },
  ]

  function toggle(menu: string) {
    openMenu = openMenu === menu ? null : menu
  }

  function open(id: string) {
    store.openPanel(id)
    openMenu = null
  }

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

<div
  class="flex h-8 shrink-0 items-center border-b border-line bg-bar shadow-[var(--shadow-inset)] select-none"
  data-tauri-drag-region
>
  <div class="flex items-center gap-2 pr-2 pl-3 text-xs font-medium text-ink-dim" data-tauri-drag-region>
    <svg width="16" height="16" viewBox="0 0 64 64" fill="none" aria-hidden="true">
      <rect x="2" y="2" width="60" height="60" rx="14" fill="#e08a3c" />
      <rect x="2" y="2" width="60" height="60" rx="14" fill="url(#tbg)" fill-opacity="0.18" />
      <rect x="16" y="20" width="32" height="26" rx="4" fill="#2a1a06" fill-opacity="0.25" />
      <rect x="18" y="16" width="28" height="24" rx="4" fill="#fdf6ec" />
      <circle cx="26" cy="24" r="3.2" fill="#e08a3c" />
      <path d="M19 39 L29 30 L34 34 L40 28 L45 34 L45 38 a2 2 0 0 1 -2 2 H21 a2 2 0 0 1 -2 -2 Z" fill="#c9772f" />
      <defs>
        <linearGradient id="tbg" x1="0" y1="0" x2="0" y2="64" gradientUnits="userSpaceOnUse">
          <stop stop-color="#ffffff" />
          <stop offset="1" stop-color="#ffffff" stop-opacity="0" />
        </linearGradient>
      </defs>
    </svg>
    Photo Archiver
  </div>

  <!-- VS Code-style window menu -->
  <nav class="relative flex h-full items-center text-xs text-ink-dim">
    <button
      type="button"
      class={`flex h-6 items-center rounded-[5px] px-2 transition-colors hover:bg-surface-2 hover:text-ink ${openMenu === 'view' ? 'bg-surface-2 text-ink' : ''}`}
      onclick={() => toggle('view')}
    >
      View
    </button>
    <button
      type="button"
      class="flex h-6 items-center gap-1.5 rounded-[5px] px-2 transition-colors hover:bg-surface-2 hover:text-ink"
      onclick={() => open('settings')}
    >
      <Icon name="settings" size={13} />
      Settings
    </button>

    {#if openMenu === 'view'}
      <!-- click-away backdrop -->
      <button
        type="button"
        class="fixed inset-0 z-40 cursor-default"
        aria-label="Close menu"
        onclick={() => (openMenu = null)}
      ></button>
      <div
        class="absolute top-[calc(100%+2px)] left-0 z-50 min-w-44 rounded-md border border-line bg-surface py-1 shadow-[var(--shadow-lg)]"
      >
        {#each VIEW_ITEMS as item (item.id)}
          <button
            type="button"
            class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-ink-dim transition-colors hover:bg-surface-2 hover:text-ink"
            onclick={() => open(item.id)}
          >
            <Icon name={item.icon} size={14} class="text-ink-faint" />
            {item.label}
          </button>
        {/each}
      </div>
    {/if}
  </nav>

  <div class="ml-auto flex h-full">
    <button
      class="grid h-full w-11 place-items-center text-ink-dim hover:bg-surface-2 hover:text-ink"
      title="Minimize"
      onclick={() => appWindow.minimize()}
      aria-label="Minimize"
    >
      <svg width="10" height="10" viewBox="0 0 10 10"><rect y="4.5" width="10" height="1" fill="currentColor" /></svg>
    </button>
    <button
      class="grid h-full w-11 place-items-center text-ink-dim hover:bg-surface-2 hover:text-ink"
      title="Maximize"
      onclick={() => appWindow.toggleMaximize()}
      aria-label="Maximize"
    >
      {#if maximized}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"><rect x="0.5" y="2.5" width="6" height="6" /><path d="M3 2.5V0.5H9.5V7" /></svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"><rect x="0.5" y="0.5" width="9" height="9" /></svg>
      {/if}
    </button>
    <button
      class="grid h-full w-11 place-items-center text-ink-dim hover:bg-red-600 hover:text-white"
      title="Close"
      onclick={() => appWindow.close()}
      aria-label="Close"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" stroke="currentColor"><path d="M0.5 0.5l9 9M9.5 0.5l-9 9" /></svg>
    </button>
  </div>
</div>

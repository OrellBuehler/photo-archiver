<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { onMount } from 'svelte'

  const appWindow = getCurrentWindow()
  let maximized = $state(false)

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
  class="flex h-8 shrink-0 items-center border-b border-line bg-bar select-none"
  data-tauri-drag-region
>
  <div class="flex items-center gap-2 px-3 text-xs font-medium text-ink-dim" data-tauri-drag-region>
    <span class="h-3 w-3 rounded-sm bg-brand"></span>
    Photo Archiver
  </div>

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

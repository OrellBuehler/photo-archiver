<script lang="ts">
  import { store } from '../store.svelte'
  import ImageCard from './ImageCard.svelte'

  function folderName(p: string) {
    return p.replace(/[/\\]+$/, '').split(/[/\\]/).pop() || p
  }
</script>

<div class="flex h-full flex-col bg-base">
  {#if !store.settings?.source_dir}
    <div class="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
      <p class="text-ink-dim">No source folder selected.</p>
      <button class="btn" onclick={() => store.pickFolder()}>Choose source folder…</button>
    </div>
  {:else}
    <div class="flex items-center gap-2 border-b border-line px-3 py-2 text-xs">
      <span class="truncate text-ink-dim" title={store.settings.source_dir}>
        📁 {folderName(store.settings.source_dir)}
      </span>
      <button class="btn-sm" onclick={() => store.pickFolder()}>Change</button>
      <button class="btn-sm" onclick={() => store.scan()} disabled={store.scanning}>
        {store.scanning ? 'Scanning…' : 'Rescan'}
      </button>
      <span class="ml-auto text-ink-dim">
        {store.total} photos{store.selected.size ? ` · ${store.selected.size} selected` : ''}
      </span>
    </div>

    <div class="min-h-0 flex-1 overflow-auto p-3">
      {#if store.images.length === 0}
        <div class="flex h-full items-center justify-center text-ink-dim">
          {store.loading ? 'Loading…' : 'No photos found. Try Rescan.'}
        </div>
      {:else}
        <div class="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-2">
          {#each store.images as image (image.id)}
            <ImageCard {image} />
          {/each}
        </div>
      {/if}
    </div>

    {#if store.pages > 1}
      <div class="flex items-center justify-center gap-3 border-t border-line px-3 py-2 text-xs">
        <button class="btn-sm" disabled={store.page <= 1} onclick={() => store.goToPage(store.page - 1)}>
          ‹ Prev
        </button>
        <span class="text-ink-dim">Page {store.page} / {store.pages}</span>
        <button
          class="btn-sm"
          disabled={store.page >= store.pages}
          onclick={() => store.goToPage(store.page + 1)}
        >
          Next ›
        </button>
      </div>
    {/if}
  {/if}
</div>

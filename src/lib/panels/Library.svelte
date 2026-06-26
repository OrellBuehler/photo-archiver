<script lang="ts">
  import { store } from '../store.svelte'
  import { MONTHS } from '../types'
  import ImageCard from './ImageCard.svelte'
  import Icon from '../ui/Icon.svelte'

  function folderName(p: string) {
    return p.replace(/[/\\]+$/, '').split(/[/\\]/).pop() || p
  }

  // Batch-edit / move panels for the current selection.
  let showEdit = $state(false)
  let showFolder = $state(false)
  let edit = $state({ year: '', month: '', title: '' })
  let albumName = $state('')

  function closeBars() {
    showEdit = false
    showFolder = false
  }

  async function applyEdit() {
    const y = edit.year.trim() === '' ? null : Number(edit.year)
    const m = edit.month.trim() === '' ? null : Number(edit.month)
    const t = edit.title.trim() === '' ? null : edit.title.trim()
    if (y === null && m === null && t === null) return
    await store.editSelected(y, m, t)
    edit = { year: '', month: '', title: '' }
    showEdit = false
  }

  async function applyFolder(clear = false) {
    const name = albumName.trim()
    if (!clear && name === '') return
    await store.setFolderSelected(clear ? null : name)
    albumName = ''
    showFolder = false
  }
</script>

<div class="flex h-full flex-col bg-base">
  {#if store.error}
    <div
      class="flex items-center gap-2 border-b border-danger/40 px-3 py-2 text-xs text-danger"
      style="background: var(--color-danger-soft);"
    >
      <Icon name="alertTriangle" size={14} />
      {store.error}
    </div>
  {/if}

  {#if !store.settings?.source_dir}
    <div class="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
      <div
        class="grid h-16 w-16 place-items-center rounded-[12px] text-brand"
        style="background: var(--color-brand-soft);"
      >
        <Icon name="folderOpen" size={30} stroke={1.6} />
      </div>
      <div class="flex flex-col gap-1.5">
        <h2 class="font-serif text-2xl text-ink">Start with a folder of scans</h2>
        <p class="max-w-xs text-ink-dim">
          Point us at a folder of scanned photos to begin restoring and filing them.
        </p>
      </div>
      <button class="btn" onclick={() => store.pickFolder()}>
        <Icon name="folder" size={15} />
        Choose source folder…
      </button>
      {#if store.scanning}<p class="text-xs text-ink-faint">Scanning…</p>{/if}
    </div>
  {:else}
    <div class="flex flex-wrap items-center gap-2 border-b border-line px-3 py-2 text-xs">
      <span class="flex min-w-0 items-center gap-1.5 text-ink-dim" title={store.settings.source_dir}>
        <Icon name="folderOpen" size={14} class="text-brand" />
        <span class="truncate">{folderName(store.settings.source_dir)}</span>
      </span>
      <button class="btn-sm" onclick={() => store.pickFolder()}>Change…</button>
      <button class="btn-sm" onclick={() => store.scan()} disabled={store.scanning}>
        <Icon name="refresh" size={13} class={store.scanning ? 'animate-spin' : ''} />
        {store.scanning ? 'Scanning…' : 'Rescan'}
      </button>
      {#if store.selected.size}
        <button class="btn-sm" onclick={() => { store.clearSelection(); closeBars() }}>Clear</button>
        <button class="btn-sm" class:btn-sm-active={showEdit} onclick={() => { showEdit = !showEdit; showFolder = false }}>
          <Icon name="edit" size={13} /> Edit
        </button>
        <button class="btn-sm" title="Rotate left" aria-label="Rotate selected left" onclick={() => store.rotateSelected(false)}>
          <Icon name="rotateCcw" size={13} />
        </button>
        <button class="btn-sm" title="Rotate right" aria-label="Rotate selected right" onclick={() => store.rotateSelected(true)}>
          <Icon name="rotateCw" size={13} />
        </button>
        <button class="btn-sm" class:btn-sm-active={showFolder} onclick={() => { showFolder = !showFolder; showEdit = false }}>
          <Icon name="folder" size={13} /> Folder
        </button>
        <button
          class="btn-danger"
          onclick={async () => {
            if (confirm(`Delete ${store.selected.size} photo(s)? This removes processed copies.`))
              await store.deleteSelected()
          }}
        >
          <Icon name="trash" size={13} />
          Delete {store.selected.size}
        </button>
      {:else}
        <button class="btn-sm" onclick={() => store.selectAllOnPage()}>Select page</button>
      {/if}
      <span class="ml-auto whitespace-nowrap text-ink-dim">
        <b class="font-medium text-ink">{store.total}</b> photos{store.selected.size
          ? ` · ${store.selected.size} selected`
          : ''}
      </span>
    </div>

    {#if store.selected.size && showEdit}
      <div class="flex flex-wrap items-end gap-2 border-b border-line bg-surface px-3 py-2 text-xs">
        <label class="flex flex-col gap-1">
          <span class="eyebrow">Year</span>
          <input class="input w-20" bind:value={edit.year} placeholder="????" />
        </label>
        <label class="flex flex-col gap-1">
          <span class="eyebrow">Month</span>
          <select class="input w-32" bind:value={edit.month}>
            <option value="">—</option>
            {#each MONTHS.slice(1) as name, i (i)}
              <option value={String(i + 1)}>{name}</option>
            {/each}
          </select>
        </label>
        <label class="flex flex-1 flex-col gap-1">
          <span class="eyebrow">Title</span>
          <input class="input" bind:value={edit.title} placeholder="Leave blank to keep" />
        </label>
        <button class="btn" onclick={applyEdit}>
          <Icon name="check" size={14} /> Apply to {store.selected.size}
        </button>
        <span class="w-full text-[11px] text-ink-faint">Only the fields you fill are changed.</span>
      </div>
    {/if}

    {#if store.selected.size && showFolder}
      <div class="flex flex-wrap items-center gap-2 border-b border-line bg-surface px-3 py-2 text-xs">
        <input class="input w-48" bind:value={albumName} placeholder="Folder name (e.g. Wedding)" list="album-suggestions" />
        <datalist id="album-suggestions">
          {#each store.counts.folders as fl (fl.value)}<option value={fl.value}></option>{/each}
        </datalist>
        <button class="btn" onclick={() => applyFolder(false)}>
          <Icon name="folder" size={14} /> Move {store.selected.size}
        </button>
        <button class="btn-sm" onclick={() => applyFolder(true)}>Ungroup</button>
        <span class="w-full text-[11px] text-ink-faint">Applied when these photos are next organized.</span>
      </div>
    {/if}

    <div class="min-h-0 flex-1 overflow-auto p-3.5">
      {#if store.images.length === 0}
        <div class="flex h-full items-center justify-center text-center">
          {#if store.loading}
            <p class="text-ink-dim">Loading…</p>
          {:else}
            <div class="flex flex-col gap-1">
              <p class="font-serif text-xl text-ink">Nothing here yet</p>
              <p class="text-sm text-ink-dim">No photos match. Try a rescan or clearing filters.</p>
            </div>
          {/if}
        </div>
      {:else}
        <div class="grid grid-cols-[repeat(auto-fill,minmax(132px,1fr))] gap-2.5">
          {#each store.images as image (image.id)}
            <ImageCard {image} />
          {/each}
        </div>
      {/if}
    </div>

    {#if store.pages > 1}
      <div class="flex items-center justify-center gap-3 border-t border-line px-3 py-2 text-xs">
        <button class="btn-sm" disabled={store.page <= 1} onclick={() => store.goToPage(store.page - 1)}>
          <Icon name="chevronLeft" size={13} /> Prev
        </button>
        <span class="text-ink-dim tabular-nums">Page {store.page} / {store.pages}</span>
        <button
          class="btn-sm"
          disabled={store.page >= store.pages}
          onclick={() => store.goToPage(store.page + 1)}
        >
          Next <Icon name="chevronRight" size={13} />
        </button>
      </div>
    {/if}
  {/if}
</div>

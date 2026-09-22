<script lang="ts">
  import { store } from '../store.svelte'
  import { MONTHS, type Image } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Thumb from '../ui/Thumb.svelte'
  import Spinner from '../ui/Spinner.svelte'

  let scroller = $state<HTMLDivElement>()
  let sentinel = $state<HTMLDivElement>()

  const STATUS_COLOR: Record<string, string> = {
    new: 'var(--status-new)',
    organized: 'var(--status-organized)',
    enhanced: 'var(--status-enhanced)',
    processing: 'var(--status-processing)',
    failed: 'var(--status-failed)',
  }
  const STATUS_LABEL: Record<string, string> = {
    new: 'To restore',
    organized: 'Organized',
    enhanced: 'Restored',
    failed: 'Failed',
  }

  // Group by year; within a single year, by month.
  function groupLabel(img: Image) {
    if (img.year == null) return 'Undated'
    if (store.filters.year == null) return String(img.year)
    return img.month ? `${MONTHS[img.month]} ${img.year}` : `${img.year} · month unknown`
  }

  // The list is sorted year/month descending server-side, so grouping is a
  // single pass over consecutive runs.
  const groups = $derived.by(() => {
    const out: { label: string; items: Image[] }[] = []
    for (const img of store.images) {
      const label = groupLabel(img)
      const last = out[out.length - 1]
      if (last && last.label === label) last.items.push(img)
      else out.push({ label, items: [img] })
    }
    return out
  })

  const f = $derived(store.filters)
  const chips = $derived.by(() => {
    const c: { label: string; clear: () => void }[] = []
    if (f.status) c.push({ label: STATUS_LABEL[f.status] ?? f.status, clear: () => store.setFilter({ status: null }) })
    if (f.folder) c.push({ label: f.folder, clear: () => store.setFilter({ folder: null }) })
    if (f.year_unknown) c.push({ label: 'Undated', clear: () => store.setFilter({ year_unknown: false }) })
    if (f.year != null) c.push({ label: String(f.year), clear: () => store.setFilter({ year: null, month: null }) })
    if (f.month != null) c.push({ label: MONTHS[f.month], clear: () => store.setFilter({ month: null }) })
    if (f.search) c.push({ label: `“${f.search}”`, clear: () => store.setSearch('') })
    return c
  })

  function onClick(e: MouseEvent, id: number) {
    store.click(id, e.shiftKey ? 'range' : e.ctrlKey || e.metaKey ? 'toggle' : 'single')
  }

  $effect(() => {
    if (!sentinel || !scroller) return
    const io = new IntersectionObserver((entries) => {
      if (entries.some((e) => e.isIntersecting)) store.loadMore()
    }, { root: scroller, rootMargin: '600px' })
    io.observe(sentinel)
    return () => io.disconnect()
  })

  // Keep the keyboard cursor visible.
  $effect(() => {
    const id = store.focusedId
    if (id == null || !scroller) return
    scroller.querySelector(`[data-id="${id}"]`)?.scrollIntoView({ block: 'nearest' })
  })

  // Arrow-key navigation, resolved geometrically so it works across the
  // month headers that break up the grid's rows.
  function neighbour(dir: 'up' | 'down'): number | null {
    if (!scroller) return null
    const cur = scroller.querySelector<HTMLElement>(`[data-id="${store.focusedId}"]`)
    if (!cur) return null
    const r = cur.getBoundingClientRect()
    const cx = r.left + r.width / 2
    const cells = [...scroller.querySelectorAll<HTMLElement>('[data-id]')]
    let bestRow = dir === 'down' ? Infinity : -Infinity
    let best: HTMLElement | null = null
    let bestDx = Infinity
    for (const el of cells) {
      const b = el.getBoundingClientRect()
      const inDir = dir === 'down' ? b.top > r.bottom - 2 : b.bottom < r.top + 2
      if (!inDir) continue
      const row = b.top
      const closerRow = dir === 'down' ? row < bestRow - 1 : row > bestRow + 1
      const sameRow = Math.abs(row - bestRow) <= 1
      const dx = Math.abs(b.left + b.width / 2 - cx)
      if (closerRow || (sameRow && dx < bestDx)) {
        bestRow = row
        best = el
        bestDx = dx
      }
    }
    return best ? Number(best.dataset.id) : null
  }

  $effect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (store.view !== 'grid' || e.defaultPrevented) return
      const t = e.target as HTMLElement
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable)) return
      if (store.settingsOpen || store.shortcutsOpen || store.confirmRequest) return

      const extend = e.shiftKey
      if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
        e.preventDefault()
        if (store.focusedId == null) return store.images[0] && store.click(store.images[0].id)
        const idx = store.images.findIndex((i) => i.id === store.focusedId)
        const next = store.images[idx + (e.key === 'ArrowRight' ? 1 : -1)]
        if (next) store.focus(next.id, extend)
      } else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        e.preventDefault()
        if (store.focusedId == null) return store.images[0] && store.click(store.images[0].id)
        const id = neighbour(e.key === 'ArrowDown' ? 'down' : 'up')
        if (id != null) store.focus(id, extend)
        else if (e.key === 'ArrowDown') store.loadMore()
      } else if (e.key === 'Enter' && store.focusedId != null) {
        e.preventDefault()
        store.openLoupe()
      } else if (e.key === ' ' && store.focusedId != null) {
        e.preventDefault()
        store.click(store.focusedId, 'toggle')
      } else if (e.key === 'Home' && store.images.length) {
        e.preventDefault()
        store.click(store.images[0].id)
      } else if (e.key === 'End' && store.images.length) {
        e.preventDefault()
        store.click(store.images[store.images.length - 1].id)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  })

  function onWheel(e: WheelEvent) {
    if (!e.ctrlKey) return
    e.preventDefault()
    store.setGridSize(store.gridSize * (e.deltaY < 0 ? 1.1 : 1 / 1.1))
  }
</script>

<div class="flex h-full min-w-0 flex-col">
  <!-- Toolbar -->
  <div class="flex h-11 shrink-0 items-center gap-2 border-b border-line px-3">
    {#if chips.length}
      <div class="flex min-w-0 flex-wrap items-center gap-1.5">
        {#each chips as c (c.label)}
          <span class="chip">
            {c.label}
            <button class="grid h-4 w-4 place-items-center rounded-full hover:bg-brand/20" aria-label="Remove filter" onclick={c.clear}>
              <Icon name="x" size={10} stroke={2.2} />
            </button>
          </span>
        {/each}
        {#if chips.length > 1}
          <button class="text-xs text-ink-faint hover:text-ink" onclick={() => store.clearFilters()}>Clear all</button>
        {/if}
      </div>
    {:else}
      <h1 class="text-[13px] font-semibold text-ink">All photos</h1>
    {/if}
    <span class="text-xs text-ink-faint tabular-nums">{store.total}</span>

    <div class="ml-auto flex items-center gap-1">
      {#if store.selected.size > 0}
        <span class="mr-1 text-xs text-focus tabular-nums">{store.selected.size} selected</span>
        <button class="icon-btn" title="Rotate left ([)" aria-label="Rotate left" onclick={() => store.rotateTargets(false)}>
          <Icon name="rotateCcw" size={15} />
        </button>
        <button class="icon-btn" title="Rotate right (])" aria-label="Rotate right" onclick={() => store.rotateTargets(true)}>
          <Icon name="rotateCw" size={15} />
        </button>
        <button class="icon-btn" title="Delete (Del)" aria-label="Delete" onclick={() => store.deleteTargets()}>
          <Icon name="trash" size={15} />
        </button>
        <button class="btn-sm ml-1" onclick={() => store.openInspector('restore')}>
          <Icon name="wand" size={13} />Restore…
        </button>
        <button class="icon-btn" title="Clear selection (Esc)" aria-label="Clear selection" onclick={() => store.clearSelection()}>
          <Icon name="x" size={15} />
        </button>
        <div class="mx-1.5 h-4 w-px bg-line"></div>
      {:else if store.images.length}
        <button class="btn-sm mr-1" title="Select all (Ctrl+A)" onclick={() => store.selectAll()}>Select all</button>
      {/if}
      <Icon name="grid" size={13} class="text-ink-faint" />
      <input
        type="range"
        min="90"
        max="320"
        value={store.gridSize}
        oninput={(e) => store.setGridSize(Number(e.currentTarget.value))}
        class="w-24 accent-[var(--color-brand)]"
        aria-label="Thumbnail size"
        title="Thumbnail size (Ctrl+scroll)"
      />
    </div>
  </div>

  <!-- Grid -->
  <div bind:this={scroller} class="min-h-0 flex-1 overflow-y-auto" onwheel={onWheel}>
    {#if store.images.length === 0}
      <div class="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
        {#if store.loading}
          <Spinner size={18} />
        {:else}
          <div class="mb-1 grid h-14 w-14 place-items-center rounded-[12px] bg-surface-2 text-ink-faint">
            <Icon name={store.hasFilters ? 'search' : 'image'} size={26} stroke={1.5} />
          </div>
          <p class="font-serif text-xl text-ink">{store.hasFilters ? 'No matches' : 'No photos yet'}</p>
          <p class="max-w-xs text-ink-dim">
            {store.hasFilters ? 'Nothing matches these filters.' : 'This folder has no supported images. Try a rescan.'}
          </p>
          {#if store.hasFilters}
            <button class="btn-sm mt-2" onclick={() => store.clearFilters()}>Clear filters</button>
          {:else}
            <button class="btn-sm mt-2" onclick={() => store.scan()}>
              <Icon name="refresh" size={13} />Rescan
            </button>
          {/if}
        {/if}
      </div>
    {:else}
      <div class="px-4 pb-6">
        {#each groups as g, gi (g.label + gi)}
          <div
            class="sticky top-0 z-10 -mx-4 flex items-baseline gap-2 px-4 pt-4 pb-2"
            style="background: linear-gradient(var(--color-base) 70%, transparent);"
          >
            <h2 class="text-[13px] font-semibold text-ink">{g.label}</h2>
            <span class="text-[11px] text-ink-faint tabular-nums">{g.items.length}</span>
          </div>
          <div class="grid gap-2" style="grid-template-columns: repeat(auto-fill, minmax({store.gridSize}px, 1fr));">
            {#each g.items as image (image.id)}
              {@const selected = store.selected.has(image.id)}
              {@const focused = store.focusedId === image.id}
              {@const label = image.title ?? image.filename}
              <button
                type="button"
                data-id={image.id}
                tabindex="-1"
                onclick={(e) => onClick(e, image.id)}
                ondblclick={() => store.openLoupe(image.id)}
                class="group relative aspect-square scroll-mt-12 scroll-mb-2 overflow-hidden rounded-[6px] bg-surface-2 outline-none"
                title={label}
              >
                <Thumb id={image.id} alt={label} />

                <span
                  class="pointer-events-none absolute inset-0 rounded-[6px] transition-shadow duration-[110ms]"
                  style="box-shadow: {selected
                    ? 'inset 0 0 0 2px var(--color-focus)'
                    : 'inset 0 0 0 1px rgba(255,255,255,0.06)'}{focused ? ', 0 0 0 2px var(--color-base), 0 0 0 4px var(--color-focus)' : ''};"
                ></span>
                {#if selected}
                  <span class="pointer-events-none absolute inset-0 rounded-[6px] bg-focus/10"></span>
                {/if}

                {#if store.processingIds.has(image.id)}
                  <div class="absolute inset-0 grid place-items-center" style="background: var(--bg-overlay);">
                    <Spinner size={20} />
                  </div>
                {/if}

                <span
                  role="checkbox"
                  aria-checked={selected}
                  tabindex="-1"
                  class="absolute top-1.5 left-1.5 grid h-[18px] w-[18px] place-items-center rounded-full border-[1.5px] text-white transition-opacity duration-[110ms] {selected
                    ? 'opacity-100'
                    : 'opacity-0 group-hover:opacity-100'}"
                  style={selected
                    ? 'border-color: var(--color-focus); background: var(--color-focus);'
                    : 'border-color: rgba(255,255,255,0.7); background: rgba(12,10,8,0.35);'}
                  onclick={(e) => {
                    e.stopPropagation()
                    store.click(image.id, e.shiftKey ? 'range' : 'toggle')
                  }}
                  onkeydown={() => {}}
                >
                  {#if selected}
                    <Icon name="check" size={11} stroke={3} />
                  {/if}
                </span>

                {#if image.status !== 'new'}
                  <span
                    class="absolute top-2 right-2 h-2 w-2 rounded-full"
                    style="background: {STATUS_COLOR[image.status] ?? 'var(--status-new)'}; box-shadow: 0 0 0 2px rgba(12,10,8,0.55);"
                    title={STATUS_LABEL[image.status] ?? image.status}
                  ></span>
                {/if}

                <span
                  class="absolute inset-x-0 bottom-0 truncate px-2 pt-5 pb-1.5 text-left text-[11px] text-ink opacity-0 transition-opacity group-hover:opacity-100"
                  style="background: linear-gradient(to top, rgba(12,10,8,0.85), transparent);"
                >
                  {label}
                </span>
              </button>
            {/each}
          </div>
        {/each}
        <div bind:this={sentinel} class="flex h-12 items-center justify-center">
          {#if store.loadingMore}<Spinner size={14} />{/if}
        </div>
      </div>
    {/if}
  </div>
</div>

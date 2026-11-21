<script lang="ts">
  import { store, type MenuItem } from '../store.svelte'
  import { STATUS, dateLabel, plural, type Image } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Thumb from '../ui/Thumb.svelte'
  import Spinner from '../ui/Spinner.svelte'

  let scroller = $state<HTMLDivElement>()
  let sentinel = $state<HTMLDivElement>()

  const count = $derived(store.selected.size)

  function onClick(e: MouseEvent, id: number) {
    store.click(id, e.shiftKey ? 'range' : e.ctrlKey || e.metaKey ? 'toggle' : 'single')
  }

  function onMenu(e: MouseEvent, image: Image) {
    e.preventDefault()
    if (!store.selected.has(image.id)) store.click(image.id)
    const n = store.selected.size
    const last = store.lastMeta
    const items: MenuItem[] = []
    if (n === 1) items.push({ label: 'Open', icon: 'maximize', hint: 'Enter', run: () => store.openLoupe(image.id) })
    items.push({ label: n > 1 ? `Restore ${n} photos…` : 'Restore…', icon: 'wand', hint: 'R', run: () => store.openInspector('restore') })
    if (n === 1) items.push({ label: 'Edit date', icon: 'calendar', hint: 'D', run: () => store.editDate() })
    if (last?.year != null || last?.folder) {
      const what = [last.year != null ? dateLabel(last) : null, last.folder].filter(Boolean).join(' · ')
      items.push({ label: `Same as last: ${what}`, icon: 'repeat', hint: '.', run: () => store.applyLast() })
    }
    items.push(
      { label: 'Rotate left', icon: 'rotateCcw', hint: '[', separator: true, run: () => store.rotateTargets(false) },
      { label: 'Rotate right', icon: 'rotateCw', hint: ']', run: () => store.rotateTargets(true) },
    )
    if (n === 1) items.push({ label: 'Show in folder', icon: 'external', separator: true, run: () => store.reveal(image.id) })
    if (n > 1 || image.organized_path) {
      items.push({ label: 'Start over from the scan', icon: 'history', separator: n > 1, run: () => store.resetTargets() })
    }
    items.push({ label: 'Remove from library', icon: 'trash', hint: 'Del', danger: true, separator: true, run: () => store.deleteTargets() })
    store.menu = { x: e.clientX, y: e.clientY, items }
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

  // Back at the top whenever the place changes.
  $effect(() => {
    void store.filters
    scroller?.scrollTo({ top: 0 })
  })

  // Up/down resolved geometrically, so it holds for any column count.
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
      if (store.settingsOpen || store.shortcutsOpen || store.confirmRequest || store.menu) return

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

  const empty = $derived.by(() => {
    const f = store.filters
    if (f.search) return { icon: 'search', title: 'No matches', text: `Nothing in ${store.placeLabel} matches “${f.search}”.` }
    if (f.status === 'new') return { icon: 'checkCircle', title: 'Inbox zero', text: 'Every scan has a copy in the archive. Rescan when you have scanned more.' }
    if (f.year_unknown) return { icon: 'checkCircle', title: 'Everything is dated', text: 'No photo is waiting for a year.' }
    if (store.hasPlace) return { icon: 'image', title: 'Nothing here', text: `No photos in ${store.placeLabel}.` }
    return { icon: 'image', title: 'No photos yet', text: 'This folder has no supported images. Add scans and rescan.' }
  })
</script>

<div class="@container flex h-full min-w-0 flex-col">
  <div class="flex h-11 shrink-0 items-center gap-2 border-b border-line pr-2 pl-4">
    <h1 class="truncate text-[13px] font-semibold text-ink">{store.placeLabel}</h1>
    <span class="text-xs text-ink-faint tabular-nums">{store.total}</span>
    {#if store.filters.search}
      <span class="chip ml-1 min-w-0">
        <span class="truncate">“{store.filters.search}”</span>
        <button class="grid h-4 w-4 shrink-0 place-items-center rounded-full hover:bg-brand/20" aria-label="Clear search" onclick={() => store.setSearch('')}>
          <Icon name="x" size={10} stroke={2.2} />
        </button>
      </span>
    {/if}

    <div class="ml-auto flex shrink-0 items-center gap-1">
      {#if count > 0}
        <span class="mr-1 text-xs whitespace-nowrap text-focus tabular-nums @max-xl:hidden">{count} selected</span>
        <button class="icon-btn" title="Rotate left ([)" aria-label="Rotate left" onclick={() => store.rotateTargets(false)}>
          <Icon name="rotateCcw" size={15} />
        </button>
        <button class="icon-btn" title="Rotate right (])" aria-label="Rotate right" onclick={() => store.rotateTargets(true)}>
          <Icon name="rotateCw" size={15} />
        </button>
        <button class="icon-btn" title="Remove from library (Del)" aria-label="Remove" onclick={() => store.deleteTargets()}>
          <Icon name="trash" size={15} />
        </button>
        <button class="icon-btn" title="Clear selection (Esc)" aria-label="Clear selection" onclick={() => store.clearSelection()}>
          <Icon name="x" size={15} />
        </button>
      {:else if store.total > 0}
        <button class="btn-sm border-transparent bg-transparent @max-md:hidden" title="Select all (Ctrl+A)" onclick={() => store.selectAll()}>Select all</button>
      {/if}
      <div class="mx-1.5 h-4 w-px bg-line @max-xl:hidden"></div>
      <button
        class="icon-btn @max-xl:hidden"
        class:icon-btn-active={store.gridFit}
        title={store.gridFit ? 'Showing whole scans — click to fill the tiles' : 'Tiles are filled — click to show whole scans'}
        aria-label="Show whole scans"
        aria-pressed={store.gridFit}
        onclick={() => store.toggleGridFit()}
      >
        <Icon name="fit" size={15} />
      </button>
      <input
        type="range"
        min="90"
        max="320"
        value={store.gridSize}
        oninput={(e) => store.setGridSize(Number(e.currentTarget.value))}
        class="w-20 accent-[var(--color-brand)] @max-xl:hidden"
        aria-label="Thumbnail size"
        title="Thumbnail size (Ctrl+scroll)"
      />
      {#if store.total > 0}
        <button class="btn ml-2 h-7 shadow-none" title="Choose what to do (R)" onclick={() => store.openInspector('restore')}>
          {#if store.busy}
            <Spinner size={12} color="var(--color-on-brand)" />Restoring…
          {:else}
            <Icon name="wand" size={14} />{count > 0 ? `Restore ${count}` : 'Restore all'}
          {/if}
        </button>
      {/if}
    </div>
  </div>

  <div bind:this={scroller} class="min-h-0 flex-1 overflow-y-auto" onwheel={onWheel}>
    {#if store.images.length === 0}
      <div class="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
        {#if store.loading}
          <Spinner size={18} />
        {:else}
          <div class="mb-1 grid h-14 w-14 place-items-center rounded-[12px] bg-surface-2 text-ink-faint">
            <Icon name={empty.icon} size={26} stroke={1.5} />
          </div>
          <p class="font-serif text-xl text-ink">{empty.title}</p>
          <p class="max-w-xs text-ink-dim">{empty.text}</p>
          <div class="mt-2 flex gap-2">
            {#if store.filters.search}
              <button class="btn-sm" onclick={() => store.setSearch('')}>Clear search</button>
            {/if}
            {#if store.hasPlace}
              <button class="btn-sm" onclick={() => store.show()}>Show all photos</button>
            {/if}
            {#if !store.hasPlace || store.filters.status === 'new'}
              <button class="btn-sm" onclick={() => store.scan()} disabled={store.scanning}>
                <Icon name="refresh" size={13} />Rescan
              </button>
            {/if}
          </div>
        {/if}
      </div>
    {:else}
      <div class="grid gap-x-2.5 gap-y-3 p-4" style="grid-template-columns: repeat(auto-fill, minmax({store.gridSize}px, 1fr));">
        {#each store.images as image (image.id)}
          {@const selected = store.selected.has(image.id)}
          {@const focused = store.focusedId === image.id}
          {@const date = dateLabel(image)}
          {@const st = STATUS[image.status]}
          <button
            type="button"
            data-id={image.id}
            tabindex="-1"
            onclick={(e) => onClick(e, image.id)}
            ondblclick={() => store.openLoupe(image.id)}
            oncontextmenu={(e) => onMenu(e, image)}
            class="group flex scroll-my-3 flex-col gap-1 text-left outline-none"
            title={image.title ?? image.filename}
          >
            <span
              class="relative block aspect-square w-full overflow-hidden rounded-[6px] transition-colors {store.gridFit
                ? 'p-1 group-hover:bg-surface'
                : 'bg-surface-2'}"
            >
              <Thumb id={image.id} alt={image.title ?? image.filename} fit={store.gridFit ? 'contain' : 'cover'} />

              <span
                class="pointer-events-none absolute inset-0 rounded-[6px] transition-shadow duration-[110ms] {selected ? 'bg-focus/10' : ''}"
                style="box-shadow: {selected
                  ? 'inset 0 0 0 2px var(--color-focus)'
                  : focused
                    ? 'inset 0 0 0 2px var(--border-strong)'
                    : store.gridFit
                      ? 'none'
                      : 'inset 0 0 0 1px rgba(255,255,255,0.05)'};"
              ></span>

              {#if store.processingIds.has(image.id)}
                <span class="absolute inset-0 grid place-items-center" style="background: var(--bg-overlay);">
                  <Spinner size={20} />
                </span>
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

            </span>

            <span class="flex w-full items-center gap-1.5 px-0.5 text-[11px] leading-4">
              {#if image.status === 'failed'}
                <span class="shrink-0 text-danger" title={image.error ?? st?.hint}><Icon name="alertTriangle" size={12} stroke={2.2} /></span>
              {:else if image.status !== 'new' && st}
                <span class="h-1.5 w-1.5 shrink-0 rounded-full" style="background: {st.color};" title="{st.label} — {st.hint}"></span>
              {/if}
              <span class="min-w-0 flex-1 truncate {image.title ? 'text-ink-dim' : 'text-ink-faint'}">
                {image.title ?? image.scan_id ?? image.filename}
              </span>
              <span class="shrink-0 tabular-nums {date ? 'text-ink-dim' : 'text-warning/80'}">{date ?? 'no date'}</span>
            </span>
          </button>
        {/each}
      </div>
      <div bind:this={sentinel} class="flex h-10 items-center justify-center">
        {#if store.loadingMore}<Spinner size={14} />{/if}
      </div>
    {/if}
  </div>

  {#if count > 1 && !store.inspectorOpen}
    <div class="flex h-9 shrink-0 items-center gap-2 border-t border-line bg-surface px-4 text-xs text-ink-dim">
      {plural(count, 'photo')} selected
      <button class="btn-sm ml-auto" onclick={() => store.openInspector('info')}>Edit together</button>
    </div>
  {/if}
</div>

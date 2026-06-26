<script lang="ts">
  import { store } from '../store.svelte'
  import {
    getImage,
    getVariant,
    imageHistory,
    redoImage,
    rotateImage,
    snapshotState,
    undoImage,
    updateImage,
  } from '../api'
  import { MONTHS, type HistoryItem, type Image, type SnapshotState } from '../types'
  import { invalidateThumb } from '../thumbs'
  import Icon from '../ui/Icon.svelte'
  import Badge from '../ui/Badge.svelte'

  type Variant = 'source' | 'organized' | 'enhanced'

  let image = $state<Image | null>(null)
  let history = $state<HistoryItem[]>([])
  let variant = $state<Variant>('source')
  let url = $state<string | null>(null)
  let compareUrl = $state<string | null>(null)
  let comparing = $state(false)
  let split = $state(50)
  let editing = $state(false)
  let form = $state({ year: '', month: '', title: '' })
  let snap = $state<SnapshotState>({ pos: 0, max: 0 })

  // Zoom / pan for the single-image view (disabled while comparing).
  let scale = $state(1)
  let tx = $state(0)
  let ty = $state(0)
  let panning = $state(false)
  let panStart = { x: 0, y: 0 }

  const canUndo = $derived(snap.pos > 0)
  const canRedo = $derived(snap.pos < snap.max)

  function resetZoom() {
    scale = 1
    tx = 0
    ty = 0
  }

  let stageEl = $state<HTMLDivElement>()
  let dragging = false

  $effect(() => {
    const id = store.focusedImageId
    void store.thumbVersion
    if (id == null) {
      image = null
      return
    }
    load(id)
  })

  const variants = $derived.by<Variant[]>(() => {
    if (!image) return []
    const v: Variant[] = ['source']
    if (image.organized_path) v.push('organized')
    if (image.enhanced_path) v.push('enhanced')
    return v
  })

  // Browse within the current library page.
  const idx = $derived(image ? store.images.findIndex((i) => i.id === image!.id) : -1)

  function go(d: number) {
    const imgs = store.images
    if (idx < 0 || imgs.length === 0) return
    const next = imgs[(idx + d + imgs.length) % imgs.length]
    if (next) store.openImage(next.id)
  }

  $effect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!image || editing) return
      // Only browse when the Viewer is the focused dockview panel.
      if (store.dockApi?.activeGroup?.activePanel?.id !== 'viewer') return
      const t = e.target as HTMLElement
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return
      if (e.key === 'ArrowRight') go(1)
      else if (e.key === 'ArrowLeft') go(-1)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  })

  function setFromClientX(clientX: number) {
    if (!stageEl) return
    const r = stageEl.getBoundingClientRect()
    split = Math.max(0, Math.min(100, ((clientX - r.left) / r.width) * 100))
  }

  $effect(() => {
    const move = (e: MouseEvent) => {
      if (dragging) setFromClientX(e.clientX)
      else if (panning) {
        tx = e.clientX - panStart.x
        ty = e.clientY - panStart.y
      }
    }
    const up = () => {
      dragging = false
      panning = false
    }
    window.addEventListener('mousemove', move)
    window.addEventListener('mouseup', up)
    return () => {
      window.removeEventListener('mousemove', move)
      window.removeEventListener('mouseup', up)
    }
  })

  function onWheel(e: WheelEvent) {
    if (comparing || !stageEl) return
    e.preventDefault()
    const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15
    const next = Math.min(8, Math.max(1, scale * factor))
    const r = stageEl.getBoundingClientRect()
    const cx = e.clientX - r.left - r.width / 2
    const cy = e.clientY - r.top - r.height / 2
    // Keep the point under the cursor anchored as we scale.
    tx = cx - (cx - tx) * (next / scale)
    ty = cy - (cy - ty) * (next / scale)
    scale = next
    if (scale === 1) {
      tx = 0
      ty = 0
    }
  }

  function zoomBy(factor: number) {
    scale = Math.min(8, Math.max(1, scale * factor))
    if (scale === 1) {
      tx = 0
      ty = 0
    }
  }

  async function load(id: number) {
    image = await getImage(id)
    history = await imageHistory(id)
    if (!image) return
    form = {
      year: image.year?.toString() ?? '',
      month: image.month?.toString() ?? '',
      title: image.title ?? '',
    }
    variant = image.enhanced_path ? 'enhanced' : image.organized_path ? 'organized' : 'source'
    comparing = false
    split = 50
    resetZoom()
    snap = await snapshotState(id)
    await loadVariant()
  }

  async function undo() {
    if (!image || !canUndo) return
    const id = image.id
    await undoImage(id)
    afterMutation(id)
  }

  async function redo() {
    if (!image || !canRedo) return
    const id = image.id
    await redoImage(id)
    afterMutation(id)
  }

  // After a mutation that rewrote the organized file, invalidate the cached
  // thumbnail and bump thumbVersion — the loader $effect (which depends on
  // thumbVersion) then performs the single reload of image/history/variant/snap.
  function afterMutation(id: number) {
    invalidateThumb(id)
    store.thumbVersion++
    store.refresh()
  }

  async function loadVariant() {
    if (!image) return
    if (url) URL.revokeObjectURL(url)
    url = await getVariant(image.id, variant)
  }

  async function pick(v: Variant) {
    variant = v
    await loadVariant()
  }

  async function toggleCompare() {
    comparing = !comparing
    if (comparing && image) {
      split = 50
      if (compareUrl) URL.revokeObjectURL(compareUrl)
      compareUrl = await getVariant(image.id, 'source')
    }
  }

  async function rotate(clockwise: boolean) {
    if (!image) return
    const id = image.id
    await rotateImage(id, clockwise)
    afterMutation(id)
  }

  async function save() {
    if (!image) return
    const y = form.year.trim() === '' ? null : Number(form.year)
    const m = form.month.trim() === '' ? null : Number(form.month)
    const t = form.title.trim() === '' ? null : form.title.trim()
    image = await updateImage(image.id, y, m, t)
    editing = false
    store.refresh()
  }
</script>

<div class="flex h-full flex-col bg-base">
  {#if !image}
    <div class="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center">
      <div
        class="grid h-14 w-14 place-items-center rounded-[12px] text-ink-dim"
        style="background: var(--color-surface-2);"
      >
        <Icon name="columns" size={26} stroke={1.5} />
      </div>
      <div class="flex flex-col gap-1">
        <p class="font-serif text-xl text-ink">Nothing open yet</p>
        <p class="text-sm text-ink-dim">Double-click a photo to compare its before & after here.</p>
      </div>
    </div>
  {:else}
    <div class="flex items-center gap-1 border-b border-line px-2 py-1.5 text-xs">
      {#each variants as v (v)}
        <button class="btn-sm" class:btn-sm-active={variant === v && !comparing} onclick={() => pick(v)}>
          {v}
        </button>
      {/each}
      <div class="ml-auto flex items-center gap-1">
        {#if variants.length > 1}
          <button class="btn-sm" class:btn-sm-active={comparing} onclick={toggleCompare}>
            <Icon name="columns" size={13} />
            Compare
          </button>
        {/if}
        {#if !comparing}
          <button class="btn-sm" title="Zoom out" aria-label="Zoom out" disabled={scale <= 1} onclick={() => zoomBy(1 / 1.3)}>
            <Icon name="zoomOut" size={14} />
          </button>
          <button class="btn-sm" title="Fit" aria-label="Fit to view" disabled={scale === 1} onclick={resetZoom}>
            <span class="tabular-nums">{Math.round(scale * 100)}%</span>
          </button>
          <button class="btn-sm" title="Zoom in" aria-label="Zoom in" disabled={scale >= 8} onclick={() => zoomBy(1.3)}>
            <Icon name="zoomIn" size={14} />
          </button>
        {/if}
        <button class="btn-sm" title="Undo" aria-label="Undo" disabled={!canUndo} onclick={undo}>
          <Icon name="undo" size={14} />
        </button>
        <button class="btn-sm" title="Redo" aria-label="Redo" disabled={!canRedo} onclick={redo}>
          <Icon name="redo" size={14} />
        </button>
        <button class="btn-sm" title="Rotate left" aria-label="Rotate left" onclick={() => rotate(false)}>
          <Icon name="rotateCcw" size={14} />
        </button>
        <button class="btn-sm" title="Rotate right" aria-label="Rotate right" onclick={() => rotate(true)}>
          <Icon name="rotateCw" size={14} />
        </button>
        {#if store.images.length > 1}
          <button class="btn-sm" title="Previous" aria-label="Previous" onclick={() => go(-1)}>
            <Icon name="chevronLeft" size={14} />
          </button>
          <span class="min-w-[44px] text-center tabular-nums text-ink-dim">{idx + 1} / {store.images.length}</span>
          <button class="btn-sm" title="Next" aria-label="Next" onclick={() => go(1)}>
            <Icon name="chevronRight" size={14} />
          </button>
        {/if}
      </div>
    </div>

    <div
      bind:this={stageEl}
      class="relative min-h-0 flex-1 overflow-hidden bg-black"
      class:cursor-ew-resize={comparing && compareUrl}
      class:cursor-grab={!comparing && scale > 1 && !panning}
      class:cursor-grabbing={!comparing && panning}
      onwheel={onWheel}
      onmousedown={(e) => {
        if (comparing && compareUrl) {
          dragging = true
          setFromClientX(e.clientX)
        } else if (scale > 1) {
          panning = true
          panStart = { x: e.clientX - tx, y: e.clientY - ty }
        }
      }}
      role="presentation"
    >
      {#if comparing && compareUrl && url}
        <!-- after (selected variant) — full -->
        <img src={url} alt={variant} draggable="false" class="absolute inset-0 h-full w-full object-contain" />
        <span
          class="eyebrow pointer-events-none absolute right-2.5 top-2.5 rounded-[3px] px-2 py-0.5 text-ink"
          style="background: var(--bg-overlay); backdrop-filter: blur(4px);"
        >{variant}</span>

        <!-- before (source) — clipped to left of divider -->
        <div class="absolute inset-0 overflow-hidden" style="clip-path: inset(0 {100 - split}% 0 0)">
          <img src={compareUrl} alt="source" draggable="false" class="absolute inset-0 h-full w-full object-contain" />
          <span
            class="eyebrow pointer-events-none absolute left-2.5 top-2.5 rounded-[3px] px-2 py-0.5 text-ink"
            style="background: var(--bg-overlay); backdrop-filter: blur(4px);"
          >original</span>
        </div>

        <!-- divider + amber handle -->
        <div
          class="pointer-events-none absolute bottom-0 top-0"
          style="left: {split}%; width: 2px; background: var(--color-brand); transform: translateX(-1px);"
        >
          <span
            class="absolute left-1/2 top-1/2 grid h-[34px] w-[34px] -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full shadow-[var(--shadow-md)]"
            style="background: var(--color-brand); color: var(--color-on-brand);"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 6 4 12l5 6M15 6l5 6-5 6" /></svg>
          </span>
        </div>
      {:else if url}
        <img
          src={url}
          alt={variant}
          draggable="false"
          class="absolute inset-0 h-full w-full select-none object-contain"
          style="transform: translate({tx}px, {ty}px) scale({scale}); transition: {panning ? 'none' : 'transform 120ms ease-out'};"
        />
      {/if}
    </div>

    <div class="max-h-72 shrink-0 overflow-auto border-t border-line bg-surface p-3.5 text-sm">
      <div class="mb-3 flex items-center gap-2">
        <h2 class="min-w-0 flex-1 truncate font-serif text-lg text-ink" title={image.filename}>
          {image.title ?? image.filename}
        </h2>
        <Badge status={image.status} />
        <button class="btn-sm" onclick={() => (editing = !editing)}>
          {#if editing}Cancel{:else}<Icon name="edit" size={13} />Edit{/if}
        </button>
      </div>

      {#if editing}
        <div class="flex flex-col gap-2.5">
          <div class="grid grid-cols-2 gap-2.5">
            <label class="flex flex-col gap-1">
              <span class="eyebrow">Year</span>
              <input class="input" bind:value={form.year} placeholder="????" />
            </label>
            <label class="flex flex-col gap-1">
              <span class="eyebrow">Month</span>
              <select class="input" bind:value={form.month}>
                <option value="">—</option>
                {#each MONTHS.slice(1) as name, i (i)}
                  <option value={String(i + 1)}>{name}</option>
                {/each}
              </select>
            </label>
          </div>
          <label class="flex flex-col gap-1">
            <span class="eyebrow">Title</span>
            <input class="input" bind:value={form.title} placeholder="Add a title…" />
          </label>
          <button class="btn mt-1" onclick={save}>
            <Icon name="check" size={15} />
            Save
          </button>
        </div>
      {:else}
        <div class="eyebrow mb-2">Details</div>
        <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs text-ink-dim">
          <dt>Filed</dt>
          <dd class="font-mono text-ink">
            {image.year ?? '????'}/{String(image.month ?? 0).padStart(2, '0')}/
            {#if image.month}<span class="text-ink-dim">· {MONTHS[image.month]}</span>{/if}
          </dd>
          <dt>Dimensions</dt>
          <dd class="text-ink">{image.width ?? '?'} × {image.height ?? '?'}</dd>
          <dt>Source</dt>
          <dd class="truncate font-mono text-ink" title={image.source_path}>{image.source_path}</dd>
        </dl>

        {#if history.length}
          <h3 class="eyebrow mb-1 mt-3">History</h3>
          <ul class="flex flex-col gap-0.5 text-xs text-ink-dim">
            {#each history as h (h.id)}
              <li>{h.step} <span class="opacity-60">· {h.created_at.replace('T', ' ').slice(0, 16)}</span></li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<script lang="ts">
  import { store } from '../store.svelte'
  import { getVariant, imageTimeline } from '../api'
  import { dateLabel } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Thumb from '../ui/Thumb.svelte'
  import Spinner from '../ui/Spinner.svelte'

  type Variant = 'source' | 'organized' | 'enhanced'

  const image = $derived(store.focused)
  let variant = $state<Variant>('source')
  let url = $state<string | null>(null)
  let beforeUrl = $state<string | null>(null)
  let loadingImg = $state(false)
  let comparing = $state(false)
  let split = $state(50)
  let snap = $state({ pos: 0, max: 0 })
  // Holding O shows the untouched scan for as long as the key is down.
  let peeking = $state(false)

  let scale = $state(1)
  let tx = $state(0)
  let ty = $state(0)
  let panning = $state(false)
  let dragging = false
  let panStart = { x: 0, y: 0 }
  let stageEl = $state<HTMLDivElement>()
  let stripEl = $state<HTMLDivElement>()

  const variants = $derived.by<Variant[]>(() => {
    if (!image) return []
    const v: Variant[] = ['source']
    if (image.organized_path) v.push('organized')
    if (image.enhanced_path) v.push('enhanced')
    return v
  })
  const idx = $derived(image ? store.images.findIndex((i) => i.id === image.id) : -1)
  const canCompare = $derived(variants.length > 1 && variant !== 'source')

  // Reload the preview whenever the focused photo or its files change.
  let loadedKey = ''
  $effect(() => {
    const img = image
    void store.thumbVersion
    if (!img) return
    const key = `${img.id}:${img.updated_at}:${store.thumbVersion}`
    if (key === loadedKey) return
    const sameImage = loadedKey.startsWith(`${img.id}:`)
    loadedKey = key
    if (!sameImage) {
      variant = img.enhanced_path ? 'enhanced' : img.organized_path ? 'organized' : 'source'
      comparing = false
      peeking = false
      split = 50
      resetZoom()
      if (beforeUrl) URL.revokeObjectURL(beforeUrl)
      beforeUrl = null
    } else if (!variants.includes(variant)) {
      variant = 'source'
    }
    void load(img.id)
  })

  async function load(id: number) {
    loadingImg = true
    try {
      const [next, t] = await Promise.all([getVariant(id, variant), imageTimeline(id)])
      if (store.focusedId !== id) return URL.revokeObjectURL(next)
      if (url) URL.revokeObjectURL(url)
      url = next
      snap = { pos: t.pos, max: t.steps.length }
      if (comparing) await loadBefore(id)
    } catch (e) {
      store.fail(e)
    } finally {
      loadingImg = false
    }
  }

  async function loadBefore(id: number) {
    const next = await getVariant(id, 'source')
    if (store.focusedId !== id) return URL.revokeObjectURL(next)
    if (beforeUrl) URL.revokeObjectURL(beforeUrl)
    beforeUrl = next
  }

  function peek(on: boolean) {
    if (!image || variant === 'source' || comparing) return (peeking = false)
    peeking = on
    if (on && !beforeUrl) loadBefore(image.id).catch((e) => store.fail(e))
  }

  async function pick(v: Variant) {
    if (!image || v === variant) return
    variant = v
    if (v === 'source') comparing = false
    await load(image.id)
  }

  async function toggleCompare() {
    if (!image || !canCompare) return
    comparing = !comparing
    if (comparing) {
      resetZoom()
      split = 50
      await loadBefore(image.id)
    }
  }

  function resetZoom() {
    scale = 1
    tx = 0
    ty = 0
  }

  function zoomBy(factor: number) {
    scale = Math.min(8, Math.max(1, scale * factor))
    if (scale === 1) resetZoom()
  }

  function onWheel(e: WheelEvent) {
    if (comparing || !stageEl) return
    e.preventDefault()
    const next = Math.min(8, Math.max(1, scale * (e.deltaY < 0 ? 1.15 : 1 / 1.15)))
    const r = stageEl.getBoundingClientRect()
    const cx = e.clientX - r.left - r.width / 2
    const cy = e.clientY - r.top - r.height / 2
    tx = cx - (cx - tx) * (next / scale)
    ty = cy - (cy - ty) * (next / scale)
    scale = next
    if (scale === 1) resetZoom()
  }

  function setSplit(clientX: number) {
    if (!stageEl) return
    const r = stageEl.getBoundingClientRect()
    split = Math.max(0, Math.min(100, ((clientX - r.left) / r.width) * 100))
  }

  $effect(() => {
    const move = (e: MouseEvent) => {
      if (dragging) setSplit(e.clientX)
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

  function history(redo: boolean) {
    if (image) store.undoRedo(image.id, redo)
  }

  $effect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (store.view !== 'loupe' || e.defaultPrevented) return
      const t = e.target as HTMLElement
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable)) return
      if (store.settingsOpen || store.shortcutsOpen || store.confirmRequest || store.menu) return
      const mod = e.ctrlKey || e.metaKey
      if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
        e.preventDefault()
        store.step(1)
      } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
        e.preventDefault()
        store.step(-1)
      } else if (e.key === 'Escape' || e.key === 'g') {
        e.preventDefault()
        store.view = 'grid'
      } else if (e.key === 'c') toggleCompare()
      else if (e.key === 'o' && !mod) peek(true)
      else if (e.key === '+' || e.key === '=') zoomBy(1.3)
      else if (e.key === '-') zoomBy(1 / 1.3)
      else if (e.key === '0') resetZoom()
      else if (mod && e.key.toLowerCase() === 'z') {
        e.preventDefault()
        history(e.shiftKey)
      } else if (mod && e.key.toLowerCase() === 'y') {
        e.preventDefault()
        history(true)
      }
    }
    const onKeyUp = (e: KeyboardEvent) => {
      if (e.key === 'o') peeking = false
    }
    window.addEventListener('keydown', onKey)
    window.addEventListener('keyup', onKeyUp)
    return () => {
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('keyup', onKeyUp)
    }
  })

  $effect(() => {
    const id = store.focusedId
    stripEl?.querySelector(`[data-id="${id}"]`)?.scrollIntoView({ inline: 'center', block: 'nearest', behavior: 'smooth' })
  })
</script>

<div class="@container flex h-full min-w-0 flex-col">
  <div class="flex h-11 shrink-0 items-center gap-2 border-b border-line px-2">
    <button class="btn-sm" onclick={() => (store.view = 'grid')} title="Back to grid (Esc)">
      <Icon name="arrowLeft" size={13} />Grid
    </button>
    {#if image}
      <span class="min-w-0 truncate px-1 text-[13px] font-medium text-ink" title={image.source_path}>
        {image.title ?? image.scan_id ?? image.filename}
      </span>
      <span class="shrink-0 text-xs tabular-nums @max-xl:hidden {image.year == null ? 'text-warning' : 'text-ink-dim'}">{dateLabel(image) ?? 'no date'}</span>
      <span class="shrink-0 text-xs text-ink-faint tabular-nums @max-xl:hidden">{idx + 1} / {store.total}</span>

      <div class="ml-auto flex shrink-0 items-center gap-1">
        {#if variants.length > 2}
          <div class="seg mr-1">
            <button aria-pressed={variant === 'organized'} onclick={() => pick('organized')} title="The restored copy at its scanned size">1×</button>
            <button aria-pressed={variant === 'enhanced'} onclick={() => pick('enhanced')} title="The 4× upscaled copy">4×</button>
          </div>
        {/if}
        {#if variants.length > 1}
          <button
            class="btn-sm"
            class:btn-sm-active={peeking}
            disabled={!canCompare || comparing}
            title="Hold to see the untouched scan (hold O)"
            onmousedown={() => peek(true)}
            onmouseup={() => peek(false)}
            onmouseleave={() => peek(false)}
          >
            <Icon name="eye" size={13} />Scan
          </button>
          <button class="btn-sm" class:btn-sm-active={comparing} disabled={!canCompare} onclick={toggleCompare} title="Slide between scan and result (C)">
            <Icon name="columns" size={13} />Compare
          </button>
          <div class="mx-1 h-4 w-px bg-line @max-xl:hidden"></div>
        {/if}
        <button
          class="btn-sm w-14 tabular-nums @max-xl:hidden"
          title="Zoom: scroll or double-click · fit with 0"
          disabled={comparing}
          onclick={() => (scale === 1 ? (scale = 2) : resetZoom())}
        >
          {scale === 1 ? 'Fit' : `${Math.round(scale * 100)}%`}
        </button>
        <div class="mx-1 h-4 w-px bg-line @max-xl:hidden"></div>
        <button class="icon-btn @max-xl:hidden" title="Undo last step (Ctrl+Z)" aria-label="Undo" disabled={snap.pos <= 0} onclick={() => history(false)}>
          <Icon name="undo" size={15} />
        </button>
        <button class="icon-btn @max-xl:hidden" title="Redo (Ctrl+Shift+Z)" aria-label="Redo" disabled={snap.pos >= snap.max} onclick={() => history(true)}>
          <Icon name="redo" size={15} />
        </button>
        <button class="icon-btn @max-xl:hidden" title="Rotate left ([)" aria-label="Rotate left" onclick={() => store.rotateTargets(false)}>
          <Icon name="rotateCcw" size={15} />
        </button>
        <button class="icon-btn @max-xl:hidden" title="Rotate right (])" aria-label="Rotate right" onclick={() => store.rotateTargets(true)}>
          <Icon name="rotateCw" size={15} />
        </button>
      </div>
    {/if}
  </div>

  <div
    bind:this={stageEl}
    class="relative min-h-0 flex-1 overflow-hidden bg-[#0e0c0a]"
    class:cursor-ew-resize={comparing}
    class:cursor-grab={!comparing && scale > 1 && !panning}
    class:cursor-grabbing={panning}
    onwheel={onWheel}
    ondblclick={() => {
      if (comparing) return
      if (scale > 1) resetZoom()
      else scale = 2
    }}
    onmousedown={(e) => {
      if (e.button !== 0) return
      if (comparing) {
        dragging = true
        setSplit(e.clientX)
      } else if (scale > 1) {
        panning = true
        panStart = { x: e.clientX - tx, y: e.clientY - ty }
      }
    }}
    role="presentation"
  >
    {#if !image}
      <div class="grid h-full place-items-center text-ink-dim">No photo selected</div>
    {:else if comparing && beforeUrl && url}
      <img src={url} alt="" draggable="false" class="absolute inset-4 h-[calc(100%-2rem)] w-[calc(100%-2rem)] object-contain" />
      <div class="absolute inset-0" style="clip-path: inset(0 {100 - split}% 0 0)">
        <img src={beforeUrl} alt="" draggable="false" class="absolute inset-4 h-[calc(100%-2rem)] w-[calc(100%-2rem)] object-contain" />
      </div>
      <span class="eyebrow pointer-events-none absolute top-3 left-3 rounded-[4px] px-2 py-1 text-ink" style="background: var(--bg-overlay);">Scan</span>
      <span class="eyebrow pointer-events-none absolute top-3 right-3 rounded-[4px] px-2 py-1 text-ink" style="background: var(--bg-overlay);">{variant === 'enhanced' ? 'Upscaled' : 'Restored'}</span>
      <div class="pointer-events-none absolute inset-y-0" style="left: {split}%; width: 2px; background: var(--color-brand); transform: translateX(-1px);">
        <span
          class="absolute top-1/2 left-1/2 grid h-8 w-8 -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full text-on-brand shadow-[var(--shadow-md)]"
          style="background: var(--color-brand);"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M9 6 4 12l5 6M15 6l5 6-5 6" /></svg>
        </span>
      </div>
    {:else if url}
      <img
        src={peeking && beforeUrl ? beforeUrl : url}
        alt={image.title ?? image.filename}
        draggable="false"
        class="absolute inset-4 h-[calc(100%-2rem)] w-[calc(100%-2rem)] object-contain select-none"
        style="transform: translate({tx}px, {ty}px) scale({scale}); transition: {panning ? 'none' : 'transform 120ms var(--ease-out)'};"
      />
    {/if}

    {#if peeking}
      <span class="eyebrow pointer-events-none absolute top-3 left-3 rounded-[4px] px-2 py-1 text-ink" style="background: var(--bg-overlay);">Scan</span>
    {/if}

    {#if loadingImg}
      <div class="absolute top-3 left-1/2 -translate-x-1/2"><Spinner size={16} /></div>
    {/if}

    {#if image && store.images.length > 1}
      <button
        class="absolute top-1/2 left-3 grid h-10 w-10 -translate-y-1/2 place-items-center rounded-full text-ink opacity-0 transition-opacity hover:opacity-100 focus-visible:opacity-100"
        style="background: var(--bg-overlay);"
        aria-label="Previous"
        onmousedown={(e) => e.stopPropagation()}
        onclick={() => store.step(-1)}
      >
        <Icon name="chevronLeft" size={20} />
      </button>
      <button
        class="absolute top-1/2 right-3 grid h-10 w-10 -translate-y-1/2 place-items-center rounded-full text-ink opacity-0 transition-opacity hover:opacity-100 focus-visible:opacity-100"
        style="background: var(--bg-overlay);"
        aria-label="Next"
        onmousedown={(e) => e.stopPropagation()}
        onclick={() => store.step(1)}
      >
        <Icon name="chevronRight" size={20} />
      </button>
    {/if}
  </div>

  <div bind:this={stripEl} class="flex h-[84px] shrink-0 gap-1.5 overflow-x-auto overflow-y-hidden border-t border-line bg-surface px-2 py-2">
    {#each store.images as img (img.id)}
      {@const active = img.id === store.focusedId}
      <button
        data-id={img.id}
        class="relative aspect-square h-full shrink-0 overflow-hidden rounded-[4px] transition-opacity {active ? 'opacity-100' : 'opacity-55 hover:opacity-90'}"
        style={active ? 'box-shadow: 0 0 0 2px var(--color-brand);' : ''}
        onclick={() => store.click(img.id)}
        title={img.title ?? img.filename}
      >
        <Thumb id={img.id} />
      </button>
    {/each}
    {#if store.hasMore}
      <button class="btn-sm h-full shrink-0" onclick={() => store.loadMore()}>More</button>
    {/if}
  </div>
</div>

<script lang="ts">
  import { store } from '../store.svelte'
  import { getImage, getVariant, imageHistory, rotateImage, updateImage } from '../api'
  import { MONTHS, type HistoryItem, type Image } from '../types'
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
    const move = (e: MouseEvent) => dragging && setFromClientX(e.clientX)
    const up = () => (dragging = false)
    window.addEventListener('mousemove', move)
    window.addEventListener('mouseup', up)
    return () => {
      window.removeEventListener('mousemove', move)
      window.removeEventListener('mouseup', up)
    }
  })

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
    await loadVariant()
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
    image = await rotateImage(image.id, clockwise)
    store.thumbVersion++
    await loadVariant()
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
      onmousedown={(e) => {
        if (comparing && compareUrl) {
          dragging = true
          setFromClientX(e.clientX)
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
            <Icon name="columns" size={17} stroke={2.1} />
          </span>
        </div>
      {:else if url}
        <img src={url} alt={variant} class="absolute inset-0 h-full w-full object-contain" />
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

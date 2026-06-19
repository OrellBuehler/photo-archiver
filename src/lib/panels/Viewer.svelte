<script lang="ts">
  import { store } from '../store.svelte'
  import { getImage, getVariant, imageHistory, rotateImage, updateImage } from '../api'
  import type { HistoryItem, Image } from '../types'

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
    <div class="flex flex-1 items-center justify-center p-6 text-center text-ink-dim">
      Double-click a photo to open it here.
    </div>
  {:else}
    <div class="flex items-center gap-1 border-b border-line px-2 py-1.5 text-xs">
      {#each variants as v (v)}
        <button class="btn-sm" class:!bg-brand={variant === v && !comparing} onclick={() => pick(v)}>
          {v}
        </button>
      {/each}
      <div class="ml-auto flex items-center gap-1">
        {#if variants.length > 1}
          <button class="btn-sm" class:!bg-brand={comparing} onclick={toggleCompare}>Compare</button>
        {/if}
        <button class="btn-sm" title="Rotate left" onclick={() => rotate(false)}>↺</button>
        <button class="btn-sm" title="Rotate right" onclick={() => rotate(true)}>↻</button>
      </div>
    </div>

    <div class="relative min-h-0 flex-1 overflow-hidden bg-black/40">
      {#if comparing && compareUrl && url}
        <img src={compareUrl} alt="source" class="absolute inset-0 h-full w-full object-contain" />
        <div class="absolute inset-0 overflow-hidden" style="clip-path: inset(0 {100 - split}% 0 0)">
          <img src={url} alt={variant} class="h-full w-full object-contain" />
        </div>
        <input
          type="range"
          min="0"
          max="100"
          bind:value={split}
          class="absolute bottom-3 left-1/2 w-2/3 -translate-x-1/2 accent-[var(--color-focus)]"
        />
        <span class="absolute left-2 top-2 rounded bg-black/60 px-1.5 text-[11px]">source</span>
        <span class="absolute right-2 top-2 rounded bg-black/60 px-1.5 text-[11px]">{variant}</span>
      {:else if url}
        <img src={url} alt={variant} class="absolute inset-0 h-full w-full object-contain" />
      {/if}
    </div>

    <div class="max-h-64 shrink-0 overflow-auto border-t border-line p-3 text-sm">
      <div class="mb-2 flex items-center justify-between">
        <span class="truncate font-medium text-ink" title={image.filename}>
          {image.title ?? image.filename}
        </span>
        <button class="text-xs text-ink-dim hover:text-ink" onclick={() => (editing = !editing)}>
          {editing ? 'Cancel' : 'Edit'}
        </button>
      </div>

      {#if editing}
        <div class="flex flex-col gap-2">
          <label class="flex items-center gap-2">
            <span class="w-14 text-ink-dim">Year</span>
            <input class="flex-1 rounded border border-line bg-surface px-2 py-1" bind:value={form.year} />
          </label>
          <label class="flex items-center gap-2">
            <span class="w-14 text-ink-dim">Month</span>
            <input class="flex-1 rounded border border-line bg-surface px-2 py-1" bind:value={form.month} />
          </label>
          <label class="flex items-center gap-2">
            <span class="w-14 text-ink-dim">Title</span>
            <input class="flex-1 rounded border border-line bg-surface px-2 py-1" bind:value={form.title} />
          </label>
          <button class="btn" onclick={save}>Save</button>
        </div>
      {:else}
        <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5 text-xs text-ink-dim">
          <dt>Status</dt>
          <dd class="text-ink">{image.status}</dd>
          <dt>Year / Month</dt>
          <dd class="text-ink">{image.year ?? '—'} / {image.month ?? '—'}</dd>
          <dt>Dimensions</dt>
          <dd class="text-ink">{image.width ?? '?'} × {image.height ?? '?'}</dd>
          <dt>Source</dt>
          <dd class="truncate text-ink" title={image.source_path}>{image.source_path}</dd>
        </dl>

        {#if history.length}
          <h3 class="mt-3 mb-1 text-[11px] font-semibold uppercase text-ink-dim">History</h3>
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

<script lang="ts">
  import type { Image } from '../types'
  import { thumbUrl } from '../thumbs'
  import { store } from '../store.svelte'

  let { image }: { image: Image } = $props()

  let src = $state<string | null>(null)
  let failed = $state(false)

  $effect(() => {
    // Re-run when the thumbnail cache is busted after processing.
    void store.thumbVersion
    let alive = true
    src = null
    failed = false
    thumbUrl(image.id)
      .then((u) => alive && (src = u))
      .catch(() => alive && (failed = true))
    return () => {
      alive = false
    }
  })

  const selected = $derived(store.selected.has(image.id))
  const processing = $derived(store.processingIds.has(image.id))
  const label = $derived(image.title ?? image.filename)

  const STATUS_COLOR: Record<string, string> = {
    new: 'var(--status-new)',
    organized: 'var(--status-organized)',
    enhanced: 'var(--status-enhanced)',
    processing: 'var(--status-processing)',
    failed: 'var(--status-failed)',
  }
  const dotColor = $derived(STATUS_COLOR[image.status] ?? 'var(--status-new)')
</script>

<button
  type="button"
  onclick={() => store.toggleSelect(image.id)}
  ondblclick={() => store.openImage(image.id)}
  class={`group relative aspect-square w-full overflow-hidden rounded-[5px] border-2 bg-surface-2 text-left transition-[border-color] duration-[110ms] ${
    selected ? 'border-focus' : 'border-line hover:border-[var(--border-strong)]'
  }`}
  title={label}
>
  {#if src}
    <img {src} alt={label} class="h-full w-full object-cover" loading="lazy" />
  {:else if failed}
    <div class="flex h-full w-full items-center justify-center text-ink-faint">⚠</div>
  {:else}
    <div class="h-full w-full animate-pulse bg-surface-2"></div>
  {/if}

  {#if processing}
    <div class="absolute inset-0 grid place-items-center" style="background: var(--bg-overlay);">
      <div
        class="h-5 w-5 rounded-full border-[2.5px] border-white/25"
        style="border-top-color: var(--color-brand); animation: pa-spin 0.7s linear infinite;"
      ></div>
    </div>
  {/if}

  <!-- selection check -->
  <span
    class="absolute left-1.5 top-1.5 grid h-[18px] w-[18px] place-items-center rounded-[3px] border-[1.5px] text-white transition-opacity duration-[110ms]"
    class:opacity-0={!selected}
    class:opacity-100={selected}
    class:group-hover:opacity-100={!selected}
    style={selected
      ? 'border-color: var(--color-focus); background: var(--color-focus);'
      : 'border-color: rgba(255,255,255,0.5); background: var(--bg-overlay);'}
  >
    {#if selected}
      <svg width="11" height="11" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg>
    {/if}
  </span>

  <!-- status dot -->
  <span
    class="absolute right-2 top-2 h-[9px] w-[9px] rounded-full"
    style="background: {dotColor}; box-shadow: 0 0 0 2px var(--bg-overlay);"
    title={image.status}
  ></span>

  <!-- caption -->
  <span
    class="absolute inset-x-0 bottom-0 truncate px-2 pb-1 pt-3.5 text-[11px] text-ink"
    style="background: linear-gradient(to top, rgba(12,10,8,0.85), transparent);"
  >
    {label}
  </span>
</button>

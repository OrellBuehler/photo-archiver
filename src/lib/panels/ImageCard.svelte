<script lang="ts">
  import type { Image } from '../types'
  import { thumbUrl } from '../thumbs'
  import { store } from '../store.svelte'

  let { image }: { image: Image } = $props()

  let src = $state<string | null>(null)
  let failed = $state(false)

  $effect(() => {
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
  const label = $derived(image.title ?? image.filename)
</script>

<button
  type="button"
  onclick={() => store.toggleSelect(image.id)}
  class="group relative aspect-square overflow-hidden rounded border bg-surface text-left transition-colors"
  class:border-line={!selected}
  class:border-focus={selected}
  title={label}
>
  {#if src}
    <img {src} alt={label} class="h-full w-full object-cover" loading="lazy" />
  {:else if failed}
    <div class="flex h-full w-full items-center justify-center text-ink-dim">⚠</div>
  {:else}
    <div class="h-full w-full animate-pulse bg-surface-2"></div>
  {/if}

  <span
    class="absolute left-1.5 top-1.5 grid h-4 w-4 place-items-center rounded-sm border text-[10px]"
    class:border-line={!selected}
    class:bg-base={!selected}
    class:border-focus={selected}
    class:bg-focus={selected}
    class:text-white={selected}
  >
    {#if selected}✓{/if}
  </span>

  <span
    class="absolute inset-x-0 bottom-0 truncate bg-gradient-to-t from-black/80 to-transparent px-1.5 pb-1 pt-3 text-[11px] text-ink"
  >
    {label}
  </span>
</button>

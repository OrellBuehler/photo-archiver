<script lang="ts">
  import { thumbUrl } from '../thumbs'
  import { store } from '../store.svelte'
  import Icon from './Icon.svelte'

  let { id, alt = '', fit = 'cover' }: { id: number; alt?: string; fit?: 'cover' | 'contain' } = $props()

  let src = $state<string | null>(null)
  let failed = $state(false)

  $effect(() => {
    void store.thumbVersion
    let alive = true
    src = null
    failed = false
    thumbUrl(id)
      .then((u) => alive && (src = u))
      .catch(() => alive && (failed = true))
    return () => {
      alive = false
    }
  })
</script>

{#if src}
  <img {src} {alt} draggable="false" class="h-full w-full {fit === 'cover' ? 'object-cover' : 'object-contain'}" />
{:else if failed}
  <div class="grid h-full w-full place-items-center text-ink-faint"><Icon name="alertTriangle" size={16} /></div>
{:else}
  <div class="h-full w-full animate-pulse bg-surface-2"></div>
{/if}

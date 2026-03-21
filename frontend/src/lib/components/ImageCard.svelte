<script lang="ts">
  import type { Image } from '$lib/types';
  import { thumbnailUrl } from '$lib/api';

  let { image, selected = false, processing = false, onselect, onclick }: {
    image: Image;
    selected?: boolean;
    processing?: boolean;
    onselect?: (id: number, checked: boolean) => void;
    onclick?: (image: Image) => void;
  } = $props();

  let loaded = $state(false);
</script>

<div
  class="group relative cursor-pointer overflow-hidden rounded-lg transition-[box-shadow,outline] hover:shadow-md"
  class:outline-2={selected}
  class:outline-primary={selected}
  role="button"
  tabindex="0"
  onclick={() => onclick?.(image)}
  onkeydown={(e) => e.key === 'Enter' && onclick?.(image)}
>
  {#if onselect}
    <div class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity" class:opacity-100={selected}>
      <input
        type="checkbox"
        checked={selected}
        onclick={(e) => e.stopPropagation()}
        onchange={(e) => onselect?.(image.id, e.currentTarget.checked)}
        class="h-4 w-4 rounded border-gray-300 shadow-sm"
      />
    </div>
  {/if}

  <div class="relative aspect-[4/3] bg-muted rounded-lg overflow-hidden">
    {#if !loaded}
      <div class="h-full w-full animate-pulse bg-muted"></div>
    {/if}
    <img
      src="{thumbnailUrl(image.id)}?t={image.updated_at}"
      alt={image.title || image.filename}
      class="h-full w-full object-cover"
      class:opacity-0={!loaded}
      onload={() => loaded = true}
    />
    {#if processing}
      <div class="absolute inset-0 flex items-center justify-center bg-black/30">
        <div class="h-8 w-8 animate-spin rounded-full border-2 border-white border-t-transparent"></div>
      </div>
    {/if}
  </div>

  <div class="px-0.5 pt-1.5 pb-1 opacity-80 group-hover:opacity-100 transition-opacity">
    <p class="text-xs font-medium truncate">{image.title || image.scan_id || image.filename}</p>
    <div class="flex items-center gap-1 mt-0.5">
      {#if image.year}
        <span class="text-[11px] text-muted-foreground">{image.year}{#if image.month}/{String(image.month).padStart(2, '0')}{/if}</span>
      {/if}
      <span class="ml-auto text-[11px] px-1.5 py-0.5 rounded-full {image.status === 'enhanced' ? 'bg-green-100 text-green-700' : image.status === 'organized' ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-600'}">{image.status}</span>
    </div>
  </div>
</div>

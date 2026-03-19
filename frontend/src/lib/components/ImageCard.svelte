<script lang="ts">
  import type { Image } from '$lib/types';
  import { thumbnailUrl } from '$lib/api';

  let { image, selected = false, onselect, onclick }: {
    image: Image;
    selected?: boolean;
    onselect?: (id: number, checked: boolean) => void;
    onclick?: (image: Image) => void;
  } = $props();

  let loaded = $state(false);
</script>

<div
  class="group relative cursor-pointer overflow-hidden rounded-lg border bg-card transition-shadow hover:shadow-lg"
  class:ring-2={selected}
  class:ring-primary={selected}
  role="button"
  tabindex="0"
  onclick={() => onclick?.(image)}
  onkeydown={(e) => e.key === 'Enter' && onclick?.(image)}
>
  {#if onselect}
    <div class="absolute top-2 left-2 z-10">
      <input
        type="checkbox"
        checked={selected}
        onclick={(e) => e.stopPropagation()}
        onchange={(e) => onselect?.(image.id, e.currentTarget.checked)}
        class="h-4 w-4 rounded border-gray-300"
      />
    </div>
  {/if}

  <div class="aspect-[4/3] bg-muted">
    {#if !loaded}
      <div class="h-full w-full animate-pulse bg-muted"></div>
    {/if}
    <img
      src={thumbnailUrl(image.id)}
      alt={image.title || image.filename}
      class="h-full w-full object-cover"
      class:opacity-0={!loaded}
      onload={() => loaded = true}
    />
  </div>

  <div class="p-2">
    <p class="text-sm font-medium truncate">{image.title || image.scan_id || image.filename}</p>
    <div class="flex items-center gap-1 mt-1">
      {#if image.year}
        <span class="text-xs text-muted-foreground">{image.year}</span>
      {/if}
      {#if image.month}
        <span class="text-xs text-muted-foreground">/ {String(image.month).padStart(2, '0')}</span>
      {/if}
      <span class="ml-auto text-xs px-1.5 py-0.5 rounded-full {image.status === 'enhanced' ? 'bg-green-100 text-green-700' : image.status === 'organized' ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-600'}">{image.status}</span>
    </div>
  </div>
</div>

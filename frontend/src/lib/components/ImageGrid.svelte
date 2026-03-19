<script lang="ts">
  import type { Image } from '$lib/types';
  import ImageCard from './ImageCard.svelte';

  let { images, selectedIds = new Set(), onselect, onclick }: {
    images: Image[];
    selectedIds?: Set<number>;
    onselect?: (id: number, checked: boolean) => void;
    onclick?: (image: Image) => void;
  } = $props();
</script>

{#if images.length === 0}
  <div class="flex items-center justify-center py-20 text-muted-foreground">
    No images found
  </div>
{:else}
  <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
    {#each images as image (image.id)}
      <ImageCard
        {image}
        selected={selectedIds.has(image.id)}
        {onselect}
        {onclick}
      />
    {/each}
  </div>
{/if}

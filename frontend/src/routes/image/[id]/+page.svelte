<script lang="ts">
  import { onMount } from 'svelte';
  import { getImage } from '$lib/api';
  import type { Image } from '$lib/types';
  import ImageDetail from '$lib/components/ImageDetail.svelte';

  let { data } = $props();

  let image = $state<Image | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      image = await getImage(data.imageId);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load image';
    }
  });
</script>

{#if error}
  <div class="flex items-center justify-center py-20">
    <p class="text-destructive">{error}</p>
  </div>
{:else if image}
  <ImageDetail {image} />
{:else}
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <div class="lg:col-span-2">
      <div class="rounded-lg border overflow-hidden bg-muted aspect-[4/3] animate-pulse"></div>
    </div>
    <div class="space-y-4">
      <div class="h-6 bg-muted animate-pulse rounded w-3/4"></div>
      <div class="h-4 bg-muted animate-pulse rounded w-1/2"></div>
      <div class="rounded-lg border p-4 space-y-3">
        {#each Array(6) as _}
          <div class="h-4 bg-muted animate-pulse rounded"></div>
        {/each}
      </div>
    </div>
  </div>
{/if}

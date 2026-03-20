<script lang="ts">
  import { onMount } from 'svelte';
  import { getImages, getImageStats } from '$lib/api';
  import type { Image, ImageStats } from '$lib/types';
  import ImageGrid from '$lib/components/ImageGrid.svelte';
  import FilterBar from '$lib/components/FilterBar.svelte';
  import ProcessingPanel from '$lib/components/ProcessingPanel.svelte';

  let images = $state<Image[]>([]);
  let stats = $state<ImageStats[]>([]);
  let total = $state(0);
  let page = $state(1);
  let perPage = 60;
  let loading = $state(true);

  let selectedYear = $state<number | null>(null);
  let selectedStatus = $state<string | null>(null);
  let selectedIds = $state(new Set<number>());

  let pageImageIds = $derived(images.map((img) => img.id));

  async function loadImages() {
    loading = true;
    try {
      const res = await getImages({
        year: selectedYear,
        status: selectedStatus,
        page,
        per_page: perPage,
      });
      images = res.images;
      total = res.total;
    } finally {
      loading = false;
    }
  }

  async function loadStats() {
    stats = await getImageStats();
  }

  async function refresh() {
    await Promise.all([loadImages(), loadStats()]);
  }

  function handleFilter(year: number | null, status: string | null) {
    selectedYear = year;
    selectedStatus = status;
    page = 1;
    loadImages();
  }

  function handleSelect(id: number, checked: boolean) {
    const next = new Set(selectedIds);
    if (checked) next.add(id); else next.delete(id);
    selectedIds = next;
  }

  function handleClick(image: Image) {
    window.location.href = `/image/${image.id}`;
  }

  onMount(() => {
    loadStats();
    loadImages();
  });

  let totalPages = $derived(Math.ceil(total / perPage));
</script>

<FilterBar
  {stats}
  {selectedYear}
  {selectedStatus}
  {total}
  onfilter={handleFilter}
/>

{#if loading}
  <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
    {#each Array(12) as _}
      <div class="rounded-lg border overflow-hidden">
        <div class="aspect-[4/3] bg-muted animate-pulse"></div>
        <div class="p-2 space-y-2">
          <div class="h-4 bg-muted animate-pulse rounded"></div>
          <div class="h-3 bg-muted animate-pulse rounded w-2/3"></div>
        </div>
      </div>
    {/each}
  </div>
{:else}
  <ImageGrid
    {images}
    {selectedIds}
    onselect={handleSelect}
    onclick={handleClick}
  />

  {#if totalPages > 1}
    <div class="flex items-center justify-center gap-2 py-6">
      <button
        class="rounded-md border px-3 py-1.5 text-sm disabled:opacity-50"
        disabled={page <= 1}
        onclick={() => { page--; loadImages(); }}
      >Previous</button>
      <span class="text-sm text-muted-foreground">Page {page} of {totalPages}</span>
      <button
        class="rounded-md border px-3 py-1.5 text-sm disabled:opacity-50"
        disabled={page >= totalPages}
        onclick={() => { page++; loadImages(); }}
      >Next</button>
    </div>
  {/if}
{/if}

<ProcessingPanel
  {selectedIds}
  totalOnPage={images.length}
  {pageImageIds}
  onTaskCreated={(id) => {
    selectedIds = new Set();
    window.location.href = '/processing';
  }}
  onSelectionChange={(ids) => { selectedIds = ids; }}
  onRefresh={refresh}
/>

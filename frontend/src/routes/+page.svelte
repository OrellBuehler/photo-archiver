<script lang="ts">
  import { onMount } from 'svelte';
  import { getImages, getImageStats } from '$lib/api';
  import { subscribe } from '$lib/ws';
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
  let processingIds = $state(new Set<number>());

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

  let unsubscribe: (() => void) | undefined;

  onMount(() => {
    loadStats();
    loadImages();
    unsubscribe = subscribe((msg) => {
      if (msg.type === 'image_started' && msg.image_id) {
        processingIds = new Set([...processingIds, msg.image_id]);
      } else if (msg.type === 'progress' && msg.image_id) {
        const next = new Set(processingIds);
        next.delete(msg.image_id);
        processingIds = next;
      } else if (msg.type === 'task_completed' || msg.type === 'task_failed') {
        processingIds = new Set();
      }
    });
    return () => unsubscribe?.();
  });

  let totalPages = $derived(Math.ceil(total / perPage));
</script>

<div class="flex flex-col h-full">
  <div class="shrink-0">
    <FilterBar
      {stats}
      {selectedYear}
      {selectedStatus}
      {total}
      onfilter={handleFilter}
    />
  </div>

  <div class="flex-1 min-h-0 overflow-y-auto py-4">
    {#if loading}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 lg:gap-4">
        {#each Array(12) as _}
          <div>
            <div class="aspect-[4/3] bg-muted animate-pulse rounded-lg"></div>
            <div class="pt-1.5 space-y-1">
              <div class="h-3 bg-muted animate-pulse rounded w-3/4"></div>
              <div class="h-3 bg-muted animate-pulse rounded w-1/3"></div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <ImageGrid
        {images}
        {selectedIds}
        {processingIds}
        onselect={handleSelect}
        onclick={handleClick}
      />
    {/if}
  </div>

  {#if totalPages > 1}
    <div class="shrink-0 flex items-center justify-center gap-2 py-3 border-t">
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
</div>

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

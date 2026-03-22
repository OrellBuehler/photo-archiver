<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { getImages, getImageStats } from '$lib/api';
  import { subscribe } from '$lib/ws';
  import type { Image, FilterCounts, FilterParams } from '$lib/types';
  import ImageGrid from '$lib/components/ImageGrid.svelte';
  import FilterBar from '$lib/components/FilterBar.svelte';
  import ProcessingPanel from '$lib/components/ProcessingPanel.svelte';

  let images = $state<Image[]>([]);
  let counts = $state<FilterCounts | null>(null);
  let total = $state(0);
  let perPage = 60;
  let loading = $state(true);

  let selectedIds = $state(new Set<number>());
  let processingIds = $state(new Set<number>());

  let pageImageIds = $derived(images.map((img) => img.id));

  let filters = $derived<FilterParams>({
    year: $page.url.searchParams.has('year') ? Number($page.url.searchParams.get('year')) : null,
    month: $page.url.searchParams.has('month') ? Number($page.url.searchParams.get('month')) : null,
    status: $page.url.searchParams.get('status') ?? null,
    step: $page.url.searchParams.get('step') ?? null,
    year_unknown: $page.url.searchParams.has('year_unknown') ? true : null,
  });

  let currentPage = $derived(
    $page.url.searchParams.has('page') ? Number($page.url.searchParams.get('page')) : 1
  );

  function setFilter(key: string, value: string | null) {
    const sp = new URLSearchParams($page.url.searchParams);
    if (value === null) sp.delete(key);
    else sp.set(key, value);
    if (key !== 'page') sp.delete('page');
    goto(`?${sp}`, { keepFocus: true });
  }

  function setPage(p: number) {
    const sp = new URLSearchParams($page.url.searchParams);
    if (p <= 1) sp.delete('page');
    else sp.set('page', String(p));
    goto(`?${sp}`, { keepFocus: true });
  }

  async function loadImages() {
    loading = true;
    try {
      const res = await getImages({
        ...filters,
        page: currentPage,
        per_page: perPage,
      });
      images = res.images;
      total = res.total;
    } finally {
      loading = false;
    }
  }

  async function loadStats() {
    counts = await getImageStats(filters);
  }

  async function refresh() {
    await Promise.all([loadImages(), loadStats()]);
  }

  function handleSelect(id: number, checked: boolean) {
    const next = new Set(selectedIds);
    if (checked) next.add(id); else next.delete(id);
    selectedIds = next;
  }

  function handleClick(image: Image) {
    goto(`/image/${image.id}`);
  }

  let unsubscribe: (() => void) | undefined;

  onMount(() => {
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

  $effect(() => {
    void filters;
    void currentPage;
    loadImages();
    loadStats();
  });

  let totalPages = $derived(Math.ceil(total / perPage));
</script>

<div class="flex flex-col h-full">
  <div class="shrink-0">
    <FilterBar
      {counts}
      {filters}
      onfilter={setFilter}
    />
  </div>

  <div class="flex-1 min-h-0 overflow-y-auto py-3">
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
        disabled={currentPage <= 1}
        onclick={() => setPage(currentPage - 1)}
      >Previous</button>
      <span class="text-sm text-muted-foreground">Page {currentPage} of {totalPages}</span>
      <button
        class="rounded-md border px-3 py-1.5 text-sm disabled:opacity-50"
        disabled={currentPage >= totalPages}
        onclick={() => setPage(currentPage + 1)}
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
    goto('/processing');
  }}
  onSelectionChange={(ids) => { selectedIds = ids; }}
  onRefresh={refresh}
/>

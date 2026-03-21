<script lang="ts">
  import { onMount } from 'svelte';
  import { getDuplicates, scanDuplicates, bulkDeleteImages, thumbnailUrl } from '$lib/api';
  import type { DuplicateGroup } from '$lib/types';

  let groups = $state<DuplicateGroup[]>([]);
  let loading = $state(true);
  let scanning = $state(false);
  let threshold = $state(6);
  let debounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  let selected = $state<Map<number, Set<number>>>(new Map());

  async function loadDuplicates() {
    loading = true;
    try {
      groups = await getDuplicates(threshold);
      selected = new Map();
    } finally {
      loading = false;
    }
  }

  async function handleScan() {
    scanning = true;
    try {
      await scanDuplicates();
      await loadDuplicates();
    } finally {
      scanning = false;
    }
  }

  function handleThresholdChange() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(loadDuplicates, 300);
  }

  function toggleImage(groupIdx: number, imageId: number, checked: boolean) {
    const next = new Map(selected);
    const groupSet = new Set(next.get(groupIdx) ?? []);
    if (checked) groupSet.add(imageId); else groupSet.delete(imageId);
    next.set(groupIdx, groupSet);
    selected = next;
  }

  async function deleteSelected(groupIdx: number) {
    const ids = Array.from(selected.get(groupIdx) ?? []);
    if (ids.length === 0) return;
    if (!window.confirm(`Delete ${ids.length} image(s)? This cannot be undone.`)) return;
    await bulkDeleteImages(ids);
    groups = groups
      .map((g, i) => i === groupIdx ? { ...g, images: g.images.filter(img => !ids.includes(img.id)) } : g)
      .filter(g => g.images.length > 1);
    const next = new Map(selected);
    next.delete(groupIdx);
    selected = next;
  }

  function statusColor(s: string): string {
    switch (s) {
      case 'organized': return 'bg-green-100 text-green-700';
      case 'enhanced': return 'bg-blue-100 text-blue-700';
      case 'failed': return 'bg-red-100 text-red-700';
      default: return 'bg-gray-100 text-gray-600';
    }
  }

  onMount(loadDuplicates);
</script>

<div class="py-4 space-y-6 h-full overflow-y-auto">
  <div class="flex items-center gap-4 flex-wrap">
    <h1 class="text-2xl font-bold">Duplicates</h1>
    <button
      class="rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium disabled:opacity-50"
      disabled={scanning}
      onclick={handleScan}
    >
      {scanning ? 'Scanning...' : 'Scan'}
    </button>
    <label class="flex items-center gap-2 text-sm ml-auto">
      Threshold: <span class="font-medium w-4 text-center">{threshold}</span>
      <input
        type="range"
        min="0"
        max="20"
        bind:value={threshold}
        oninput={handleThresholdChange}
        class="w-32"
      />
    </label>
    {#if !loading}
      <span class="text-sm text-muted-foreground">{groups.length} duplicate groups found</span>
    {/if}
  </div>

  {#if loading}
    <div class="space-y-4">
      {#each Array(3) as _}
        <div class="rounded-lg border p-4 space-y-3">
          <div class="h-5 bg-muted animate-pulse rounded w-40"></div>
          <div class="flex gap-3">
            {#each Array(4) as _}
              <div class="w-36 space-y-2">
                <div class="aspect-[4/3] bg-muted animate-pulse rounded"></div>
                <div class="h-3 bg-muted animate-pulse rounded"></div>
                <div class="h-3 bg-muted animate-pulse rounded w-2/3"></div>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {:else if groups.length === 0}
    <p class="text-sm text-muted-foreground py-8 text-center">
      No duplicates found. Try adjusting the threshold or scanning for new images.
    </p>
  {:else}
    <div class="space-y-4">
      {#each groups as group, groupIdx}
        {@const groupSelected = selected.get(groupIdx) ?? new Set()}
        <div class="rounded-lg border">
          <div class="flex items-center justify-between px-4 py-3 border-b">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium">Group</span>
              <span class="text-xs px-2 py-0.5 rounded-full bg-muted text-muted-foreground">
                Distance: {group.distance}
              </span>
            </div>
            <button
              class="rounded-md border border-destructive text-destructive px-3 py-1.5 text-xs hover:bg-destructive/10 disabled:opacity-50"
              disabled={groupSelected.size === 0}
              onclick={() => deleteSelected(groupIdx)}
            >
              Delete Selected ({groupSelected.size})
            </button>
          </div>
          <div class="p-4 flex flex-wrap gap-3">
            {#each group.images as image}
              <label class="flex flex-col gap-1 cursor-pointer w-36 group">
                <div class="relative rounded overflow-hidden border aspect-[4/3]">
                  <img
                    src={thumbnailUrl(image.id)}
                    alt={image.title ?? image.filename}
                    class="w-full h-full object-cover"
                  />
                  <input
                    type="checkbox"
                    checked={groupSelected.has(image.id)}
                    onchange={(e) => toggleImage(groupIdx, image.id, (e.target as HTMLInputElement).checked)}
                    class="absolute top-1.5 left-1.5 w-4 h-4"
                  />
                </div>
                <p class="text-xs font-medium truncate">{image.title ?? image.filename}</p>
                <div class="flex items-center gap-1 flex-wrap">
                  {#if image.year}
                    <span class="text-xs text-muted-foreground">{image.year}</span>
                  {/if}
                  {#if image.width && image.height}
                    <span class="text-xs text-muted-foreground">{image.width}×{image.height}</span>
                  {/if}
                  <span class="text-xs px-1.5 py-0.5 rounded-full {statusColor(image.status)}">{image.status}</span>
                </div>
              </label>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

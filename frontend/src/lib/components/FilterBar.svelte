<script lang="ts">
  import type { ImageStats } from '$lib/types';

  let { stats, selectedYear, selectedStatus, total, onfilter }: {
    stats: ImageStats[];
    selectedYear: number | null;
    selectedStatus: string | null;
    total: number;
    onfilter: (year: number | null, status: string | null) => void;
  } = $props();

  let years = $derived(
    [...new Set(stats.map(s => s.year).filter((y): y is number => y !== null))].sort()
  );

  let statuses = ['source', 'organized', 'enhanced'];
</script>

<div class="flex flex-wrap items-center gap-3 py-4">
  <div class="flex items-center gap-2">
    <label class="text-sm font-medium">Year:</label>
    <select
      class="rounded-md border bg-background px-3 py-1.5 text-sm"
      value={selectedYear ?? ''}
      onchange={(e) => {
        const val = e.currentTarget.value;
        onfilter(val ? Number(val) : null, selectedStatus);
      }}
    >
      <option value="">All</option>
      {#each years as year}
        <option value={year}>{year}</option>
      {/each}
    </select>
  </div>

  <div class="flex items-center gap-2">
    <label class="text-sm font-medium">Status:</label>
    <div class="flex gap-1">
      <button
        class="rounded-md px-3 py-1.5 text-sm transition-colors {selectedStatus === null ? 'bg-primary text-primary-foreground' : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'}"
        onclick={() => onfilter(selectedYear, null)}
      >All</button>
      {#each statuses as status}
        <button
          class="rounded-md px-3 py-1.5 text-sm transition-colors {selectedStatus === status ? 'bg-primary text-primary-foreground' : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'}"
          onclick={() => onfilter(selectedYear, status)}
        >{status}</button>
      {/each}
    </div>
  </div>

  <span class="ml-auto text-sm text-muted-foreground">{total} images</span>
</div>

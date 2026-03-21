<script lang="ts">
  import type { FilterCounts } from '$lib/types';

  let { stats, selectedYear, selectedStatus, total, onfilter }: {
    stats: FilterCounts | null;
    selectedYear: number | null;
    selectedStatus: string | null;
    total: number;
    onfilter: (year: number | null, status: string | null) => void;
  } = $props();

  let years = $derived(
    (stats?.years ?? []).map(y => y.value).filter((y): y is number => typeof y === 'number').sort()
  );

  let statuses = ['source', 'organized', 'enhanced'];
</script>

<div class="flex flex-wrap items-center gap-3 py-3">
  <div class="flex items-center gap-2">
    <label class="text-xs text-muted-foreground uppercase tracking-wide">Year</label>
    <select
      class="rounded-md border bg-background px-2.5 py-1 text-sm"
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
    <label class="text-xs text-muted-foreground uppercase tracking-wide">Status</label>
    <div class="flex gap-0.5">
      <button
        class="rounded-md px-2.5 py-1 text-sm transition-colors {selectedStatus === null ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-secondary'}"
        onclick={() => onfilter(selectedYear, null)}
      >All</button>
      {#each statuses as status}
        <button
          class="rounded-md px-2.5 py-1 text-sm transition-colors capitalize {selectedStatus === status ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-secondary'}"
          onclick={() => onfilter(selectedYear, status)}
        >{status}</button>
      {/each}
    </div>
  </div>

  <span class="ml-auto text-xs text-muted-foreground">{total} images</span>
</div>

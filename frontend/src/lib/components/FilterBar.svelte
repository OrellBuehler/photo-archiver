<script lang="ts">
  import type { FilterCounts, FilterParams } from '$lib/types';

  const MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const STEP_LABELS: Record<string, string> = {
    organize: 'Organize',
    auto_orient: 'Auto Orient',
    deskew: 'Deskew',
    restore_color: 'Restore Color',
    remove_dust: 'Remove Dust',
    enhance: 'Enhance',
  };

  let { counts, filters, onfilter }: {
    counts: FilterCounts | null;
    filters: FilterParams;
    onfilter: (key: string, value: string | null) => void;
  } = $props();

  let years = $derived(
    (counts?.years ?? [])
      .filter(y => y.value !== null)
      .sort((a, b) => (a.value as number) - (b.value as number))
  );

  let unknownYearCount = $derived(
    counts?.years.find(y => y.value === null)?.count ?? 0
  );

  let months = $derived(
    (counts?.months ?? [])
      .filter(m => m.value !== null)
      .sort((a, b) => (a.value as number) - (b.value as number))
  );

  let statuses = $derived(counts?.statuses ?? []);
  let steps = $derived((counts?.steps ?? []).filter(s => s.count > 0));

  function countFor(items: { value: string | number | null; count: number }[], val: string | number | null): number {
    return items.find(i => i.value === val)?.count ?? 0;
  }

  function toggleFilter(key: string, value: string | null) {
    const current = (filters as Record<string, unknown>)[key];
    onfilter(key, current?.toString() === value ? null : value);
  }
</script>

<div class="flex flex-col gap-1.5 py-3">
  <div class="flex flex-wrap items-center gap-x-4 gap-y-1.5">
    <span class="text-xs text-muted-foreground uppercase tracking-wide w-12 shrink-0">Year</span>
    <div class="flex flex-wrap gap-1">
      <button
        class="rounded-full px-3 py-1 text-sm transition-colors {!filters.year && !filters.year_unknown ? 'bg-primary text-primary-foreground' : 'bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground'}"
        onclick={() => { onfilter('year', null); onfilter('year_unknown', null); }}
      >All</button>
      {#each years as y}
        <button
          class="rounded-full px-3 py-1 text-sm transition-colors {filters.year === y.value ? 'bg-primary text-primary-foreground' : 'bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground'}"
          onclick={() => { onfilter('year_unknown', null); toggleFilter('year', String(y.value)); }}
        >{y.value} <span class="opacity-60">({y.count})</span></button>
      {/each}
      {#if unknownYearCount > 0}
        <button
          class="rounded-full px-3 py-1 text-sm transition-colors {filters.year_unknown ? 'bg-primary text-primary-foreground' : 'bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground'}"
          onclick={() => { onfilter('year', null); onfilter('year_unknown', filters.year_unknown ? null : 'true'); }}
        >Unknown <span class="opacity-60">({unknownYearCount})</span></button>
      {/if}
    </div>
  </div>

  {#if filters.year}
    <div class="flex flex-wrap items-center gap-x-4 gap-y-1.5">
      <span class="text-xs text-muted-foreground uppercase tracking-wide w-12 shrink-0">Month</span>
      <div class="flex flex-wrap gap-1">
        <button
          class="rounded-full px-3 py-1 text-sm transition-colors {!filters.month ? 'bg-primary text-primary-foreground' : 'bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground'}"
          onclick={() => onfilter('month', null)}
        >All</button>
        {#each months as m}
          <button
            class="rounded-full px-3 py-1 text-sm transition-colors {filters.month === m.value ? 'bg-primary text-primary-foreground' : 'bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground'}"
            onclick={() => toggleFilter('month', String(m.value))}
          >{MONTH_NAMES[(m.value as number) - 1] ?? m.value} <span class="opacity-60">({m.count})</span></button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="flex flex-wrap items-center gap-x-4 gap-y-1.5">
    <span class="text-xs text-muted-foreground uppercase tracking-wide w-12 shrink-0">Status</span>
    <div class="flex flex-wrap gap-1">
      <button
        class="rounded-full px-3 py-1 text-sm transition-colors {!filters.status ? 'bg-primary text-primary-foreground' : 'bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground'}"
        onclick={() => onfilter('status', null)}
      >All</button>
      {#each statuses as s}
        <button
          class="rounded-full px-3 py-1 text-sm transition-colors capitalize {filters.status === s.value ? 'bg-primary text-primary-foreground' : 'bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground'}"
          onclick={() => toggleFilter('status', String(s.value))}
        >{s.value} <span class="opacity-60">({s.count})</span></button>
      {/each}
    </div>
  </div>

  {#if steps.length > 0}
    <div class="flex flex-wrap items-center gap-x-4 gap-y-1.5">
      <span class="text-xs text-muted-foreground uppercase tracking-wide w-12 shrink-0">Steps</span>
      <div class="flex flex-wrap gap-1">
        {#each steps as s}
          <button
            class="rounded-full px-3 py-1 text-sm transition-colors {filters.step === s.value ? 'bg-primary text-primary-foreground' : 'bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground'}"
            onclick={() => toggleFilter('step', String(s.value))}
          >{STEP_LABELS[String(s.value)] ?? s.value} <span class="opacity-60">({s.count})</span></button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="text-xs text-muted-foreground text-right">{counts?.total ?? 0} images</div>
</div>

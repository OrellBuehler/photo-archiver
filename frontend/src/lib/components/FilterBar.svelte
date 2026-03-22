<script lang="ts">
  import type { FilterCounts, FilterParams } from '$lib/types';

  const MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const STEP_LABELS: Record<string, string> = {
    organize: 'Organize',
    crop: 'Crop',
    auto_orient: 'Auto Orient',
    deskew: 'Deskew',
    restore_color: 'Restore Color',
    remove_dust: 'Remove Dust',
    remove_lines: 'Remove Lines',
    enhance: 'Enhance',
    rotate_left: 'Rotate Left',
    rotate_right: 'Rotate Right',
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

  function toggleFilter(key: string, value: string | null) {
    const current = (filters as Record<string, unknown>)[key];
    onfilter(key, current?.toString() === value ? null : value);
  }

  const pill = 'rounded-md px-2.5 py-1 text-sm transition-colors';
  const pillActive = `${pill} bg-foreground text-background`;
  const pillInactive = `${pill} text-muted-foreground hover:bg-secondary hover:text-foreground`;
</script>

<div class="flex flex-col gap-1.5 py-2 border-b">
  <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1.5">
    <span class="text-sm font-medium text-foreground w-14 shrink-0">Year</span>
    <div class="flex gap-1 overflow-x-auto" style="mask-image: linear-gradient(to right, transparent 0%, black 2%, black 95%, transparent 100%); scrollbar-width: none; -ms-overflow-style: none;">
      <button
        class={!filters.year && !filters.year_unknown ? pillActive : pillInactive}
        onclick={() => { onfilter('year', null); onfilter('year_unknown', null); }}
      >All</button>
      {#each years as y}
        <button
          class={filters.year === y.value ? pillActive : pillInactive}
          onclick={() => { onfilter('year_unknown', null); toggleFilter('year', String(y.value)); }}
        >{y.value} <span class="text-xs opacity-50">{y.count}</span></button>
      {/each}
      {#if unknownYearCount > 0}
        <button
          class={filters.year_unknown ? pillActive : pillInactive}
          onclick={() => { onfilter('year', null); onfilter('year_unknown', filters.year_unknown ? null : 'true'); }}
        >Unknown <span class="text-xs opacity-50">{unknownYearCount}</span></button>
      {/if}
    </div>
  </div>

  {#if filters.year}
    <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1.5">
      <span class="text-sm font-medium text-foreground w-14 shrink-0">Month</span>
      <div class="flex flex-wrap gap-1">
        <button
          class={!filters.month ? pillActive : pillInactive}
          onclick={() => onfilter('month', null)}
        >All</button>
        {#each months as m}
          <button
            class={filters.month === m.value ? pillActive : pillInactive}
            onclick={() => toggleFilter('month', String(m.value))}
          >{MONTH_NAMES[(m.value as number) - 1] ?? m.value} <span class="text-xs opacity-50">{m.count}</span></button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1.5">
    <span class="text-sm font-medium text-foreground w-14 shrink-0">Status</span>
    <div class="flex flex-wrap gap-1">
      <button
        class={!filters.status ? pillActive : pillInactive}
        onclick={() => onfilter('status', null)}
      >All</button>
      {#each statuses as s}
        <button
          class="{filters.status === s.value ? pillActive : pillInactive} capitalize"
          onclick={() => toggleFilter('status', String(s.value))}
        >{s.value} <span class="text-xs opacity-50">{s.count}</span></button>
      {/each}
    </div>
  </div>

  {#if steps.length > 0}
    <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1.5">
      <span class="text-sm font-medium text-foreground w-14 shrink-0">Steps</span>
      <div class="flex flex-wrap gap-1">
        {#each steps as s}
          <button
            class={filters.step === s.value ? pillActive : pillInactive}
            onclick={() => toggleFilter('step', String(s.value))}
          >{STEP_LABELS[String(s.value)] ?? s.value} <span class="text-xs opacity-50">{s.count}</span></button>
        {/each}
      </div>
    </div>
  {/if}

  <div class="text-xs text-muted-foreground text-right -mt-1">{counts?.total ?? 0} images</div>
</div>

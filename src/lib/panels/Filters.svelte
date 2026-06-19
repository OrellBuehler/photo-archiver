<script lang="ts">
  import { store } from '../store.svelte'
  import { MONTHS } from '../types'

  const f = $derived(store.filters)
  const hasActive = $derived(
    f.year != null || f.month != null || f.status != null || f.year_unknown,
  )

  function pickYear(value: string) {
    if (value === 'unknown') {
      store.setFilter({ year_unknown: !f.year_unknown, year: null, month: null })
    } else {
      const y = Number(value)
      store.setFilter({ year: f.year === y ? null : y, year_unknown: false, month: null })
    }
  }

  function pickMonth(value: string) {
    const m = Number(value)
    store.setFilter({ month: f.month === m ? null : m })
  }

  function pickStatus(value: string) {
    store.setFilter({ status: f.status === value ? null : value })
  }

  const yearActive = (v: string) =>
    v === 'unknown' ? f.year_unknown : f.year === Number(v)
</script>

<div class="flex h-full flex-col gap-4 overflow-auto bg-base p-3 text-sm">
  <div class="flex items-center justify-between">
    <h2 class="text-xs font-semibold uppercase tracking-wide text-ink-dim">Filters</h2>
    {#if hasActive}
      <button class="text-xs text-ink-dim hover:text-ink" onclick={() => store.clearFilters()}>
        Clear
      </button>
    {/if}
  </div>

  {#if store.counts.years.length}
    <section>
      <h3 class="mb-2 text-[11px] font-semibold uppercase text-ink-dim">Year</h3>
      <div class="flex flex-wrap gap-1.5">
        {#each store.counts.years as y (y.value)}
          <button class="pill" class:pill-active={yearActive(y.value)} onclick={() => pickYear(y.value)}>
            {y.value}
            <span class="opacity-60">{y.count}</span>
          </button>
        {/each}
      </div>
    </section>
  {/if}

  {#if store.counts.months.length}
    <section>
      <h3 class="mb-2 text-[11px] font-semibold uppercase text-ink-dim">Month</h3>
      <div class="flex flex-wrap gap-1.5">
        {#each store.counts.months as m (m.value)}
          <button class="pill" class:pill-active={f.month === Number(m.value)} onclick={() => pickMonth(m.value)}>
            {MONTHS[Number(m.value)] ?? m.value}
            <span class="opacity-60">{m.count}</span>
          </button>
        {/each}
      </div>
    </section>
  {/if}

  {#if store.counts.statuses.length}
    <section>
      <h3 class="mb-2 text-[11px] font-semibold uppercase text-ink-dim">Status</h3>
      <div class="flex flex-wrap gap-1.5">
        {#each store.counts.statuses as s (s.value)}
          <button class="pill" class:pill-active={f.status === s.value} onclick={() => pickStatus(s.value)}>
            {s.value}
            <span class="opacity-60">{s.count}</span>
          </button>
        {/each}
      </div>
    </section>
  {/if}

  {#if !store.counts.years.length && !store.counts.statuses.length}
    <p class="text-ink-dim">No photos scanned yet.</p>
  {/if}
</div>

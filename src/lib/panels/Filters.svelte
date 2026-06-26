<script lang="ts">
  import { store } from '../store.svelte'
  import { MONTHS } from '../types'
  import Icon from '../ui/Icon.svelte'

  const f = $derived(store.filters)
  const hasActive = $derived(
    f.year != null ||
      f.month != null ||
      f.status != null ||
      f.folder != null ||
      f.year_unknown ||
      !!f.search,
  )

  function pickFolder(value: string) {
    store.setFilter({ folder: f.folder === value ? null : value })
  }

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
    <h2 class="flex items-center gap-1.5 text-sm font-semibold text-ink">
      <Icon name="sliders" size={15} class="text-ink-dim" />
      Filters
    </h2>
    {#if hasActive}
      <button class="text-xs text-ink-dim hover:text-ink" onclick={() => store.clearFilters()}>
        Clear
      </button>
    {/if}
  </div>

  <div class="relative">
    <Icon name="search" size={14} class="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-ink-faint" />
    <input
      class="input pl-7"
      placeholder="Search filename or title…"
      value={f.search ?? ''}
      oninput={(e) => store.setSearch((e.currentTarget as HTMLInputElement).value)}
    />
  </div>

  {#if store.counts.folders.length}
    <section>
      <h3 class="eyebrow mb-2">Folder</h3>
      <div class="flex flex-wrap gap-1.5">
        {#each store.counts.folders as fl (fl.value)}
          <button class="pill" class:pill-active={f.folder === fl.value} onclick={() => pickFolder(fl.value)}>
            {fl.value}
            <span class="opacity-60">{fl.count}</span>
          </button>
        {/each}
      </div>
    </section>
  {/if}

  {#if store.counts.years.length}
    <section>
      <h3 class="eyebrow mb-2">Year</h3>
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
      <h3 class="eyebrow mb-2">Month</h3>
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
      <h3 class="eyebrow mb-2">Status</h3>
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

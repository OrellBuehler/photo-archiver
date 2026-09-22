<script lang="ts">
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'

  const total = $derived(store.libraryTotal)
  const rows = $derived([
    { key: 'enhanced', label: 'Restored', color: 'var(--status-enhanced)' },
    { key: 'organized', label: 'Organized', color: 'var(--status-organized)' },
    { key: 'new', label: 'To restore', color: 'var(--status-new)' },
    { key: 'failed', label: 'Failed', color: 'var(--status-failed)' },
  ].map((r) => ({ ...r, count: store.statusCount(r.key) })).filter((r) => r.count > 0 || r.key !== 'failed'))
  const doneCount = $derived(store.statusCount('enhanced') + store.statusCount('organized'))
  const pct = $derived(total ? Math.round((doneCount / total) * 100) : 0)
  const undated = $derived(store.counts.years.find((y) => y.value === 'unknown')?.count ?? 0)
</script>

<div class="flex h-full flex-col gap-5 overflow-y-auto px-3 py-4">
  <section>
    <h3 class="eyebrow mb-2">Library progress</h3>
    <div class="flex items-baseline gap-2">
      <span class="font-serif text-[32px] leading-none text-ink tabular-nums">{pct}%</span>
      <span class="text-xs text-ink-dim">filed or restored</span>
    </div>
    <div class="mt-3 flex h-2 overflow-hidden rounded-full bg-surface-2">
      {#each rows as r (r.key)}
        {#if r.key !== 'new' && total}
          <div style="width: {(r.count / total) * 100}%; background: {r.color};"></div>
        {/if}
      {/each}
    </div>
    <ul class="mt-3 flex flex-col gap-1.5">
      {#each rows as r (r.key)}
        <li>
          <button
            class="flex w-full items-center gap-2 rounded-[4px] text-xs text-ink-dim hover:text-ink"
            onclick={() => store.setFilter({ status: r.key })}
          >
            <span class="h-2 w-2 rounded-full" style="background: {r.color};"></span>
            {r.label}
            <span class="ml-auto text-ink tabular-nums">{r.count}</span>
          </button>
        </li>
      {/each}
    </ul>
  </section>

  {#if undated > 0}
    <button
      class="flex items-start gap-2.5 rounded-[8px] border border-warning/30 bg-warning-soft p-3 text-left"
      onclick={() => store.setFilter({ year_unknown: true, year: null, month: null })}
    >
      <Icon name="calendarOff" size={16} class="mt-0.5 text-warning" />
      <span>
        <span class="block text-xs font-medium text-ink">{undated} undated photo{undated === 1 ? '' : 's'}</span>
        <span class="mt-0.5 block text-[11px] leading-snug text-ink-dim">Add a year so they're filed in the right place.</span>
      </span>
    </button>
  {/if}

  <section class="rounded-[8px] border border-line bg-base p-3">
    <p class="text-xs text-ink-dim">Select a photo to see its details. Select several to edit them together.</p>
    <div class="mt-3 grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5 text-[11px] text-ink-faint">
      <span><span class="kbd">Click</span></span><span>Select</span>
      <span><span class="kbd">Ctrl</span> <span class="kbd">Click</span></span><span>Add to selection</span>
      <span><span class="kbd">Shift</span> <span class="kbd">Click</span></span><span>Select range</span>
      <span><span class="kbd">Enter</span></span><span>Open large</span>
    </div>
  </section>
</div>

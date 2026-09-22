<script lang="ts">
  import { store } from '../store.svelte'
  import { MONTHS } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Spinner from '../ui/Spinner.svelte'

  const f = $derived(store.filters)
  const grid = $derived(store.view !== 'duplicates')

  const COLLECTIONS = [
    { status: 'new', label: 'To restore', icon: 'inbox', color: 'var(--status-new)' },
    { status: 'organized', label: 'Organized', icon: 'folderTree', color: 'var(--status-organized)' },
    { status: 'enhanced', label: 'Restored', icon: 'sparkles', color: 'var(--status-enhanced)' },
    { status: 'failed', label: 'Failed', icon: 'alertTriangle', color: 'var(--status-failed)' },
  ]

  const allActive = $derived(grid && f.status == null && f.folder == null && f.year == null && !f.year_unknown)

  function folderName(p: string) {
    return p.replace(/[/\\]+$/, '').split(/[/\\]/).pop() || p
  }

  function showAll() {
    store.view = 'grid'
    store.setFilter({ status: null, folder: null, year: null, month: null, year_unknown: false })
  }

  function pickYear(value: string) {
    if (value === 'unknown') store.setFilter({ year_unknown: !f.year_unknown, year: null, month: null })
    else {
      const y = Number(value)
      store.setFilter({ year: f.year === y ? null : y, year_unknown: false, month: null })
    }
  }
</script>

{#snippet item(label: string, count: number | null, active: boolean, onclick: () => void, icon?: string, dot?: string, indent = false)}
  <button
    type="button"
    {onclick}
    class="group flex h-7 w-full items-center gap-2 rounded-[5px] pr-2 text-left text-[13px] transition-colors {indent ? 'pl-8' : 'pl-2'} {active
      ? 'bg-brand-soft text-brand'
      : 'text-ink-dim hover:bg-surface-2 hover:text-ink'}"
  >
    {#if icon}
      <Icon name={icon} size={15} class={active ? 'text-brand' : 'text-ink-faint group-hover:text-ink-dim'} />
    {/if}
    {#if dot}
      <span class="h-2 w-2 shrink-0 rounded-full" style="background: {dot};"></span>
    {/if}
    <span class="min-w-0 flex-1 truncate">{label}</span>
    {#if count != null}
      <span class="text-[11px] tabular-nums {active ? 'text-brand/80' : 'text-ink-faint'}">{count}</span>
    {/if}
  </button>
{/snippet}

<aside class="flex h-full w-[232px] shrink-0 flex-col border-r border-line bg-surface">
  <nav class="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto px-2 py-3">
    <section class="flex flex-col gap-px">
      <h3 class="eyebrow mb-1 px-2">Library</h3>
      {@render item('All photos', store.libraryTotal, allActive, showAll, 'image')}
      {#each COLLECTIONS as c (c.status)}
        {@const n = store.statusCount(c.status)}
        {#if n > 0 || c.status !== 'failed'}
          {@render item(
            c.label,
            n,
            grid && f.status === c.status,
            () => store.setFilter({ status: f.status === c.status ? null : c.status }),
            undefined,
            c.color,
          )}
        {/if}
      {/each}
    </section>

    {#if store.counts.folders.length}
      <section class="flex flex-col gap-px">
        <h3 class="eyebrow mb-1 px-2">Folders</h3>
        {#each store.counts.folders as fl (fl.value)}
          {@render item(
            fl.value,
            fl.count,
            grid && f.folder === fl.value,
            () => store.setFilter({ folder: f.folder === fl.value ? null : fl.value }),
            'folder',
          )}
        {/each}
      </section>
    {/if}

    {#if store.counts.years.length}
      <section class="flex flex-col gap-px">
        <h3 class="eyebrow mb-1 px-2">Timeline</h3>
        {#each store.counts.years as y (y.value)}
          {#if y.value === 'unknown'}
            {@render item('Undated', y.count, grid && f.year_unknown, () => pickYear('unknown'), 'calendarOff')}
          {:else}
            {@const open = f.year === Number(y.value)}
            {@render item(y.value, y.count, grid && open && f.month == null, () => pickYear(y.value), 'calendar')}
            {#if open}
              {#each store.counts.months as m (m.value)}
                {@render item(
                  MONTHS[Number(m.value)] ?? m.value,
                  m.count,
                  grid && f.month === Number(m.value),
                  () => store.setFilter({ month: f.month === Number(m.value) ? null : Number(m.value) }),
                  undefined,
                  undefined,
                  true,
                )}
              {/each}
            {/if}
          {/if}
        {/each}
      </section>
    {/if}

    <section class="flex flex-col gap-px">
      <h3 class="eyebrow mb-1 px-2">Tools</h3>
      {@render item('Find duplicates', null, store.view === 'duplicates', () => (store.view = 'duplicates'), 'copy')}
    </section>
  </nav>

  {#if store.settings?.source_dir}
    <div class="border-t border-line p-2">
      <div class="flex items-center gap-2 rounded-[6px] px-2 py-1.5" title={store.settings.source_dir}>
        <Icon name="folderOpen" size={15} class="text-brand" />
        <div class="min-w-0 flex-1">
          <div class="truncate text-xs font-medium text-ink">{folderName(store.settings.source_dir)}</div>
          <div class="truncate text-[11px] text-ink-faint">{store.settings.source_dir}</div>
        </div>
      </div>
      <div class="mt-1 grid grid-cols-2 gap-1.5">
        <button class="btn-sm" onclick={() => store.scan()} disabled={store.scanning}>
          {#if store.scanning}<Spinner size={11} />Scanning{:else}<Icon name="refresh" size={13} />Rescan{/if}
        </button>
        <button class="btn-sm" onclick={() => store.pickFolder()}>
          <Icon name="folder" size={13} />Change
        </button>
      </div>
    </div>
  {/if}
</aside>

<script lang="ts">
  import { store, type Place } from '../store.svelte'
  import { MONTHS, STATUS } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Spinner from '../ui/Spinner.svelte'

  const f = $derived(store.filters)
  const grid = $derived(store.view !== 'duplicates')

  function folderName(p: string | null | undefined) {
    if (!p) return ''
    return p.replace(/[/\\]+$/, '').split(/[/\\]/).pop() || p
  }

  const here = (p: Place) =>
    grid &&
    (f.status ?? null) === (p.status ?? null) &&
    (f.folder ?? null) === (p.folder ?? null) &&
    (f.year ?? null) === (p.year ?? null) &&
    (f.month ?? null) === (p.month ?? null) &&
    f.year_unknown === (p.year_unknown ?? false)
</script>

{#snippet item(label: string, count: number | null, place: Place, icon?: string, dot?: string, indent = false)}
  {@const active = here(place)}
  <button
    type="button"
    onclick={() => store.show(place)}
    aria-current={active ? 'page' : undefined}
    class="group flex h-7 w-full items-center gap-2 rounded-[5px] pr-2 text-left text-[13px] transition-colors {indent ? 'pl-8' : 'pl-2'} {active
      ? 'bg-brand-soft text-brand'
      : 'text-ink-dim hover:bg-surface-2 hover:text-ink'}"
  >
    {#if icon}
      <Icon name={icon} size={15} class={active ? 'text-brand' : 'text-ink-faint group-hover:text-ink-dim'} />
    {/if}
    {#if dot}
      <span class="mx-[3.5px] h-2 w-2 shrink-0 rounded-full" style="background: {dot};"></span>
    {/if}
    <span class="min-w-0 flex-1 truncate">{label}</span>
    {#if count != null}
      <span class="text-[11px] tabular-nums {active ? 'text-brand/80' : 'text-ink-faint'}">{count}</span>
    {/if}
  </button>
{/snippet}

<aside class="flex h-full w-[224px] shrink-0 flex-col border-r border-line bg-surface">
  <nav class="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto px-2 py-3">
    <section class="flex flex-col gap-px">
      {@render item('All photos', store.libraryTotal, {}, 'image')}
      {@render item('Inbox', store.statusCount('new'), { status: 'new' }, 'inbox')}
      {#if store.undated > 0}
        {@render item('Needs a date', store.undated, { year_unknown: true }, 'calendarOff')}
      {/if}
      {#if store.statusCount('failed') > 0}
        {@render item('Failed', store.statusCount('failed'), { status: 'failed' }, 'alertTriangle')}
      {/if}
    </section>

    <section class="flex flex-col gap-px">
      <h3 class="eyebrow mb-1 px-2">Archive</h3>
      {@render item('Filed', store.statusCount('organized'), { status: 'organized' }, undefined, STATUS.organized.color)}
      {@render item('Restored', store.statusCount('restored'), { status: 'restored' }, undefined, STATUS.restored.color)}
      {#each store.counts.folders as fl (fl.value)}
        {@render item(fl.value, fl.count, { folder: fl.value }, 'folder')}
      {/each}
    </section>

    {#if store.counts.years.some((y) => y.value !== 'unknown')}
      <section class="flex flex-col gap-px">
        <h3 class="eyebrow mb-1 px-2">Years</h3>
        {#each store.counts.years as y (y.value)}
          {#if y.value !== 'unknown'}
            {@const year = Number(y.value)}
            {@render item(y.value, y.count, { year }, 'calendar')}
            {#if f.year === year && grid}
              {#each store.counts.months as m (m.value)}
                {@render item(MONTHS[Number(m.value)] ?? m.value, m.count, { year, month: Number(m.value) }, undefined, undefined, true)}
              {/each}
            {/if}
          {/if}
        {/each}
      </section>
    {/if}
  </nav>

  <div class="flex flex-col gap-px border-t border-line p-2">
    <button
      type="button"
      onclick={() => (store.view = 'duplicates')}
      class="group mb-1 flex h-7 w-full items-center gap-2 rounded-[5px] px-2 text-left text-[13px] transition-colors {store.view === 'duplicates'
        ? 'bg-brand-soft text-brand'
        : 'text-ink-dim hover:bg-surface-2 hover:text-ink'}"
    >
      <Icon name="copy" size={15} class={store.view === 'duplicates' ? 'text-brand' : 'text-ink-faint group-hover:text-ink-dim'} />
      Find duplicates
    </button>

    <div class="flex items-center gap-2 py-1 pr-0.5 pl-2" title={store.settings?.source_dir ?? ''}>
      <div class="min-w-0 flex-1">
        <div class="eyebrow !text-[10px]">Scans</div>
        <div class="truncate text-xs text-ink">{folderName(store.settings?.source_dir)}</div>
      </div>
      <button class="icon-btn" title="Look for new scans" aria-label="Rescan" onclick={() => store.scan()} disabled={store.scanning}>
        {#if store.scanning}<Spinner size={12} />{:else}<Icon name="refresh" size={14} />{/if}
      </button>
      <button class="icon-btn" title="Choose another scans folder" aria-label="Change scans folder" onclick={() => store.pickFolder()}>
        <Icon name="folder" size={14} />
      </button>
    </div>
    <div class="flex items-center gap-2 py-1 pr-0.5 pl-2" title={store.settings?.output_dir ?? ''}>
      <div class="min-w-0 flex-1">
        <div class="eyebrow !text-[10px]">Archive</div>
        <div class="truncate text-xs text-ink">{folderName(store.settings?.output_dir)}</div>
      </div>
      <button class="icon-btn" title="Open the archive folder" aria-label="Open archive folder" onclick={() => store.openOutput()}>
        <Icon name="external" size={14} />
      </button>
    </div>
  </div>
</aside>

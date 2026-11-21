<script lang="ts">
  import { store } from '../store.svelte'
  import { STATUS, plural } from '../types'
  import Icon from '../ui/Icon.svelte'

  const total = $derived(store.libraryTotal)
  const fresh = $derived(store.statusCount('new'))
  const failed = $derived(store.statusCount('failed'))
  const bars = $derived(
    ['restored', 'organized', 'failed'].map((key) => ({ key, ...STATUS[key], count: store.statusCount(key) })),
  )
  const inArchive = $derived(store.statusCount('restored') + store.statusCount('organized'))
  const pct = $derived(total ? Math.round((inArchive / total) * 100) : 0)

  // The one or two things worth doing next, most useful first.
  const next = $derived.by(() => {
    const out: { icon: string; tone: string; title: string; text: string; action: string; run: () => void }[] = []
    if (fresh > 0) {
      out.push({
        icon: 'inbox',
        tone: 'text-brand',
        title: `${plural(fresh, 'new scan')} in the inbox`,
        text: 'Clean them up and copy them into the archive.',
        action: 'Restore them',
        run: async () => {
          await store.show({ status: 'new' })
          store.openInspector('restore')
        },
      })
    }
    if (store.undated > 0) {
      out.push({
        icon: 'calendarOff',
        tone: 'text-warning',
        title: `${plural(store.undated, 'photo')} without a date`,
        text: 'Give them a year so they land in the right folder.',
        action: 'Add dates',
        run: async () => {
          await store.show({ year_unknown: true })
          store.editDate()
        },
      })
    }
    if (failed > 0) {
      out.push({
        icon: 'alertTriangle',
        tone: 'text-danger',
        title: `${plural(failed, 'photo')} failed`,
        text: 'See what went wrong and try again.',
        action: 'Review',
        run: () => store.show({ status: 'failed' }),
      })
    }
    return out
  })
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto px-4 py-4">
  <section>
    <h3 class="eyebrow mb-2">Archive</h3>
    <div class="flex items-baseline gap-2">
      <span class="font-serif text-[34px] leading-none text-ink tabular-nums">{inArchive}</span>
      <span class="text-xs text-ink-dim">of {plural(total, 'photo')} · {pct}%</span>
    </div>
    <div class="mt-3 flex h-1.5 gap-px overflow-hidden rounded-full bg-surface-2">
      {#each bars as b (b.key)}
        {#if b.count && total}
          <div style="width: {(b.count / total) * 100}%; background: {b.color};"></div>
        {/if}
      {/each}
    </div>
    <ul class="mt-3 flex flex-col gap-1">
      {#each bars as b (b.key)}
        {#if b.count || b.key !== 'failed'}
          <li>
            <button class="flex w-full items-center gap-2 text-xs text-ink-dim hover:text-ink" title={b.hint} onclick={() => store.show({ status: b.key })}>
              <span class="h-2 w-2 rounded-full" style="background: {b.color};"></span>
              {b.label}
              <span class="ml-auto text-ink tabular-nums">{b.count}</span>
            </button>
          </li>
        {/if}
      {/each}
    </ul>
  </section>

  <section>
    <h3 class="eyebrow mb-2">Next</h3>
    {#if next.length === 0}
      <div class="flex items-start gap-2.5 rounded-[8px] border border-success/25 bg-success-soft p-3">
        <Icon name="checkCircle" size={16} class="mt-0.5 text-success" />
        <div>
          <p class="text-xs font-medium text-ink">All caught up</p>
          <p class="mt-0.5 text-[11px] leading-snug text-ink-dim">Every scan is dated and in the archive.</p>
          <button class="btn-sm mt-2" onclick={() => store.openOutput()}><Icon name="external" size={13} />Open archive folder</button>
        </div>
      </div>
    {:else}
      <div class="flex flex-col gap-2">
        {#each next as n, i (n.title)}
          <div class="flex items-start gap-2.5 rounded-[8px] border border-line bg-base p-3">
            <Icon name={n.icon} size={16} class="mt-0.5 {n.tone}" />
            <div class="min-w-0 flex-1">
              <p class="text-xs font-medium text-ink">{n.title}</p>
              <p class="mt-0.5 text-[11px] leading-snug text-ink-dim">{n.text}</p>
              <button class="{i === 0 ? 'btn h-7 text-xs shadow-none' : 'btn-sm'} mt-2.5" onclick={n.run}>
                {n.action}<Icon name="arrowRight" size={13} />
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section class="mt-auto">
    <div class="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5 text-[11px] text-ink-faint">
      <span><span class="kbd">Enter</span></span><span>Open a photo large</span>
      <span><span class="kbd">D</span></span><span>Type its date</span>
      <span><span class="kbd">.</span></span><span>Same date as the last one</span>
      <span><span class="kbd">R</span></span><span>Restore</span>
      <span><span class="kbd">?</span></span><span>All shortcuts</span>
    </div>
  </section>
</div>

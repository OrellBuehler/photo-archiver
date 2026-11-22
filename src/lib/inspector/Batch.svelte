<script lang="ts">
  import { store } from '../store.svelte'
  import { MONTHS, dateLabel } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Thumb from '../ui/Thumb.svelte'

  let form = $state({ title: '', year: '', month: '', folder: '' })

  const ids = $derived([...store.selected])
  const yearValid = $derived(form.year.trim() === '' || /^(1[89]|20)\d{2}$/.test(form.year.trim()))
  const hasMeta = $derived(form.title.trim() !== '' || form.year.trim() !== '' || form.month !== '')
  const canApply = $derived(yearValid && (hasMeta || form.folder.trim() !== ''))
  const last = $derived(store.lastMeta)
  const lastLabel = $derived(
    last ? [last.year != null ? dateLabel(last) : null, last.folder].filter(Boolean).join(' · ') : '',
  )

  async function apply() {
    if (!canApply) return
    const folder = form.folder.trim()
    if (hasMeta) {
      await store.editTargets(
        form.year.trim() ? Number(form.year) : null,
        form.month ? Number(form.month) : null,
        form.title.trim() || null,
      )
    }
    if (folder) await store.setFolderTargets(folder)
    form = { title: '', year: '', month: '', folder: '' }
  }
</script>

<div class="flex h-full flex-col overflow-y-auto">
  <div class="flex items-center gap-3 px-3 pt-4 pb-3">
    <div class="relative h-14 w-16 shrink-0">
      {#each ids.slice(0, 3) as id, i (id)}
        <div
          class="absolute h-12 w-12 overflow-hidden rounded-[5px] border-2 border-surface bg-surface-2 shadow-[var(--shadow-sm)]"
          style="left: {i * 7}px; top: {i * 3}px; transform: rotate({(i - 1) * 5}deg); z-index: {3 - i};"
        >
          <Thumb {id} />
        </div>
      {/each}
    </div>
    <div class="min-w-0">
      <p class="text-[15px] font-semibold text-ink">{ids.length} photos</p>
      <button class="text-xs text-ink-dim hover:text-ink" onclick={() => store.clearSelection()}>Clear selection</button>
    </div>
  </div>

  <div class="flex gap-1 border-b border-line px-3 pb-3">
    <button class="btn-sm flex-1" onclick={() => store.rotateTargets(false)}><Icon name="rotateCcw" size={13} />Left</button>
    <button class="btn-sm flex-1" onclick={() => store.rotateTargets(true)}><Icon name="rotateCw" size={13} />Right</button>
    <button class="btn-sm flex-1" onclick={() => store.openInspector('restore')}><Icon name="wand" size={13} />Restore</button>
  </div>

  <section class="flex flex-col gap-2.5 border-b border-line px-3 py-4">
    <div>
      <h3 class="eyebrow">Date, title and folder</h3>
      <p class="mt-0.5 text-[11px] text-ink-faint">Only what you fill in changes. Filed copies move along.</p>
    </div>
    <div class="grid grid-cols-[84px_1fr] gap-2">
      <input
        class="input tabular-nums"
        class:!border-danger={!yearValid}
        bind:value={form.year}
        placeholder="Year"
        inputmode="numeric"
        maxlength="4"
        onkeydown={(e) => e.key === 'Enter' && apply()}
      />
      <select class="input" bind:value={form.month}>
        <option value="">Month —</option>
        {#each MONTHS.slice(1) as name, i (i)}
          <option value={String(i + 1)}>{name}</option>
        {/each}
      </select>
    </div>
    <input class="input" bind:value={form.title} placeholder="Title" onkeydown={(e) => e.key === 'Enter' && apply()} />
    <input
      class="input"
      bind:value={form.folder}
      placeholder="Folder — e.g. Hochzeit 1978"
      list="batch-folder-suggestions"
      onkeydown={(e) => e.key === 'Enter' && apply()}
    />
    <datalist id="batch-folder-suggestions">
      {#each store.counts.folders as fl (fl.value)}<option value={fl.value}></option>{/each}
    </datalist>
    <button class="btn" disabled={!canApply} onclick={apply}>
      <Icon name="check" size={14} />Apply to {ids.length}
    </button>
    {#if lastLabel}
      <button class="btn-sm justify-start" onclick={() => store.applyLast()}>
        <Icon name="repeat" size={13} />
        <span class="truncate">Same as last: {lastLabel}</span>
        <span class="kbd ml-auto">.</span>
      </button>
    {/if}
  </section>

  <section class="flex flex-col gap-1.5 px-3 py-4">
    <button class="btn-sm justify-start" onclick={() => store.setFolderTargets(null)}>
      <Icon name="folder" size={13} />Take out of their folder
    </button>
    <button class="btn-sm justify-start" onclick={() => store.resetTargets()}>
      <Icon name="history" size={13} />Start over from the scans
    </button>
    <button class="btn-danger justify-start" onclick={() => store.deleteTargets()}>
      <Icon name="trash" size={13} />Remove from library
    </button>
  </section>
</div>

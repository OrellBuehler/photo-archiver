<script lang="ts">
  import { store } from '../store.svelte'
  import { MONTHS } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Thumb from '../ui/Thumb.svelte'

  let form = $state({ title: '', year: '', month: '' })
  let folder = $state('')

  const ids = $derived([...store.selected])
  const yearValid = $derived(form.year.trim() === '' || /^\d{4}$/.test(form.year.trim()))
  const canApply = $derived(yearValid && (form.title.trim() !== '' || form.year.trim() !== '' || form.month !== ''))

  async function apply() {
    if (!canApply) return
    await store.editTargets(
      form.year.trim() ? Number(form.year) : null,
      form.month ? Number(form.month) : null,
      form.title.trim() || null,
    )
    form = { title: '', year: '', month: '' }
  }

  async function move(clear = false) {
    if (!clear && !folder.trim()) return
    await store.setFolderTargets(clear ? null : folder.trim())
    folder = ''
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
    <button class="btn-danger flex-1" onclick={() => store.deleteTargets()}><Icon name="trash" size={13} />Delete</button>
  </div>

  <section class="flex flex-col gap-2.5 border-b border-line px-3 py-4">
    <div>
      <h3 class="eyebrow">Set date & title</h3>
      <p class="mt-0.5 text-[11px] text-ink-faint">Only fields you fill in are changed.</p>
    </div>
    <div class="grid grid-cols-[88px_1fr] gap-2">
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
    <button class="btn" disabled={!canApply} onclick={apply}>
      <Icon name="check" size={14} />Apply to {ids.length}
    </button>
  </section>

  <section class="flex flex-col gap-2.5 px-3 py-4">
    <div>
      <h3 class="eyebrow">Group into folder</h3>
      <p class="mt-0.5 text-[11px] text-ink-faint">Keeps an event together, e.g. “Wedding 1978”. Applied when next organized.</p>
    </div>
    <input
      class="input"
      bind:value={folder}
      placeholder="Folder name"
      list="batch-folder-suggestions"
      onkeydown={(e) => e.key === 'Enter' && move()}
    />
    <datalist id="batch-folder-suggestions">
      {#each store.counts.folders as fl (fl.value)}<option value={fl.value}></option>{/each}
    </datalist>
    <div class="flex gap-2">
      <button class="btn-sm h-8 flex-1" disabled={!folder.trim()} onclick={() => move()}>
        <Icon name="folder" size={13} />Move {ids.length}
      </button>
      <button class="btn-sm h-8" onclick={() => move(true)}>Remove from folder</button>
    </div>
  </section>
</div>

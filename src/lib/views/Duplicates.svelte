<script lang="ts">
  import { bulkDelete, findDuplicates, scanDuplicates } from '../api'
  import { store } from '../store.svelte'
  import type { DuplicateGroup } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Thumb from '../ui/Thumb.svelte'
  import Spinner from '../ui/Spinner.svelte'

  let groups = $state<DuplicateGroup[] | null>(null)
  let threshold = $state(6)
  let busy = $state<'hash' | 'find' | null>(null)
  let marked = $state<Set<number>>(new Set())

  async function scan() {
    busy = 'hash'
    try {
      await scanDuplicates()
      await find()
    } catch (e) {
      store.fail(e)
    } finally {
      busy = null
    }
  }

  async function find() {
    busy = busy ?? 'find'
    try {
      groups = await findDuplicates(threshold)
      marked = new Set()
    } catch (e) {
      store.fail(e)
    } finally {
      if (busy === 'find') busy = null
    }
  }

  function toggle(id: number) {
    const n = new Set(marked)
    if (n.has(id)) n.delete(id)
    else n.add(id)
    marked = n
  }

  function keepFirst(g: DuplicateGroup) {
    const n = new Set(marked)
    g.image_ids.forEach((id, i) => (i === 0 ? n.delete(id) : n.add(id)))
    marked = n
  }

  async function del() {
    if (marked.size === 0) return
    const ok = await store.confirm({
      title: `Delete ${marked.size} duplicate${marked.size === 1 ? '' : 's'}?`,
      message: 'They are removed from the library. The original scans on disk are not touched.',
      confirmLabel: 'Delete',
      danger: true,
    })
    if (!ok) return
    try {
      await bulkDelete([...marked])
      store.toast(`Deleted ${marked.size} duplicate${marked.size === 1 ? '' : 's'}`)
      await store.refresh()
      await find()
    } catch (e) {
      store.fail(e)
    }
  }

  const strictness = $derived(threshold <= 3 ? 'Near-identical' : threshold <= 8 ? 'Very similar' : threshold <= 14 ? 'Similar' : 'Loose')
</script>

<div class="flex h-full min-w-0 flex-col">
  <div class="flex h-11 shrink-0 items-center gap-3 border-b border-line px-3">
    <h1 class="text-[13px] font-semibold text-ink">Duplicates</h1>
    {#if groups}
      <span class="text-xs text-ink-faint tabular-nums">{groups.length} group{groups.length === 1 ? '' : 's'}</span>
    {/if}
    <div class="ml-auto flex items-center gap-3">
      <label class="flex items-center gap-2 text-xs text-ink-dim">
        Match
        <input
          type="range"
          min="0"
          max="20"
          bind:value={threshold}
          onchange={() => groups && find()}
          class="w-28 accent-[var(--color-brand)]"
        />
        <span class="w-24 text-ink">{strictness}</span>
      </label>
      {#if marked.size}
        <button class="btn-danger" onclick={del}>
          <Icon name="trash" size={13} />Delete {marked.size}
        </button>
      {/if}
      <button class="btn-sm" onclick={scan} disabled={busy != null}>
        {#if busy === 'hash'}<Spinner size={11} />Analysing…{:else}<Icon name="refresh" size={13} />{groups ? 'Re-scan' : 'Scan library'}{/if}
      </button>
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-4">
    {#if groups == null}
      <div class="flex h-full flex-col items-center justify-center gap-2 text-center">
        <div class="mb-1 grid h-14 w-14 place-items-center rounded-[12px] bg-brand-soft text-brand">
          <Icon name="copy" size={26} stroke={1.5} />
        </div>
        <p class="font-serif text-xl text-ink">Find scanned-twice photos</p>
        <p class="max-w-sm text-ink-dim">
          Compares every photo by how it looks, not its file name — so rescans, crops and slightly rotated
          copies are caught too.
        </p>
        <button class="btn mt-3" onclick={scan} disabled={busy != null}>
          {#if busy}<Spinner size={13} color="var(--color-on-brand)" />Analysing library…{:else}<Icon name="search" size={15} />Scan library{/if}
        </button>
      </div>
    {:else if groups.length === 0}
      <div class="flex h-full flex-col items-center justify-center gap-2 text-center">
        <Icon name="checkCircle" size={28} stroke={1.5} class="text-success" />
        <p class="font-serif text-xl text-ink">No duplicates</p>
        <p class="text-ink-dim">Nothing looks alike at this match level. Loosen it to dig deeper.</p>
      </div>
    {:else}
      <p class="mb-4 text-xs text-ink-faint">Click photos to mark them for deletion. The unmarked ones stay.</p>
      <div class="flex flex-col gap-3">
        {#each groups as g, i (i)}
          <section class="rounded-[8px] border border-line bg-surface p-3">
            <div class="mb-2.5 flex items-center gap-2">
              <span class="text-xs font-medium text-ink">{g.image_ids.length} alike</span>
              <span class="text-[11px] text-ink-faint">distance {g.distance}</span>
              <button class="ml-auto text-xs text-ink-dim hover:text-ink" onclick={() => keepFirst(g)}>Keep first, mark rest</button>
            </div>
            <div class="flex flex-wrap gap-2">
              {#each g.image_ids as id (id)}
                {@const m = marked.has(id)}
                <button
                  type="button"
                  onclick={() => toggle(id)}
                  class="relative h-32 w-32 overflow-hidden rounded-[6px] bg-surface-2"
                  style="box-shadow: {m ? 'inset 0 0 0 2px var(--color-danger)' : 'inset 0 0 0 1px rgba(255,255,255,0.06)'};"
                >
                  <div class="h-full w-full transition-opacity" class:opacity-40={m}><Thumb {id} /></div>
                  {#if m}
                    <span class="absolute inset-0 grid place-items-center">
                      <span class="grid h-8 w-8 place-items-center rounded-full bg-danger text-white"><Icon name="trash" size={15} /></span>
                    </span>
                  {/if}
                </button>
              {/each}
            </div>
          </section>
        {/each}
      </div>
    {/if}
  </div>
</div>

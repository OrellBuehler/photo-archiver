<script lang="ts">
  import { bulkDelete, findDuplicates, scanDuplicates } from '../api'
  import { store } from '../store.svelte'
  import { dateLabel, plural, type DuplicateGroup, type Image } from '../types'
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

  // Each group comes best-quality first; keep that one.
  function keepBest(list: DuplicateGroup[]) {
    const n = new Set(marked)
    for (const g of list) g.images.forEach((img, i) => (i === 0 ? n.delete(img.id) : n.add(img.id)))
    marked = n
  }

  async function del() {
    if (marked.size === 0) return
    const ok = await store.confirm({
      title: `Remove ${plural(marked.size, 'duplicate')}?`,
      message: 'They leave the library and their archive copies are deleted. The original scans are not touched.',
      confirmLabel: 'Remove',
      danger: true,
    })
    if (!ok) return
    try {
      await bulkDelete([...marked])
      store.toast(`Removed ${plural(marked.size, 'duplicate')}`)
      store.clearSelection()
      await store.refresh()
      await find()
    } catch (e) {
      store.fail(e)
    }
  }

  const strictness = $derived(threshold <= 3 ? 'Near-identical' : threshold <= 8 ? 'Very similar' : threshold <= 14 ? 'Similar' : 'Loose')
  const megapixels = (i: Image) => (i.width && i.height ? `${((i.width * i.height) / 1e6).toFixed(1)} MP` : '')
</script>

<div class="flex h-full min-w-0 flex-col">
  <div class="flex h-11 shrink-0 items-center gap-3 border-b border-line pr-2 pl-4">
    <h1 class="text-[13px] font-semibold text-ink">Duplicates</h1>
    {#if groups}
      <span class="text-xs whitespace-nowrap text-ink-faint tabular-nums">{plural(groups.length, 'group')}</span>
    {/if}
    <div class="ml-auto flex items-center gap-2">
      <label class="flex items-center gap-2 text-xs text-ink-dim">
        <input
          type="range"
          min="0"
          max="20"
          bind:value={threshold}
          onchange={() => groups && find()}
          class="w-24 accent-[var(--color-brand)]"
          aria-label="How alike photos must be"
        />
        <span class="w-[88px] whitespace-nowrap text-ink">{strictness}</span>
      </label>
      <button class="btn-sm" onclick={scan} disabled={busy != null}>
        {#if busy === 'hash'}<Spinner size={11} />Comparing…{:else}<Icon name="refresh" size={13} />{groups ? 'Scan again' : 'Scan library'}{/if}
      </button>
      {#if groups?.length}
        <button class="btn-sm" onclick={() => groups && keepBest(groups)}>Keep best of each</button>
        <button class="btn-danger-solid h-7 text-xs whitespace-nowrap" disabled={marked.size === 0} class:opacity-40={marked.size === 0} onclick={del}>
          <Icon name="trash" size={13} />Remove {marked.size || ''}
        </button>
      {/if}
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-4">
    {#if groups == null}
      <div class="flex h-full flex-col items-center justify-center gap-2 text-center">
        <div class="mb-1 grid h-14 w-14 place-items-center rounded-[12px] bg-brand-soft text-brand">
          <Icon name="copy" size={26} stroke={1.5} />
        </div>
        <p class="font-serif text-xl text-ink">Find photos scanned twice</p>
        <p class="max-w-sm text-ink-dim">
          Compares every photo by how it looks, not by file name — so rescans and slightly cropped copies are caught too.
        </p>
        <button class="btn mt-3" onclick={scan} disabled={busy != null}>
          {#if busy}<Spinner size={13} color="var(--color-on-brand)" />Comparing photos…{:else}<Icon name="search" size={15} />Scan library{/if}
        </button>
      </div>
    {:else if groups.length === 0}
      <div class="flex h-full flex-col items-center justify-center gap-2 text-center">
        <Icon name="checkCircle" size={28} stroke={1.5} class="text-success" />
        <p class="font-serif text-xl text-ink">No duplicates</p>
        <p class="text-ink-dim">Nothing looks alike at this level. Move the slider right to look harder.</p>
      </div>
    {:else}
      <p class="mb-3 text-xs text-ink-faint">Click the copies you don't want. The sharpest scan of each group comes first.</p>
      <div class="flex flex-col gap-3">
        {#each groups as g (g.images[0]?.id)}
          <section class="rounded-[8px] border border-line bg-surface p-3">
            <div class="flex flex-wrap gap-3">
              {#each g.images as img, i (img.id)}
                {@const m = marked.has(img.id)}
                <button type="button" onclick={() => toggle(img.id)} aria-pressed={m} class="flex w-36 flex-col gap-1.5 text-left">
                  <span
                    class="relative block h-36 w-36 overflow-hidden rounded-[6px] bg-base p-1"
                    style="box-shadow: {m ? 'inset 0 0 0 2px var(--color-danger)' : 'inset 0 0 0 1px rgba(255,255,255,0.06)'};"
                  >
                    <span class="block h-full w-full transition-opacity" class:opacity-35={m}><Thumb id={img.id} fit="contain" /></span>
                    {#if m}
                      <span class="absolute inset-0 grid place-items-center">
                        <span class="grid h-8 w-8 place-items-center rounded-full bg-danger text-white"><Icon name="trash" size={15} /></span>
                      </span>
                    {:else if i === 0}
                      <span class="absolute top-1.5 left-1.5 rounded-full bg-success px-1.5 py-px text-[10px] font-semibold text-[#0f1f10]">Best</span>
                    {/if}
                  </span>
                  <span class="px-0.5 text-[11px] leading-tight">
                    <span class="block truncate {m ? 'text-ink-faint line-through' : 'text-ink-dim'}" title={img.source_path}>{img.title ?? img.filename}</span>
                    <span class="block text-ink-faint tabular-nums">{[megapixels(img), dateLabel(img) ?? 'no date'].filter(Boolean).join(' · ')}</span>
                  </span>
                </button>
              {/each}
            </div>
          </section>
        {/each}
      </div>
    {/if}
  </div>
</div>

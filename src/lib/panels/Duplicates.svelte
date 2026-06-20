<script lang="ts">
  import { bulkDelete, findDuplicates, scanDuplicates } from '../api'
  import { thumbUrl } from '../thumbs'
  import { store } from '../store.svelte'
  import type { DuplicateGroup } from '../types'
  import Icon from '../ui/Icon.svelte'

  let groups = $state<DuplicateGroup[]>([])
  let threshold = $state(6)
  let scanning = $state(false)
  let loading = $state(false)
  let selected = $state<Set<number>>(new Set())

  async function scan() {
    scanning = true
    try {
      await scanDuplicates()
      await find()
    } finally {
      scanning = false
    }
  }

  async function find() {
    loading = true
    try {
      groups = await findDuplicates(threshold)
    } finally {
      loading = false
    }
  }

  function toggle(id: number) {
    const n = new Set(selected)
    if (n.has(id)) n.delete(id)
    else n.add(id)
    selected = n
  }

  async function del() {
    if (selected.size === 0) return
    if (!confirm(`Delete ${selected.size} photo(s)?`)) return
    await bulkDelete([...selected])
    selected = new Set()
    await store.refresh()
    await find()
  }
</script>

<div class="flex h-full flex-col bg-base text-sm">
  <div class="flex flex-wrap items-center gap-2 border-b border-line px-3 py-2 text-xs">
    <button class="btn-sm" onclick={scan} disabled={scanning}>
      <Icon name="copy" size={13} />
      {scanning ? 'Hashing…' : 'Compute hashes'}
    </button>
    <label class="flex items-center gap-1.5 text-ink-dim">
      Distance ≤
      <input
        type="range"
        min="0"
        max="20"
        bind:value={threshold}
        onchange={find}
        class="accent-[var(--color-brand)]"
      />
      <span class="w-5 tabular-nums text-ink">{threshold}</span>
    </label>
    <button class="btn-sm" onclick={find} disabled={loading}>
      <Icon name="search" size={13} />
      Find
    </button>
    {#if selected.size}
      <button class="btn-danger ml-auto" onclick={del}>
        <Icon name="trash" size={13} />
        Delete {selected.size}
      </button>
    {/if}
  </div>

  <div class="min-h-0 flex-1 overflow-auto p-3">
    {#if loading}
      <p class="text-ink-dim">Searching…</p>
    {:else if groups.length === 0}
      <div class="flex h-full flex-col items-center justify-center gap-1 text-center">
        <Icon name="copy" size={26} stroke={1.5} class="mb-1 text-ink-faint" />
        <p class="font-serif text-lg text-ink">No duplicates found</p>
        <p class="text-ink-dim">Run “Compute hashes” first, then “Find”.</p>
      </div>
    {:else}
      <div class="flex flex-col gap-4">
        {#each groups as group, i (i)}
          <div>
            <p class="eyebrow mb-1.5">
              {group.image_ids.length} similar · distance ≤ {group.distance}
            </p>
            <div class="grid grid-cols-[repeat(auto-fill,minmax(90px,1fr))] gap-2">
              {#each group.image_ids as id (id)}
                <button
                  type="button"
                  onclick={() => toggle(id)}
                  class={`relative aspect-square overflow-hidden rounded-[5px] border-2 bg-surface-2 ${
                    selected.has(id) ? 'border-focus' : 'border-line hover:border-[var(--border-strong)]'
                  }`}
                >
                  {#await thumbUrl(id) then url}
                    <img src={url} alt={`#${id}`} class="h-full w-full object-cover" />
                  {/await}
                  {#if selected.has(id)}
                    <span
                      class="absolute right-1 top-1 grid h-[18px] w-[18px] place-items-center rounded-[3px] text-white"
                      style="background: var(--color-focus);"
                    >
                      <svg width="11" height="11" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg>
                    </span>
                  {/if}
                </button>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

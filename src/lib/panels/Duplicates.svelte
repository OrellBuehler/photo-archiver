<script lang="ts">
  import { bulkDelete, findDuplicates, scanDuplicates } from '../api'
  import { thumbUrl } from '../thumbs'
  import { store } from '../store.svelte'
  import type { DuplicateGroup } from '../types'

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
      {scanning ? 'Hashing…' : 'Compute hashes'}
    </button>
    <label class="flex items-center gap-1.5 text-ink-dim">
      Distance ≤
      <input type="range" min="0" max="20" bind:value={threshold} onchange={find} class="accent-[var(--color-focus)]" />
      <span class="w-5 text-ink">{threshold}</span>
    </label>
    <button class="btn-sm" onclick={find} disabled={loading}>Find</button>
    {#if selected.size}
      <button class="btn-sm ml-auto" onclick={del}>Delete {selected.size}</button>
    {/if}
  </div>

  <div class="min-h-0 flex-1 overflow-auto p-3">
    {#if loading}
      <p class="text-ink-dim">Searching…</p>
    {:else if groups.length === 0}
      <p class="text-ink-dim">
        No duplicate groups. Run “Compute hashes” first, then “Find”.
      </p>
    {:else}
      <div class="flex flex-col gap-4">
        {#each groups as group, i (i)}
          <div>
            <p class="mb-1 text-xs text-ink-dim">
              {group.image_ids.length} similar · distance ≤ {group.distance}
            </p>
            <div class="grid grid-cols-[repeat(auto-fill,minmax(90px,1fr))] gap-2">
              {#each group.image_ids as id (id)}
                <button
                  type="button"
                  onclick={() => toggle(id)}
                  class="relative aspect-square overflow-hidden rounded border bg-surface"
                  class:border-line={!selected.has(id)}
                  class:border-focus={selected.has(id)}
                >
                  {#await thumbUrl(id) then url}
                    <img src={url} alt={`#${id}`} class="h-full w-full object-cover" />
                  {/await}
                  {#if selected.has(id)}
                    <span class="absolute right-1 top-1 grid h-4 w-4 place-items-center rounded-sm bg-focus text-[10px] text-white">✓</span>
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

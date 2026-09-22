<script lang="ts">
  import { untrack } from 'svelte'
  import { imageHistory, setFolder, updateImage } from '../api'
  import { store } from '../store.svelte'
  import { MONTHS, PIPELINE_STEPS, type HistoryItem, type Image } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Badge from '../ui/Badge.svelte'
  import Thumb from '../ui/Thumb.svelte'

  let { image }: { image: Image } = $props()

  const STATUS_LABEL: Record<string, string> = {
    new: 'To restore',
    organized: 'Organized',
    enhanced: 'Restored',
    failed: 'Failed',
  }

  const fromImage = () => ({
    title: image.title ?? '',
    year: image.year?.toString() ?? '',
    month: image.month?.toString() ?? '',
    folder: image.folder ?? '',
  })

  let form = $state(fromImage())
  let saved = $state(fromImage())
  let history = $state<HistoryItem[]>([])
  let saving = $state(false)

  const dirty = $derived(
    form.title !== saved.title || form.year !== saved.year || form.month !== saved.month || form.folder !== saved.folder,
  )
  const yearValid = $derived(form.year.trim() === '' || /^\d{4}$/.test(form.year.trim()))

  // Pick up external changes (batch edits, processing) unless mid-edit.
  $effect(() => {
    void image.updated_at
    void image.folder
    const next = fromImage()
    untrack(() => {
      if (!dirty) form = next
      saved = next
    })
  })

  $effect(() => {
    void store.thumbVersion
    void image.updated_at
    imageHistory(image.id).then((h) => (history = h)).catch(() => (history = []))
  })

  async function save() {
    if (!dirty || !yearValid || saving) return
    saving = true
    try {
      const metaChanged = form.title !== saved.title || form.year !== saved.year || form.month !== saved.month
      if (metaChanged) {
        await updateImage(
          image.id,
          form.year.trim() ? Number(form.year) : null,
          form.month ? Number(form.month) : null,
          form.title.trim() || null,
        )
      }
      if (form.folder !== saved.folder) await setFolder([image.id], form.folder.trim() || null)
      saved = { ...form }
      await store.refresh()
    } catch (e) {
      store.fail(e)
    } finally {
      saving = false
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault()
      save()
    } else if (e.key === 'Escape') {
      form = { ...saved }
      ;(e.currentTarget as HTMLElement).blur()
    }
  }

  function bytes(n: number | null) {
    if (n == null) return '—'
    if (n < 1024 * 1024) return `${Math.round(n / 1024)} KB`
    return `${(n / 1024 / 1024).toFixed(1)} MB`
  }

  const stepLabel = (k: string) => PIPELINE_STEPS.find((s) => s.key === k)?.label ?? k
  const when = (ts: string) => ts.replace('T', ' ').slice(0, 16)
</script>

<div class="flex h-full flex-col overflow-y-auto">
  <button
    class="group relative mx-3 mt-3 aspect-[4/3] shrink-0 overflow-hidden rounded-[8px] bg-[#0e0c0a]"
    onclick={() => store.openLoupe(image.id)}
    title="Open (Enter)"
  >
    <Thumb id={image.id} fit="contain" />
    <span class="absolute inset-0 grid place-items-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
      <span class="flex items-center gap-1.5 rounded-full bg-black/60 px-3 py-1 text-xs text-ink"><Icon name="maximize" size={13} />Open</span>
    </span>
  </button>

  <div class="flex items-center gap-1 px-3 pt-2">
    <Badge status={image.status} label={STATUS_LABEL[image.status] ?? image.status} />
    <div class="ml-auto flex">
      <button class="icon-btn" title="Rotate left ([)" aria-label="Rotate left" onclick={() => store.rotateTargets(false)}>
        <Icon name="rotateCcw" size={15} />
      </button>
      <button class="icon-btn" title="Rotate right (])" aria-label="Rotate right" onclick={() => store.rotateTargets(true)}>
        <Icon name="rotateCw" size={15} />
      </button>
      <button class="icon-btn hover:!text-danger" title="Delete (Del)" aria-label="Delete" onclick={() => store.deleteTargets()}>
        <Icon name="trash" size={15} />
      </button>
    </div>
  </div>

  <section class="flex flex-col gap-2.5 border-b border-line px-3 pt-2 pb-4">
    <label class="flex flex-col gap-1">
      <span class="eyebrow">Title</span>
      <input class="input" bind:value={form.title} placeholder={image.filename} onkeydown={onKey} />
    </label>
    <div class="grid grid-cols-[88px_1fr] gap-2">
      <label class="flex flex-col gap-1">
        <span class="eyebrow">Year</span>
        <input
          class="input tabular-nums"
          class:!border-danger={!yearValid}
          bind:value={form.year}
          placeholder="YYYY"
          inputmode="numeric"
          maxlength="4"
          onkeydown={onKey}
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="eyebrow">Month</span>
        <select class="input" bind:value={form.month}>
          <option value="">Unknown</option>
          {#each MONTHS.slice(1) as name, i (i)}
            <option value={String(i + 1)}>{name}</option>
          {/each}
        </select>
      </label>
    </div>
    <label class="flex flex-col gap-1">
      <span class="eyebrow">Folder</span>
      <input class="input" bind:value={form.folder} placeholder="None — filed by date" list="folder-suggestions" onkeydown={onKey} />
      <datalist id="folder-suggestions">
        {#each store.counts.folders as fl (fl.value)}<option value={fl.value}></option>{/each}
      </datalist>
      <span class="text-[11px] text-ink-faint">Takes effect the next time this photo is organized.</span>
    </label>
    {#if dirty}
      <div class="pop flex gap-2">
        <button class="btn flex-1" disabled={!yearValid || saving} onclick={save}>
          <Icon name="check" size={14} />Save
        </button>
        <button class="btn-sm h-8" onclick={() => (form = { ...saved })}>Revert</button>
      </div>
    {/if}
  </section>

  <section class="border-b border-line px-3 py-3">
    <h3 class="eyebrow mb-2">Details</h3>
    <dl class="grid grid-cols-[84px_1fr] gap-x-3 gap-y-1.5 text-xs">
      <dt class="text-ink-faint">File</dt>
      <dd class="truncate text-ink" title={image.filename}>{image.filename}</dd>
      <dt class="text-ink-faint">Size</dt>
      <dd class="text-ink tabular-nums">{image.width ?? '?'} × {image.height ?? '?'} · {bytes(image.file_size)}</dd>
      {#if image.scan_id}
        <dt class="text-ink-faint">Scan</dt>
        <dd class="text-ink tabular-nums">{image.scan_id}</dd>
      {/if}
      <dt class="text-ink-faint">Source</dt>
      <dd class="truncate font-mono text-[11px] text-ink-dim" title={image.source_path}>{image.source_path}</dd>
      {#if image.organized_path}
        <dt class="text-ink-faint">Filed as</dt>
        <dd class="truncate font-mono text-[11px] text-ink-dim" title={image.organized_path}>{image.organized_path}</dd>
      {/if}
      {#if image.enhanced_path}
        <dt class="text-ink-faint">Restored</dt>
        <dd class="truncate font-mono text-[11px] text-ink-dim" title={image.enhanced_path}>{image.enhanced_path}</dd>
      {/if}
    </dl>
  </section>

  <section class="px-3 py-3">
    <h3 class="eyebrow mb-2">History</h3>
    {#if history.length === 0}
      <p class="text-xs text-ink-faint">Not processed yet.</p>
    {:else}
      <ol class="relative flex flex-col gap-2 border-l border-line pl-3.5">
        {#each history as h (h.id)}
          <li class="relative text-xs">
            <span class="absolute top-1.5 -left-[18px] h-[7px] w-[7px] rounded-full border border-surface bg-ink-faint"></span>
            <span class="text-ink">{stepLabel(h.step)}</span>
            <span class="ml-1 text-ink-faint tabular-nums">{when(h.created_at)}</span>
          </li>
        {/each}
      </ol>
    {/if}
  </section>
</div>

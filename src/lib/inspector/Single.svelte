<script lang="ts">
  import { untrack } from 'svelte'
  import { imageTimeline } from '../api'
  import { store, type Meta, type MetaField } from '../store.svelte'
  import { MONTHS, STATUS, dateLabel, stepLabel, type Image, type ImageTimeline } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Badge from '../ui/Badge.svelte'
  import Thumb from '../ui/Thumb.svelte'

  let { image }: { image: Image } = $props()

  const fromImage = () => ({
    title: image.title ?? '',
    year: image.year?.toString() ?? '',
    month: image.month?.toString() ?? '',
    folder: image.folder ?? '',
  })

  let form = $state(fromImage())
  // What the form was last filled from, to tell typed edits from stale values.
  let base = fromImage()
  let saving = false
  let timeline = $state<ImageTimeline>({ pos: 0, steps: [] })
  const inputs: Partial<Record<MetaField, HTMLInputElement>> = $state({})

  const parsed = (): Meta => ({
    title: form.title.trim() || null,
    year: form.year.trim() ? Number(form.year) : null,
    month: form.month ? Number(form.month) : null,
    folder: form.folder.trim() || null,
  })
  const dirty = $derived.by(() => {
    const m = parsed()
    return m.title !== image.title || m.year !== image.year || m.month !== image.month || m.folder !== image.folder
  })
  const yearValid = $derived(form.year.trim() === '' || /^(1[89]|20)\d{2}$/.test(form.year.trim()))
  const st = $derived(STATUS[image.status])
  const last = $derived(store.lastMeta)
  const lastLabel = $derived(
    last ? [last.year != null ? dateLabel(last) : null, last.folder].filter(Boolean).join(' · ') : '',
  )
  const lastDiffers = $derived(
    !!last && ((last.year != null && (last.year !== image.year || last.month !== image.month)) || (!!last.folder && last.folder !== image.folder)),
  )

  // Pick up changes made elsewhere (batch edits, "same as last", a restore).
  $effect(() => {
    void image.updated_at
    const next = fromImage()
    untrack(() => {
      if (JSON.stringify(form) === JSON.stringify(base)) form = next
      base = next
    })
  })

  $effect(() => {
    void store.thumbVersion
    void image.updated_at
    imageTimeline(image.id).then((t) => (timeline = t)).catch(() => (timeline = { pos: 0, steps: [] }))
  })

  // Land in the field the previous photo was being typed in.
  $effect(() => {
    const field = store.refocus
    if (!field) return
    untrack(() => {
      store.refocus = null
      inputs[field]?.focus()
      inputs[field]?.select()
    })
  })

  async function commit(advance: MetaField | null = null) {
    if (saving || !yearValid) return
    if (!dirty) {
      if (advance) {
        store.refocus = advance
        store.step(1)
      }
      return
    }
    saving = true
    try {
      await store.saveMeta(image, parsed(), advance)
    } finally {
      saving = false
    }
  }

  function onKey(e: KeyboardEvent, field: MetaField) {
    if (e.key === 'Enter') {
      e.preventDefault()
      commit(field)
    } else if (e.key === 'Escape') {
      e.preventDefault()
      form = fromImage()
      ;(e.currentTarget as HTMLElement).blur()
    } else if (e.key === '.' && field === 'year' && last) {
      e.preventDefault()
      form = fromImage()
      store.refocus = 'year'
      store.applyLast(true)
    }
  }

  function bytes(n: number | null) {
    if (n == null) return '—'
    if (n < 1024 * 1024) return `${Math.round(n / 1024)} KB`
    return `${(n / 1024 / 1024).toFixed(1)} MB`
  }
</script>

<div class="flex h-full flex-col overflow-y-auto">
  {#if store.view !== 'loupe'}
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
  {/if}

  <div class="flex items-center gap-1 px-3 pt-3">
    <span title={st?.hint}><Badge status={image.status} label={st?.label ?? image.status} /></span>
    <div class="ml-auto flex">
      <button class="icon-btn" title="Rotate left ([)" aria-label="Rotate left" onclick={() => store.rotateTargets(false)}>
        <Icon name="rotateCcw" size={15} />
      </button>
      <button class="icon-btn" title="Rotate right (])" aria-label="Rotate right" onclick={() => store.rotateTargets(true)}>
        <Icon name="rotateCw" size={15} />
      </button>
      <button class="icon-btn" title="Show in folder" aria-label="Show in folder" onclick={() => store.reveal(image.id)}>
        <Icon name="external" size={15} />
      </button>
      <button class="icon-btn hover:!text-danger" title="Remove from library (Del)" aria-label="Remove" onclick={() => store.deleteTargets()}>
        <Icon name="trash" size={15} />
      </button>
    </div>
  </div>

  {#if image.status === 'failed'}
    <div class="mx-3 mt-2 rounded-[8px] border border-danger/30 bg-danger-soft p-3">
      <p class="text-xs font-medium text-ink">The last restore failed</p>
      {#if image.error}<p class="mt-1 text-[11px] leading-snug break-words text-ink-dim select-text">{image.error}</p>{/if}
      <button class="btn-sm mt-2.5" onclick={() => store.openInspector('restore')}><Icon name="refresh" size={13} />Try again…</button>
    </div>
  {/if}

  <section class="flex flex-col gap-2.5 border-b border-line px-3 pt-3 pb-4">
    <div class="grid grid-cols-[84px_1fr] gap-2">
      <label class="flex flex-col gap-1">
        <span class="eyebrow">Year</span>
        <input
          bind:this={inputs.year}
          class="input tabular-nums"
          class:!border-danger={!yearValid}
          bind:value={form.year}
          placeholder="YYYY"
          inputmode="numeric"
          maxlength="4"
          onchange={() => commit()}
          onkeydown={(e) => onKey(e, 'year')}
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="eyebrow">Month</span>
        <select class="input" bind:value={form.month} onchange={() => commit()}>
          <option value="">Unknown</option>
          {#each MONTHS.slice(1) as name, i (i)}
            <option value={String(i + 1)}>{name}</option>
          {/each}
        </select>
      </label>
    </div>
    {#if !yearValid}
      <p class="-mt-1 text-[11px] text-danger">A year looks like 1978.</p>
    {/if}
    <label class="flex flex-col gap-1">
      <span class="eyebrow">Title</span>
      <input
        bind:this={inputs.title}
        class="input"
        bind:value={form.title}
        placeholder="Who or what is this?"
        onchange={() => commit()}
        onkeydown={(e) => onKey(e, 'title')}
      />
    </label>
    <label class="flex flex-col gap-1">
      <span class="eyebrow">Folder</span>
      <input
        bind:this={inputs.folder}
        class="input"
        bind:value={form.folder}
        placeholder="None — filed by year"
        list="folder-suggestions"
        onchange={() => commit()}
        onkeydown={(e) => onKey(e, 'folder')}
      />
      <datalist id="folder-suggestions">
        {#each store.counts.folders as fl (fl.value)}<option value={fl.value}></option>{/each}
      </datalist>
    </label>
    {#if lastDiffers}
      <button class="btn-sm justify-start" title="Give this photo the date of the last one you edited (.)" onclick={() => store.applyLast()}>
        <Icon name="repeat" size={13} />
        <span class="truncate">Same as last: {lastLabel}</span>
        <span class="kbd ml-auto">.</span>
      </button>
    {/if}
    <p class="text-[11px] leading-snug text-ink-faint">
      <span class="kbd">Enter</span> saves and moves to the next photo.
    </p>
  </section>

  <section class="border-b border-line px-3 py-3">
    <div class="mb-2 flex items-center">
      <h3 class="eyebrow">Restoration</h3>
      {#if timeline.steps.length}
        <div class="ml-auto flex">
          <button class="icon-btn h-6 w-6" title="Undo last step (Ctrl+Z)" aria-label="Undo" disabled={timeline.pos <= 0} onclick={() => store.undoRedo(image.id, false)}>
            <Icon name="undo" size={14} />
          </button>
          <button class="icon-btn h-6 w-6" title="Redo (Ctrl+Shift+Z)" aria-label="Redo" disabled={timeline.pos >= timeline.steps.length} onclick={() => store.undoRedo(image.id, true)}>
            <Icon name="redo" size={14} />
          </button>
        </div>
      {/if}
    </div>
    {#if !image.organized_path}
      <p class="text-xs text-ink-faint">Still an untouched scan.</p>
      <button class="btn-sm mt-2" onclick={() => store.openInspector('restore')}><Icon name="wand" size={13} />Restore…</button>
    {:else}
      <ol class="flex flex-wrap gap-1">
        <li class="rounded-full bg-info-soft px-2 py-0.5 text-[11px] text-info">Filed</li>
        {#each timeline.steps as s (s.seq)}
          {@const undone = s.seq > timeline.pos}
          <li
            class="rounded-full px-2 py-0.5 text-[11px] {undone ? 'bg-surface-2 text-ink-faint line-through' : 'bg-success-soft text-success'}"
            title={undone ? 'Undone — redo brings it back' : s.created_at.replace('T', ' ').slice(0, 16)}
          >
            {stepLabel(s.label)}
          </li>
        {/each}
        {#if image.enhanced_path}
          <li class="rounded-full bg-success-soft px-2 py-0.5 text-[11px] text-success">Upscaled 4×</li>
        {/if}
      </ol>
      <div class="mt-2.5 flex gap-1.5">
        <button class="btn-sm" onclick={() => store.openInspector('restore')}><Icon name="wand" size={13} />More steps…</button>
        <button class="btn-sm" title="Discard the archive copy and every step" onclick={() => store.resetTargets()}>Start over</button>
      </div>
    {/if}
  </section>

  <section class="px-3 py-3">
    <h3 class="eyebrow mb-2">File</h3>
    <dl class="grid grid-cols-[64px_1fr] gap-x-3 gap-y-1.5 text-xs">
      <dt class="text-ink-faint">Scan</dt>
      <dd class="truncate text-ink select-text" title={image.source_path}>{image.source_path}</dd>
      <dt class="text-ink-faint">Size</dt>
      <dd class="text-ink tabular-nums">{image.width ?? '?'} × {image.height ?? '?'} · {bytes(image.file_size)}</dd>
      {#if image.organized_path}
        <dt class="text-ink-faint">Archive</dt>
        <dd class="truncate text-ink select-text" title={image.organized_path}>{image.organized_path.replace(/^organized\//, '')}</dd>
      {/if}
    </dl>
  </section>
</div>

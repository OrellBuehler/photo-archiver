<script lang="ts">
  import { listImageIds, listModels } from '../api'
  import { store } from '../store.svelte'
  import { PIPELINE_STEPS, PRESETS, plural, stepModelKeys } from '../types'
  import type { ModelDownload, ModelStatus } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Spinner from '../ui/Spinner.svelte'
  import Progress from '../ui/Progress.svelte'

  const KNOWN = new Set(PIPELINE_STEPS.map((s) => s.key))

  function loadSteps(): Set<string> {
    try {
      const raw = localStorage.getItem('photo-archiver:steps')
      if (raw) return new Set((JSON.parse(raw) as string[]).filter((k) => KNOWN.has(k)))
    } catch {}
    return new Set(PRESETS[2].steps)
  }

  let steps = $state<Set<string>>(loadSteps())
  let scope = $state<'selection' | 'view'>('selection')
  let force = $state(false)

  $effect(() => {
    try {
      localStorage.setItem('photo-archiver:steps', JSON.stringify([...steps]))
    } catch {}
  })

  const targetCount = $derived(store.targetIds.length)
  const useView = $derived(scope === 'view' || targetCount === 0)
  const count = $derived(useView ? store.total : targetCount)
  const place = $derived(store.placeLabel + (store.filters.search ? ` matching “${store.filters.search}”` : ''))

  function toggle(key: string) {
    const next = new Set(steps)
    if (next.has(key)) next.delete(key)
    else next.add(key)
    steps = next
  }

  const activePreset = $derived(
    PRESETS.find((p) => p.steps.length === steps.size && p.steps.every((s) => steps.has(s)))?.key ?? null,
  )

  const t = $derived(store.activeTask)
  const done = $derived(t ? t.completed + t.failed : 0)
  const ordered = $derived(PIPELINE_STEPS.filter((s) => steps.has(s.key)).map((s) => s.key))
  const verb = $derived(steps.size ? 'Restore' : 'File')

  function stepState(key: string): 'idle' | 'running' | 'done' {
    if (!t?.currentStep) return 'idle'
    const ci = ordered.indexOf(t.currentStep)
    const ki = ordered.indexOf(key)
    if (ci < 0 || ki < 0) return 'idle'
    return ki < ci ? 'done' : ki === ci ? 'running' : 'idle'
  }

  // Runs that need a not-yet-downloaded model ask first.
  let pending = $state<number[] | null>(null)
  let missing = $state<ModelStatus[]>([])
  const requiredModels = $derived(PIPELINE_STEPS.filter((s) => steps.has(s.key)).flatMap(stepModelKeys))

  async function run() {
    let ids = store.targetIds
    try {
      if (useView) {
        ids = await listImageIds(store.filters)
        if (ids.length > 1) {
          const ok = await store.confirm({
            title: `${verb} all ${ids.length} photos in ${place}?`,
            message: force
              ? 'Every photo starts over from its scan — earlier steps and rotations are discarded.'
              : 'Steps a photo already has are skipped, so this only does what is missing. The scans themselves are never modified.',
            confirmLabel: 'Start',
          })
          if (!ok) return
        }
      }
      if (requiredModels.length) {
        const statuses = await listModels()
        const m = statuses.filter((s) => requiredModels.includes(s.key) && !s.downloaded)
        if (m.length) {
          missing = m
          pending = ids
          return
        }
      }
    } catch (e) {
      return store.fail(e)
    }
    await start(ids)
  }

  async function start(ids: number[]) {
    pending = null
    const f = force
    force = false
    await store.restore(ids, ordered, f)
  }

  async function downloadAndRun() {
    if (!pending) return
    if (await store.downloadModelFiles(missing.map((m) => m.key))) await start(pending)
  }

  function pctOf(p: ModelDownload) {
    return p.total ? Math.min(100, Math.round((p.downloaded / p.total) * 100)) : null
  }
</script>

<div class="flex h-full flex-col">
  <div class="min-h-0 flex-1 overflow-y-auto">
    {#if t}
      <section class="border-b border-line bg-brand-soft/30 px-3 py-3">
        <div class="flex items-center gap-2">
          <Spinner size={13} />
          <span class="text-[13px] font-medium text-ink">Working…</span>
          <span class="ml-auto text-xs text-ink-dim tabular-nums">{done} / {t.total}</span>
        </div>
        <div class="mt-2.5"><Progress value={t.total ? (done / t.total) * 100 : null} height={6} /></div>
        {#if t.failed}<p class="mt-1.5 text-[11px] text-danger">{t.failed} failed so far</p>{/if}
        <button class="btn-danger mt-2.5 w-full" onclick={() => store.cancelActive()}>
          <Icon name="stop" size={11} />Stop after this photo
        </button>
      </section>
    {/if}

    <section class="px-3 pt-4">
      <h3 class="eyebrow mb-2">Which photos</h3>
      <div class="seg w-full">
        <button
          class="min-w-0 flex-1 justify-center"
          aria-pressed={!useView}
          disabled={targetCount === 0}
          class:opacity-40={targetCount === 0}
          onclick={() => (scope = 'selection')}
        >
          {targetCount > 1 ? `${targetCount} selected` : targetCount === 1 ? 'This photo' : 'Selection'}
        </button>
        <button class="min-w-0 flex-1 justify-center" aria-pressed={useView} onclick={() => (scope = 'view')} title="Everything in {place}">
          <span class="truncate">All in {store.placeLabel}</span>
          <span class="tabular-nums opacity-70">{store.total}</span>
        </button>
      </div>
    </section>

    <section class="px-3 pt-5">
      <h3 class="eyebrow mb-2">Recipe</h3>
      <div class="grid grid-cols-2 gap-1.5">
        {#each PRESETS as p (p.key)}
          {@const on = activePreset === p.key}
          <button
            type="button"
            onclick={() => (steps = new Set(p.steps))}
            disabled={!!t}
            class="rounded-[6px] border px-2.5 py-2 text-left transition-colors {on
              ? 'border-brand/50 bg-brand-soft'
              : 'border-line bg-base hover:border-[var(--border-strong)]'}"
          >
            <span class="block text-xs font-medium {on ? 'text-brand' : 'text-ink'}">{p.label}</span>
            <span class="mt-0.5 block text-[11px] leading-tight text-ink-faint">{p.desc}</span>
          </button>
        {/each}
      </div>
    </section>

    <section class="px-3 pt-5 pb-4">
      <div class="mb-2 flex items-baseline justify-between">
        <h3 class="eyebrow">Steps</h3>
        <span class="text-[11px] text-ink-faint">{steps.size} of {PIPELINE_STEPS.length}</span>
      </div>
      <div class="flex flex-col overflow-hidden rounded-[8px] border border-line bg-base">
        <div class="flex items-center gap-2.5 border-b border-line/70 px-2.5 py-1.5">
          <span class="grid h-4 w-4 shrink-0 place-items-center rounded-[4px] bg-surface-2 text-ink-faint">
            <Icon name="check" size={10} stroke={3.2} />
          </span>
          <Icon name="archive" size={15} class="text-ink-faint" />
          <span class="min-w-0 flex-1">
            <span class="block text-xs text-ink-dim">Copy into the archive</span>
            <span class="block truncate text-[11px] text-ink-faint">Always — by folder, year and month</span>
          </span>
        </div>
        {#each PIPELINE_STEPS as s (s.key)}
          {@const on = steps.has(s.key)}
          {@const st = stepState(s.key)}
          <button
            type="button"
            role="checkbox"
            aria-checked={on}
            onclick={() => toggle(s.key)}
            disabled={!!t}
            class="flex items-center gap-2.5 border-b border-line/70 px-2.5 py-1.5 text-left transition-colors last:border-b-0 hover:bg-surface-2"
          >
            <span
              class="grid h-4 w-4 shrink-0 place-items-center rounded-[4px] border-[1.5px] transition-colors {on
                ? 'border-brand bg-brand text-on-brand'
                : 'border-[var(--border-strong)]'}"
            >
              {#if on}<Icon name="check" size={10} stroke={3.2} />{/if}
            </span>
            <Icon name={s.icon} size={15} class={on ? 'text-brand' : 'text-ink-faint'} />
            <span class="min-w-0 flex-1">
              <span class="flex items-center gap-1.5">
                <span class="text-xs {on ? 'text-ink' : 'text-ink-dim'}">{s.label}</span>
                {#if s.model}
                  <span class="rounded-[3px] bg-info-soft px-1 text-[9px] font-semibold tracking-wide text-info">AI</span>
                {/if}
              </span>
              <span class="block truncate text-[11px] text-ink-faint">{s.hint}</span>
            </span>
            {#if st === 'running'}
              <Spinner size={12} color="var(--color-warning)" />
            {:else if st === 'done'}
              <Icon name="check" size={14} class="text-success" />
            {/if}
          </button>
        {/each}
      </div>

      <label class="mt-3 flex cursor-pointer items-start gap-2.5 px-0.5">
        <input type="checkbox" bind:checked={force} disabled={!!t} class="mt-0.5 accent-[var(--color-brand)]" />
        <span>
          <span class="block text-xs text-ink-dim">Start over from the scan</span>
          <span class="block text-[11px] leading-snug text-ink-faint">Otherwise steps a photo already has are skipped.</span>
        </span>
      </label>
    </section>
  </div>

  {#if pending}
    <section class="pop border-t border-brand/40 bg-brand-soft/40 px-3 py-3">
      <div class="flex items-start gap-2">
        <Icon name="download" size={15} class="mt-0.5 text-brand" />
        <div>
          <p class="text-xs font-medium text-ink">{plural(missing.length, 'AI model')} to download first</p>
          <p class="mt-0.5 text-[11px] leading-snug text-ink-dim">One-time download; they run on this computer afterwards.</p>
        </div>
      </div>
      <div class="mt-2 flex flex-col gap-1.5">
        {#each missing as m (m.key)}
          {@const p = store.modelProgress[m.key]}
          <div class="rounded-[5px] border border-line bg-surface px-2.5 py-1.5">
            <div class="flex items-center justify-between gap-2 text-xs">
              <span class="truncate text-ink">{m.label}</span>
              {#if p?.error}
                <span class="text-danger" title={p.error}>Failed</span>
              {:else if p?.done}
                <Icon name="check" size={13} class="text-success" />
              {:else if p}
                <span class="text-ink-dim tabular-nums">{pctOf(p) ?? Math.round(p.downloaded / 1e6)}{pctOf(p) != null ? '%' : ' MB'}</span>
              {:else}
                <span class="text-ink-faint">~{m.approx_mb} MB</span>
              {/if}
            </div>
            {#if p && !p.done && !p.error}
              <div class="mt-1.5"><Progress value={pctOf(p)} height={3} /></div>
            {/if}
          </div>
        {/each}
      </div>
      <div class="mt-2.5 flex gap-1.5">
        <button class="btn flex-1" disabled={store.downloadingModels} onclick={downloadAndRun}>
          {#if store.downloadingModels}<Spinner size={12} color="var(--color-on-brand)" />Downloading…{:else}Download and start{/if}
        </button>
        <button class="btn-sm h-8" disabled={store.downloadingModels} onclick={() => (pending = null)}>Cancel</button>
      </div>
    </section>
  {:else}
    <div class="border-t border-line p-3">
      <button class="btn w-full" disabled={store.busy || count === 0 || store.downloadingModels} onclick={run}>
        <Icon name={steps.size ? 'sparkles' : 'archive'} size={15} />
        {#if store.busy}Working…{:else}{verb} {plural(count, 'photo')}{/if}
      </button>
      <p class="mt-1.5 text-center text-[11px] text-ink-faint">Your scans are never modified.</p>
    </div>
  {/if}
</div>

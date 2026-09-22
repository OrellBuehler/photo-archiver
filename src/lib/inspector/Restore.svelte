<script lang="ts">
  import { listModels } from '../api'
  import { store } from '../store.svelte'
  import { PIPELINE_STEPS, PRESETS, stepModelKeys } from '../types'
  import type { ModelDownload, ModelStatus } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Spinner from '../ui/Spinner.svelte'
  import Progress from '../ui/Progress.svelte'

  function loadSteps(): Set<string> {
    try {
      const raw = localStorage.getItem('photo-archiver:steps')
      if (raw) return new Set(JSON.parse(raw))
    } catch {}
    return new Set(['organize', 'crop', 'deskew', 'restore_color', 'remove_dust'])
  }

  let steps = $state<Set<string>>(loadSteps())
  let scope = $state<'selection' | 'all'>('selection')

  $effect(() => {
    try {
      localStorage.setItem('photo-archiver:steps', JSON.stringify([...steps]))
    } catch {}
  })

  const targetCount = $derived(store.targetIds.length)
  const useAll = $derived(scope === 'all' || targetCount === 0)
  const count = $derived(useAll ? store.libraryTotal : targetCount)

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

  function stepState(key: string): 'idle' | 'running' | 'done' {
    if (!t?.currentStep) return 'idle'
    const ci = ordered.indexOf(t.currentStep)
    const ki = ordered.indexOf(key)
    if (ci < 0 || ki < 0) return 'idle'
    return ki < ci ? 'done' : ki === ci ? 'running' : 'idle'
  }

  // Runs that need a not-yet-downloaded model ask first.
  let pending = $state<{ all: boolean } | null>(null)
  let missing = $state<ModelStatus[]>([])
  const requiredModels = $derived(PIPELINE_STEPS.filter((s) => steps.has(s.key)).flatMap(stepModelKeys))

  async function run() {
    const all = useAll
    if (all) {
      const ok = await store.confirm({
        title: `Restore all ${store.libraryTotal} photos?`,
        message: `${ordered.length} step${ordered.length === 1 ? '' : 's'} will run on every photo in the library. Originals are never modified.`,
        confirmLabel: 'Start',
      })
      if (!ok) return
    }
    if (requiredModels.length) {
      try {
        const statuses = await listModels()
        const m = statuses.filter((s) => requiredModels.includes(s.key) && !s.downloaded)
        if (m.length) {
          missing = m
          pending = { all }
          return
        }
      } catch (e) {
        store.fail(e)
      }
    }
    await store.startBatch([...steps], all)
  }

  async function downloadAndRun() {
    if (!pending) return
    const ok = await store.downloadModelFiles(missing.map((m) => m.key))
    if (!ok) return
    const { all } = pending
    pending = null
    await store.startBatch([...steps], all)
  }

  async function runAnyway() {
    if (!pending) return
    const { all } = pending
    pending = null
    await store.startBatch([...steps], all)
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
          <span class="text-[13px] font-medium text-ink">Restoring…</span>
          <span class="ml-auto text-xs text-ink-dim tabular-nums">{done} / {t.total}</span>
        </div>
        <div class="mt-2.5"><Progress value={t.total ? (done / t.total) * 100 : null} height={6} /></div>
        {#if t.failed}<p class="mt-1.5 text-[11px] text-danger">{t.failed} failed so far</p>{/if}
        <button class="btn-danger mt-2.5 w-full" onclick={() => store.cancelActive()}>
          <Icon name="stop" size={11} />Cancel
        </button>
      </section>
    {/if}

    <section class="px-3 pt-4">
      <h3 class="eyebrow mb-2">Apply to</h3>
      <div class="seg w-full">
        <button
          class="flex-1 justify-center"
          aria-pressed={!useAll}
          disabled={targetCount === 0}
          class:opacity-40={targetCount === 0}
          onclick={() => (scope = 'selection')}
        >
          {targetCount > 0 ? `${targetCount} selected` : 'Selection'}
        </button>
        <button class="flex-1 justify-center" aria-pressed={useAll} onclick={() => (scope = 'all')}>
          Whole library
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
        <span class="text-[11px] text-ink-faint">{steps.size} on</span>
      </div>
      <div class="flex flex-col overflow-hidden rounded-[8px] border border-line bg-base">
        {#each PIPELINE_STEPS as s (s.key)}
          {@const on = steps.has(s.key)}
          {@const st = stepState(s.key)}
          <button
            type="button"
            onclick={() => toggle(s.key)}
            disabled={!!t}
            class="flex items-center gap-2.5 border-b border-line/70 px-2.5 py-2 text-left transition-colors last:border-b-0 hover:bg-surface-2"
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
    </section>
  </div>

  {#if pending}
    <section class="pop border-t border-brand/40 bg-brand-soft/40 px-3 py-3">
      <div class="flex items-start gap-2">
        <Icon name="download" size={15} class="mt-0.5 text-brand" />
        <div>
          <p class="text-xs font-medium text-ink">
            {missing.length} AI model{missing.length > 1 ? 's' : ''} to download
          </p>
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
          {#if store.downloadingModels}<Spinner size={12} color="var(--color-on-brand)" />Downloading…{:else}Download & run{/if}
        </button>
        <button class="btn-sm h-8" disabled={store.downloadingModels} onclick={runAnyway}>Skip</button>
        <button class="icon-btn h-8 w-8" aria-label="Cancel" disabled={store.downloadingModels} onclick={() => (pending = null)}>
          <Icon name="x" size={14} />
        </button>
      </div>
    </section>
  {:else}
    <div class="border-t border-line p-3">
      <button class="btn w-full" disabled={store.busy || steps.size === 0 || count === 0 || store.downloadingModels} onclick={run}>
        <Icon name="sparkles" size={15} />
        {#if store.busy}Running…{:else}Restore {count} photo{count === 1 ? '' : 's'}{/if}
      </button>
      <p class="mt-1.5 text-center text-[11px] text-ink-faint">Originals are never modified.</p>
    </div>
  {/if}
</div>

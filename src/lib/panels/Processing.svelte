<script lang="ts">
  import { listModels } from '../api'
  import { store } from '../store.svelte'
  import { PIPELINE_STEPS, PRESETS } from '../types'
  import type { ModelDownload, ModelStatus } from '../types'
  import Icon from '../ui/Icon.svelte'
  import StepCard from '../ui/StepCard.svelte'

  let selectedSteps = $state<Set<string>>(new Set(['organize', 'crop', 'restore_color']))

  function toggle(key: string) {
    const next = new Set(selectedSteps)
    if (next.has(key)) next.delete(key)
    else next.add(key)
    selectedSteps = next
  }

  function applyPreset(steps: string[]) {
    selectedSteps = new Set(steps)
  }

  const activePreset = $derived.by(() => {
    for (const p of PRESETS) {
      if (p.steps.length === selectedSteps.size && p.steps.every((s) => selectedSteps.has(s)))
        return p.key
    }
    return null
  })

  const active = $derived(store.activeTask)
  const pct = $derived(
    active && active.total > 0
      ? Math.round(((active.completed + active.failed) / active.total) * 100)
      : 0,
  )

  // Pipeline-ordered view of the selected steps, for live "done / running" state.
  const orderedSelected = $derived(
    PIPELINE_STEPS.filter((s) => selectedSteps.has(s.key)).map((s) => s.key),
  )

  function stepState(key: string): 'idle' | 'running' | 'done' {
    if (!active || active.status !== 'running' || !active.currentStep) return 'idle'
    const ci = orderedSelected.indexOf(active.currentStep)
    const ki = orderedSelected.indexOf(key)
    if (ci < 0 || ki < 0) return 'idle'
    if (ki < ci) return 'done'
    if (ki === ci) return 'running'
    return 'idle'
  }

  const curStep = $derived(
    active?.currentStep ? PIPELINE_STEPS.find((s) => s.key === active.currentStep) : null,
  )

  // ETA — derived from observed throughput since the task started.
  let now = $state(Date.now())
  let startedAt = $state<number | null>(null)
  let trackedTaskId: number | null = null

  $effect(() => {
    if (active && active.status === 'running') {
      if (trackedTaskId !== active.id) {
        trackedTaskId = active.id
        startedAt = Date.now()
      }
      const t = setInterval(() => (now = Date.now()), 1000)
      return () => clearInterval(t)
    }
    startedAt = null
    trackedTaskId = null
  })

  const eta = $derived.by(() => {
    if (!active || active.status !== 'running' || !startedAt) return null
    const done = active.completed + active.failed
    if (done <= 0) return 'estimating…'
    const per = (now - startedAt) / 1000 / done
    const secs = Math.round(per * Math.max(0, active.total - done))
    if (secs <= 0) return 'almost done'
    if (secs < 60) return `~${secs}s left`
    const m = Math.floor(secs / 60)
    const s = secs % 60
    return `~${m}m${s ? ' ' + s + 's' : ''} left`
  })

  // A run is gated if any selected step needs a model that isn't downloaded.
  let pendingRun = $state<{ all: boolean } | null>(null)
  let missingModels = $state<ModelStatus[]>([])

  const requiredModelKeys = $derived(
    PIPELINE_STEPS.filter((s) => s.modelKey && selectedSteps.has(s.key)).map((s) => s.modelKey as string),
  )

  async function run(all: boolean) {
    if (requiredModelKeys.length > 0) {
      const statuses = await listModels()
      const missing = statuses.filter((m) => requiredModelKeys.includes(m.key) && !m.downloaded)
      if (missing.length > 0) {
        missingModels = missing
        pendingRun = { all }
        return
      }
    }
    await store.startBatch([...selectedSteps], all)
  }

  async function downloadAndRun() {
    if (!pendingRun) return
    const ok = await store.downloadModelFiles(missingModels.map((m) => m.key))
    if (!ok) return // leave the prompt up so the failure stays visible
    const { all } = pendingRun
    pendingRun = null
    await store.startBatch([...selectedSteps], all)
  }

  async function runAnyway() {
    if (!pendingRun) return
    const { all } = pendingRun
    pendingRun = null
    await store.startBatch([...selectedSteps], all)
  }

  function pctOf(p: ModelDownload): number {
    if (!p.total) return 0
    return Math.min(100, Math.round((p.downloaded / p.total) * 100))
  }
</script>

<div class="flex h-full flex-col gap-5 overflow-auto bg-base p-3 text-sm">
  <section>
    <h2 class="eyebrow mb-2">Presets</h2>
    <div class="grid grid-cols-2 gap-1.5">
      {#each PRESETS as p (p.key)}
        <button
          type="button"
          onclick={() => applyPreset(p.steps)}
          class={`rounded-[5px] border px-2.5 py-2 text-left transition-colors duration-[110ms] ${
            activePreset === p.key
              ? 'border-brand/50 bg-brand-soft'
              : 'border-line bg-surface hover:border-[var(--border-strong)] hover:bg-surface-2'
          }`}
        >
          <span
            class="block text-xs font-medium"
            style={activePreset === p.key ? 'color: var(--color-brand);' : 'color: var(--color-ink);'}
          >{p.label}</span>
          <span class="mt-0.5 block text-[11px] leading-tight text-ink-dim">{p.desc}</span>
        </button>
      {/each}
    </div>
  </section>

  <section>
    <div class="mb-2 flex items-center justify-between">
      <h2 class="eyebrow">Steps</h2>
      <span class="text-[11px] text-ink-faint">{selectedSteps.size} selected</span>
    </div>
    <div class="flex flex-col gap-1.5">
      {#each PIPELINE_STEPS as step (step.key)}
        <StepCard
          icon={step.icon}
          label={step.label}
          hint={step.hint}
          model={step.model}
          selected={selectedSteps.has(step.key)}
          state={stepState(step.key)}
          onToggle={() => toggle(step.key)}
        />
      {/each}
    </div>
  </section>

  <section class="flex flex-col gap-2">
    <button
      class="btn"
      disabled={store.busy || store.downloadingModels || selectedSteps.size === 0 || store.selected.size === 0}
      onclick={() => run(false)}
    >
      <Icon name="sparkles" size={15} />
      Process selected ({store.selected.size})
    </button>
    <button
      class="btn-sm"
      disabled={store.busy || store.downloadingModels || selectedSteps.size === 0}
      onclick={() => run(true)}
    >
      Process all ({store.total})
    </button>
  </section>

  {#if pendingRun}
    <section class="flex flex-col gap-2.5 rounded-[8px] border border-brand/40 bg-brand-soft/40 p-3">
      <div class="flex items-start gap-2">
        <Icon name="download" size={15} class="mt-0.5 shrink-0 text-brand" />
        <div class="min-w-0">
          <p class="text-xs font-medium text-ink">
            {missingModels.length} model{missingModels.length > 1 ? 's' : ''} needed for this run
          </p>
          <p class="mt-0.5 text-[11px] leading-tight text-ink-dim">
            Some selected steps use ML models that aren't downloaded yet. Fetch them now, or run
            anyway and they'll download on first use mid-run.
          </p>
        </div>
      </div>

      <div class="flex flex-col gap-1.5">
        {#each missingModels as m (m.key)}
          {@const p = store.modelProgress[m.key]}
          {@const active = store.downloadingModels && p && !p.done && !p.error}
          <div class="flex flex-col gap-1 rounded-[5px] border border-line bg-surface px-2.5 py-1.5">
            <div class="flex items-center justify-between gap-2 text-xs">
              <span class="truncate text-ink">{m.label}</span>
              {#if p?.error}
                <span class="flex items-center gap-1 text-danger" title={p.error}>
                  <Icon name="alertTriangle" size={12} />Failed
                </span>
              {:else if p?.done}
                <span class="flex items-center gap-1 text-brand"><Icon name="checkCircle" size={12} />Ready</span>
              {:else if active}
                <span class="font-mono text-ink-dim">
                  {p.total ? `${pctOf(p)}%` : `${Math.round(p.downloaded / 1e6)} MB`}
                </span>
              {:else}
                <span class="text-ink-faint">~{m.approx_mb} MB</span>
              {/if}
            </div>
            {#if active}
              <div class="h-1 w-full overflow-hidden rounded-full bg-surface-2">
                <div
                  class="h-full rounded-full bg-brand transition-[width] duration-200"
                  class:animate-pulse={!p.total}
                  style="width:{p.total ? pctOf(p) : 100}%"
                ></div>
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <div class="flex gap-1.5">
        <button class="btn flex-1" disabled={store.downloadingModels} onclick={downloadAndRun}>
          {#if store.downloadingModels}
            <Icon name="refresh" size={14} class="animate-spin" />Downloading…
          {:else}
            <Icon name="download" size={14} />Download & run
          {/if}
        </button>
        <button class="btn-sm" disabled={store.downloadingModels} onclick={runAnyway}>Run anyway</button>
        <button class="btn-sm" disabled={store.downloadingModels} onclick={() => (pendingRun = null)}>
          Cancel
        </button>
      </div>
    </section>
  {/if}

  {#if active}
    <section class="rounded-[8px] border border-line bg-surface p-3 shadow-[var(--shadow-sm)]">
      <div class="mb-2.5 flex items-center justify-between gap-2">
        <span class="flex min-w-0 items-center gap-2">
          {#if active.status === 'running'}
            <span
              class="h-3.5 w-3.5 shrink-0 rounded-full border-2"
              style="border-color: color-mix(in srgb, var(--color-brand) 30%, transparent); border-top-color: var(--color-brand); animation: pa-spin 0.7s linear infinite;"
            ></span>
          {/if}
          <span class="truncate text-ink">
            {#if active.status === 'running' && curStep}
              {curStep.label}…
            {:else if active.status === 'running'}
              Processing…
            {:else}
              {active.status}
            {/if}
          </span>
        </span>
        <span class="shrink-0 text-xs tabular-nums text-ink-dim">
          {active.completed + active.failed} / {active.total}
        </span>
      </div>

      <div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-2">
        <div class="h-full rounded-full bg-brand transition-[width] duration-[180ms]" style="width: {pct}%"></div>
      </div>

      <div class="mt-2 flex items-center justify-between text-[11px] text-ink-dim">
        <span>
          {#if active.failed > 0}
            <span class="text-danger">{active.failed} failed</span>
          {:else}
            {pct}% complete
          {/if}
        </span>
        {#if eta}<span class="tabular-nums">{eta}</span>{/if}
      </div>

      {#if active.status === 'running' && curStep?.model}
        <div class="relative mt-2.5 h-1 w-full overflow-hidden rounded-full bg-surface-2">
          <div
            class="absolute inset-y-0 w-2/5 rounded-full"
            style="background: var(--color-info); animation: pa-indeterminate 1.1s ease-in-out infinite;"
          ></div>
        </div>
        <p class="mt-1 flex items-center gap-1.5 text-[11px] text-ink-dim">
          <Icon name="download" size={12} />
          Loading {curStep.label} model — downloads on first use.
        </p>
      {/if}

      {#if active.status === 'running'}
        <button class="btn-sm mt-2.5 w-full" onclick={() => store.cancelActive()}>Cancel</button>
      {/if}
    </section>
  {/if}
</div>

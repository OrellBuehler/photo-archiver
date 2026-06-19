<script lang="ts">
  import { store } from '../store.svelte'
  import { PIPELINE_STEPS } from '../types'

  let selectedSteps = $state<Set<string>>(new Set(['organize', 'crop', 'restore_color']))

  function toggle(key: string) {
    const next = new Set(selectedSteps)
    if (next.has(key)) next.delete(key)
    else next.add(key)
    selectedSteps = next
  }

  const active = $derived(store.activeTask)
  const pct = $derived(
    active && active.total > 0
      ? Math.round(((active.completed + active.failed) / active.total) * 100)
      : 0,
  )

  async function run(all: boolean) {
    await store.startBatch([...selectedSteps], all)
  }
</script>

<div class="flex h-full flex-col gap-4 overflow-auto bg-base p-3 text-sm">
  <section>
    <h2 class="mb-2 text-xs font-semibold uppercase tracking-wide text-ink-dim">Steps</h2>
    <div class="flex flex-col gap-1">
      {#each PIPELINE_STEPS as step (step.key)}
        <label class="flex cursor-pointer items-start gap-2 rounded px-2 py-1.5 hover:bg-surface">
          <input
            type="checkbox"
            checked={selectedSteps.has(step.key)}
            onchange={() => toggle(step.key)}
            class="mt-0.5 accent-[var(--color-focus)]"
          />
          <span>
            <span class="text-ink">{step.label}</span>
            <span class="block text-xs text-ink-dim">{step.hint}</span>
          </span>
        </label>
      {/each}
    </div>
  </section>

  <section class="flex flex-col gap-2">
    <button
      class="btn"
      disabled={store.busy || selectedSteps.size === 0 || store.selected.size === 0}
      onclick={() => run(false)}
    >
      Process selected ({store.selected.size})
    </button>
    <button
      class="btn-sm"
      disabled={store.busy || selectedSteps.size === 0}
      onclick={() => run(true)}
    >
      Process all ({store.total})
    </button>
  </section>

  {#if active}
    <section class="rounded border border-line bg-surface p-3">
      <div class="mb-2 flex items-center justify-between text-xs">
        <span class="text-ink">
          {active.status === 'running' ? 'Processing…' : active.status}
        </span>
        <span class="text-ink-dim">{active.completed + active.failed} / {active.total}</span>
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-2">
        <div class="h-full bg-focus transition-all" style="width: {pct}%"></div>
      </div>
      {#if active.currentStep}
        <p class="mt-2 text-xs text-ink-dim">Step: {active.currentStep}</p>
      {/if}
      {#if active.failed > 0}
        <p class="mt-1 text-xs text-red-400">{active.failed} failed</p>
      {/if}
      {#if active.status === 'running'}
        <button class="btn-sm mt-2 w-full" onclick={() => store.cancelActive()}>Cancel</button>
      {/if}
    </section>
  {/if}
</div>

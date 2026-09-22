<script lang="ts">
  import { store } from '../store.svelte'
  import { PIPELINE_STEPS } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Badge from '../ui/Badge.svelte'
  import Spinner from '../ui/Spinner.svelte'
  import Progress from '../ui/Progress.svelte'

  let open = $state(false)

  const t = $derived(store.activeTask)
  const done = $derived(t ? t.completed + t.failed : 0)
  const pct = $derived(t && t.total ? (done / t.total) * 100 : 0)
  const stepLabel = $derived(PIPELINE_STEPS.find((s) => s.key === t?.currentStep)?.label ?? 'Starting')

  let now = $state(Date.now())
  $effect(() => {
    if (!t) return
    const i = setInterval(() => (now = Date.now()), 1000)
    return () => clearInterval(i)
  })

  const eta = $derived.by(() => {
    if (!t || done <= 0) return null
    const secs = Math.round(((now - t.startedAt) / 1000 / done) * Math.max(0, t.total - done))
    if (secs < 60) return `~${secs}s`
    return `~${Math.floor(secs / 60)}m ${secs % 60}s`
  })

  const stepName = (k: string) => PIPELINE_STEPS.find((s) => s.key === k)?.label ?? k

  function when(ts: string) {
    const d = new Date(ts.replace(' ', 'T') + (ts.endsWith('Z') ? '' : 'Z'))
    if (Number.isNaN(d.getTime())) return ts.slice(0, 16)
    return d.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
  }
</script>

<footer class="relative flex h-7 shrink-0 items-center gap-4 border-t border-line bg-bar px-3 text-[11px] text-ink-dim select-none">
  {#if store.settings?.source_dir}
    <span class="tabular-nums">
      {#if store.hasFilters}
        <b class="font-medium text-ink">{store.total}</b> of {store.libraryTotal} photos
      {:else}
        <b class="font-medium text-ink">{store.libraryTotal}</b> photos
      {/if}
    </span>
    {#if store.selected.size > 0}
      <span class="flex items-center gap-1.5 text-focus">
        <Icon name="selectAll" size={12} />
        {store.selected.size} selected
      </span>
    {/if}
  {/if}

  <button
    class="ml-auto flex h-full items-center gap-2 px-2 transition-colors hover:bg-surface-2 hover:text-ink"
    class:bg-surface-2={open}
    onclick={() => {
      open = !open
      if (open) store.loadTasks()
    }}
  >
    {#if t}
      <Spinner size={11} />
      <span class="text-ink">{stepLabel}</span>
      <span class="w-28"><Progress value={pct} height={3} /></span>
      <span class="tabular-nums">{done}/{t.total}</span>
      {#if eta}<span class="tabular-nums text-ink-faint">{eta}</span>{/if}
    {:else if store.downloadingModels}
      <Spinner size={11} color="var(--color-info)" />
      Downloading models…
    {:else if store.scanning}
      <Spinner size={11} />
      Scanning folder…
    {:else}
      <Icon name="activity" size={12} />
      Activity
    {/if}
  </button>

  {#if open}
    <button class="fixed inset-0 z-30 cursor-default" aria-label="Close activity" onclick={() => (open = false)}></button>
    <div class="pop absolute right-2 bottom-[calc(100%+6px)] z-40 flex max-h-[60vh] w-[360px] flex-col overflow-hidden rounded-[10px] border border-line bg-surface shadow-[var(--shadow-lg)]">
      <div class="flex items-center justify-between border-b border-line px-3 py-2">
        <span class="text-xs font-semibold text-ink">Activity</span>
        {#if t}
          <button class="btn-danger" onclick={() => store.cancelActive()}>
            <Icon name="stop" size={11} />Cancel run
          </button>
        {/if}
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto">
        {#if store.tasks.length === 0}
          <p class="px-3 py-6 text-center text-xs text-ink-faint">No restore runs yet.</p>
        {:else}
          {#each store.tasks as task (task.id)}
            <div class="flex flex-col gap-1 border-b border-line/60 px-3 py-2.5 last:border-b-0">
              <div class="flex items-center justify-between gap-2">
                <Badge status={task.status} />
                <span class="text-[11px] text-ink-faint tabular-nums">{when(task.created_at)}</span>
              </div>
              <p class="truncate text-xs text-ink-dim" title={task.steps.map(stepName).join(' → ')}>
                {task.steps.map(stepName).join(' → ')}
              </p>
              <p class="text-[11px] text-ink-faint tabular-nums">
                {task.completed_images}/{task.total_images} photos{#if task.failed_images}<span class="text-danger"> · {task.failed_images} failed</span>{/if}
              </p>
              {#if task.error_message}
                <p class="truncate text-[11px] text-danger" title={task.error_message}>{task.error_message}</p>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</footer>

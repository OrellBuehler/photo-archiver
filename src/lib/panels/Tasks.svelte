<script lang="ts">
  import { onMount } from 'svelte'
  import { store } from '../store.svelte'
  import Badge from '../ui/Badge.svelte'

  onMount(() => {
    store.loadTasks()
  })

  function when(t: string | null) {
    if (!t) return ''
    return t.replace('T', ' ').slice(0, 16)
  }
</script>

<div class="flex h-full flex-col gap-2 overflow-auto bg-base p-3 text-sm">
  <div class="flex items-center justify-between">
    <h2 class="eyebrow">Tasks</h2>
    <button class="text-xs text-ink-dim hover:text-ink" onclick={() => store.loadTasks()}>
      Refresh
    </button>
  </div>

  {#if store.tasks.length === 0}
    <p class="text-ink-dim">No tasks yet.</p>
  {:else}
    <div class="flex flex-col gap-1.5">
      {#each store.tasks as task (task.id)}
        <div class="rounded-[8px] border border-line bg-surface p-2.5">
          <div class="flex items-center justify-between gap-2">
            <Badge status={task.status} />
            <span class="text-xs tabular-nums text-ink-faint">#{task.id} · {when(task.created_at)}</span>
          </div>
          <p class="mt-1.5 truncate text-xs text-ink-dim" title={task.steps.join(', ')}>
            {task.steps.join(' › ')}
          </p>
          <p class="mt-0.5 text-xs text-ink-dim tabular-nums">
            {task.completed_images}/{task.total_images} done{task.failed_images
              ? ` · ${task.failed_images} failed`
              : ''}
          </p>
          {#if task.status === 'running'}
            <button class="btn-sm mt-1.5" onclick={() => store.cancelActive()}>Cancel</button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

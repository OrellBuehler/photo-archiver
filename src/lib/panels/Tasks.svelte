<script lang="ts">
  import { onMount } from 'svelte'
  import { store } from '../store.svelte'

  onMount(() => {
    store.loadTasks()
  })

  function badge(status: string) {
    switch (status) {
      case 'completed':
        return 'text-green-400'
      case 'failed':
        return 'text-red-400'
      case 'cancelled':
        return 'text-yellow-400'
      case 'running':
        return 'text-focus'
      default:
        return 'text-ink-dim'
    }
  }

  function when(t: string | null) {
    if (!t) return ''
    return t.replace('T', ' ').slice(0, 16)
  }
</script>

<div class="flex h-full flex-col gap-2 overflow-auto bg-base p-3 text-sm">
  <div class="flex items-center justify-between">
    <h2 class="text-xs font-semibold uppercase tracking-wide text-ink-dim">Tasks</h2>
    <button class="text-xs text-ink-dim hover:text-ink" onclick={() => store.loadTasks()}>
      Refresh
    </button>
  </div>

  {#if store.tasks.length === 0}
    <p class="text-ink-dim">No tasks yet.</p>
  {:else}
    <div class="flex flex-col gap-1.5">
      {#each store.tasks as task (task.id)}
        <div class="rounded border border-line bg-surface p-2">
          <div class="flex items-center justify-between">
            <span class="font-medium {badge(task.status)}">{task.status}</span>
            <span class="text-xs text-ink-dim">#{task.id} · {when(task.created_at)}</span>
          </div>
          <p class="mt-1 truncate text-xs text-ink-dim" title={task.steps.join(', ')}>
            {task.steps.join(' › ')}
          </p>
          <p class="mt-0.5 text-xs text-ink-dim">
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

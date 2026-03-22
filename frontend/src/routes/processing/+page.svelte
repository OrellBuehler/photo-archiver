<script lang="ts">
  import { onMount } from 'svelte';
  import { getTasks, cancelTask, createBatchTask } from '$lib/api';
  import type { Task } from '$lib/types';
  import ProgressTracker from '$lib/components/ProgressTracker.svelte';

  let tasks = $state<Task[]>([]);
  let loading = $state(true);
  let activeTaskId = $state<number | null>(null);

  let steps = $state({ organize: true, crop: false, auto_orient: false, deskew: false, restore_color: false, remove_dust: false, remove_lines: false, enhance: false });
  let processing = $state(false);

  async function loadTasks() {
    loading = true;
    try {
      tasks = await getTasks();
      const running = tasks.find(t => t.status === 'running');
      if (running) activeTaskId = running.id;
    } finally {
      loading = false;
    }
  }

  async function processAll() {
    const selectedSteps = Object.entries(steps).filter(([_, v]) => v).map(([k]) => k);
    if (selectedSteps.length === 0) return;
    processing = true;
    try {
      const task = await createBatchTask('all', selectedSteps);
      activeTaskId = task.id;
      await loadTasks();
    } finally {
      processing = false;
    }
  }

  async function handleCancel(taskId: number) {
    await cancelTask(taskId);
    await loadTasks();
  }

  function formatDate(d: string | null): string {
    if (!d) return '—';
    return new Date(d + 'Z').toLocaleString();
  }

  function statusColor(s: string): string {
    switch (s) {
      case 'completed': return 'bg-green-100 text-green-700';
      case 'running': return 'bg-blue-100 text-blue-700';
      case 'failed': return 'bg-red-100 text-red-700';
      case 'cancelled': return 'bg-yellow-100 text-yellow-700';
      default: return 'bg-gray-100 text-gray-600';
    }
  }

  onMount(loadTasks);
</script>

<div class="py-4 space-y-8 h-full overflow-y-auto">
  <h1 class="text-xl font-bold">Processing</h1>

  <div class="rounded-lg border p-4 space-y-3">
    <h2 class="font-medium">Process All Images</h2>
    <div class="flex items-center gap-4">
      <label class="flex items-center gap-1.5 text-sm">
        <input type="checkbox" bind:checked={steps.organize} class="rounded" />
        Organize
      </label>
      <label class="flex items-center gap-1.5 text-sm">
        <input type="checkbox" bind:checked={steps.crop} class="rounded" />
        Crop
      </label>
      <label class="flex items-center gap-1.5 text-sm">
        <input type="checkbox" bind:checked={steps.auto_orient} class="rounded" />
        Orient
      </label>
      <label class="flex items-center gap-1.5 text-sm">
        <input type="checkbox" bind:checked={steps.deskew} class="rounded" />
        Deskew
      </label>
      <label class="flex items-center gap-1.5 text-sm">
        <input type="checkbox" bind:checked={steps.restore_color} class="rounded" />
        Restore Color
      </label>
      <label class="flex items-center gap-1.5 text-sm">
        <input type="checkbox" bind:checked={steps.remove_dust} class="rounded" />
        Remove Dust
      </label>
      <label class="flex items-center gap-1.5 text-sm">
        <input type="checkbox" bind:checked={steps.remove_lines} class="rounded" />
        Remove Lines
      </label>
      <label class="flex items-center gap-1.5 text-sm">
        <input type="checkbox" bind:checked={steps.enhance} class="rounded" />
        Enhance
      </label>
      <button
        class="ml-auto rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium disabled:opacity-50"
        disabled={processing}
        onclick={processAll}
      >
        {processing ? 'Starting...' : 'Process All'}
      </button>
    </div>
  </div>

  {#if activeTaskId}
    <div class="rounded-lg border p-4">
      <h2 class="font-medium mb-3">Active Task #{activeTaskId}</h2>
      <ProgressTracker taskId={activeTaskId} />
    </div>
  {/if}

  <div class="rounded-lg border">
    <div class="p-4 border-b">
      <h2 class="font-medium">Task History</h2>
    </div>
    {#if loading}
      <div class="p-4 space-y-3">
        {#each Array(3) as _}
          <div class="h-12 bg-muted animate-pulse rounded"></div>
        {/each}
      </div>
    {:else if tasks.length === 0}
      <p class="p-4 text-sm text-muted-foreground">No tasks yet</p>
    {:else}
      <div class="divide-y">
        {#each tasks as task, i}
          <div class="p-4 flex items-center gap-4
            {task.status === 'failed' ? 'border-l-2 border-l-red-300' : task.status === 'cancelled' ? 'border-l-2 border-l-yellow-300' : ''}
            {task.status === 'completed' && i >= 5 ? 'opacity-60' : ''}">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium">Task #{task.id}</span>
                <span class="text-xs px-2 py-0.5 rounded-full {statusColor(task.status)}">{task.status}</span>
              </div>
              <p class="text-xs text-muted-foreground mt-1">
                Steps: {task.steps.join(' → ')} ·
                {task.completed_images}/{task.total_images} done
                {#if task.failed_images > 0}, {task.failed_images} failed{/if}
              </p>
              <p class="text-xs text-muted-foreground">{formatDate(task.created_at)}</p>
            </div>
            {#if task.status === 'running' || task.status === 'pending'}
              <button
                class="rounded-md border border-destructive text-destructive px-3 py-1.5 text-xs hover:bg-destructive/10"
                onclick={() => handleCancel(task.id)}
              >Cancel</button>
            {/if}
            {#if task.status === 'running'}
              <button
                class="rounded-md border px-3 py-1.5 text-xs hover:bg-secondary"
                onclick={() => activeTaskId = task.id}
              >Track</button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { subscribe } from '$lib/ws';
  import type { ProgressMessage } from '$lib/types';

  let { taskId }: { taskId?: number } = $props();

  let messages = $state<ProgressMessage[]>([]);
  let progress = $state(0);
  let status = $state<string>('pending');
  let unsubscribe: (() => void) | null = null;

  onMount(() => {
    unsubscribe = subscribe((msg) => {
      if (taskId && msg.task_id !== taskId) return;

      messages = [...messages.slice(-99), msg];

      if (msg.progress !== undefined) {
        progress = msg.progress;
      }
      if (msg.type === 'task_completed') {
        status = msg.status || 'completed';
      }
      if (msg.type === 'task_failed') {
        status = 'failed';
      }
      if (msg.type === 'task_started') {
        status = 'running';
      }
    });
  });

  onDestroy(() => {
    unsubscribe?.();
  });
</script>

<div class="space-y-3">
  <div class="flex items-center gap-2">
    <span class="text-sm font-medium">
      {#if status === 'running'}
        Processing...
      {:else if status === 'completed'}
        Complete
      {:else if status === 'failed'}
        Failed
      {:else}
        Pending
      {/if}
    </span>
    <span class="text-xs text-muted-foreground">{Math.round(progress * 100)}%</span>
  </div>

  <div class="h-2 rounded-full bg-secondary overflow-hidden">
    <div
      class="h-full rounded-full transition-all duration-300 {status === 'failed' ? 'bg-destructive' : status === 'completed' ? 'bg-green-500' : 'bg-primary'}"
      style="width: {progress * 100}%"
    ></div>
  </div>

  <div class="max-h-48 overflow-y-auto rounded-md border bg-muted/50 p-3 space-y-1">
    {#each messages as msg}
      <p class="text-xs text-muted-foreground font-mono">
        {#if msg.type === 'step_started'}
          ▶ Image {msg.image_id}: {msg.step}
        {:else if msg.type === 'step_completed'}
          ✓ Image {msg.image_id}: {msg.step} done
        {:else if msg.type === 'image_started'}
          → Processing image {msg.image_id}
        {:else if msg.type === 'task_completed'}
          ✓ Task completed ({msg.status})
        {:else if msg.type === 'task_failed'}
          ✗ Task failed: {msg.error}
        {:else if msg.type === 'progress'}
          {Math.round((msg.progress || 0) * 100)}% complete
        {:else}
          {msg.type}
        {/if}
      </p>
    {/each}
  </div>
</div>

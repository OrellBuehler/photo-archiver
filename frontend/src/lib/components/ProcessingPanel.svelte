<script lang="ts">
  import { createBatchTask } from '$lib/api';

  let { selectedIds, onTaskCreated }: {
    selectedIds: Set<number>;
    onTaskCreated?: (taskId: number) => void;
  } = $props();

  let steps = $state({ organize: true, orient: true, enhance: false });
  let processing = $state(false);

  let selectedSteps = $derived(
    Object.entries(steps).filter(([_, v]) => v).map(([k]) => k)
  );

  async function startProcessing() {
    if (selectedSteps.length === 0) return;
    processing = true;
    try {
      const ids = selectedIds.size > 0 ? [...selectedIds] : 'all' as const;
      const task = await createBatchTask(ids, selectedSteps);
      onTaskCreated?.(task.id);
    } finally {
      processing = false;
    }
  }
</script>

{#if selectedIds.size > 0}
  <div class="fixed bottom-0 left-0 right-0 border-t bg-background p-4 shadow-lg z-50">
    <div class="container mx-auto flex items-center gap-4">
      <span class="text-sm font-medium">{selectedIds.size} selected</span>

      <div class="flex items-center gap-3">
        <label class="flex items-center gap-1.5 text-sm">
          <input type="checkbox" bind:checked={steps.organize} class="rounded" />
          Organize
        </label>
        <label class="flex items-center gap-1.5 text-sm">
          <input type="checkbox" bind:checked={steps.orient} class="rounded" />
          Orient
        </label>
        <label class="flex items-center gap-1.5 text-sm">
          <input type="checkbox" bind:checked={steps.enhance} class="rounded" />
          Enhance
        </label>
      </div>

      <button
        class="ml-auto rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium disabled:opacity-50"
        disabled={processing || selectedSteps.length === 0}
        onclick={startProcessing}
      >
        {processing ? 'Starting...' : 'Process'}
      </button>
    </div>
  </div>
{/if}

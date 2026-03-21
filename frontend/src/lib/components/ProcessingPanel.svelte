<script lang="ts">
  import { createBatchTask, bulkDeleteImages, bulkUpdateImages } from '$lib/api';

  let { selectedIds, totalOnPage, pageImageIds, onTaskCreated, onSelectionChange, onRefresh }: {
    selectedIds: Set<number>;
    totalOnPage?: number;
    pageImageIds?: number[];
    onTaskCreated?: (taskId: number) => void;
    onSelectionChange?: (ids: Set<number>) => void;
    onRefresh?: () => void;
  } = $props();

  let steps = $state({ organize: true, crop: false, auto_orient: false, deskew: false, restore_color: false, remove_dust: false, enhance: false });
  let processing = $state(false);
  let deleting = $state(false);
  let saving = $state(false);
  let showEdit = $state(false);

  let editYear = $state('');
  let editMonth = $state('');
  let editTitle = $state('');

  let selectedSteps = $derived(
    Object.entries(steps).filter(([_, v]) => v).map(([k]) => k)
  );

  let allSelected = $derived(
    pageImageIds != null && pageImageIds.length > 0 && selectedIds.size >= pageImageIds.length
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

  async function handleDelete() {
    if (!window.confirm(`Delete ${selectedIds.size} image${selectedIds.size !== 1 ? 's' : ''}? This cannot be undone.`)) return;
    deleting = true;
    try {
      await bulkDeleteImages([...selectedIds]);
      onSelectionChange?.(new Set());
      onRefresh?.();
    } finally {
      deleting = false;
    }
  }

  async function handleSaveEdit() {
    const data: { year?: number; month?: number; title?: string } = {};
    if (editYear.trim()) data.year = Number(editYear);
    if (editMonth.trim()) data.month = Number(editMonth);
    if (editTitle.trim()) data.title = editTitle.trim();
    if (Object.keys(data).length === 0) return;
    saving = true;
    try {
      await bulkUpdateImages([...selectedIds], data);
      showEdit = false;
      editYear = '';
      editMonth = '';
      editTitle = '';
      onSelectionChange?.(new Set());
      onRefresh?.();
    } finally {
      saving = false;
    }
  }

  function toggleSelectAll() {
    if (allSelected) {
      onSelectionChange?.(new Set());
    } else {
      onSelectionChange?.(new Set(pageImageIds ?? []));
    }
  }
</script>

{#if selectedIds.size > 0}
  <div class="fixed bottom-0 left-0 right-0 border-t bg-background p-4 shadow-lg z-50">
    <div class="container mx-auto flex flex-wrap items-center gap-4">
      <div class="flex items-center gap-2 shrink-0">
        <span class="text-sm font-medium">{selectedIds.size} selected</span>
        {#if pageImageIds != null && pageImageIds.length > 0}
          <button
            class="text-sm text-primary underline-offset-2 hover:underline"
            onclick={toggleSelectAll}
          >
            {allSelected ? 'Deselect All' : 'Select All'}
          </button>
        {/if}
      </div>

      <div class="flex items-center gap-3">
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
          <input type="checkbox" bind:checked={steps.enhance} class="rounded" />
          Enhance
        </label>
        <button
          class="rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium disabled:opacity-50"
          disabled={processing || selectedSteps.length === 0}
          onclick={startProcessing}
        >
          {processing ? 'Starting...' : 'Process'}
        </button>
      </div>

      <div class="ml-auto flex items-center gap-2">
        <button
          class="rounded-md border px-4 py-2 text-sm font-medium hover:bg-muted disabled:opacity-50"
          disabled={saving || deleting}
          onclick={() => { showEdit = !showEdit; }}
        >
          Edit
        </button>
        <button
          class="rounded-md bg-destructive text-destructive-foreground px-4 py-2 text-sm font-medium disabled:opacity-50"
          disabled={deleting || processing}
          onclick={handleDelete}
        >
          {deleting ? 'Deleting...' : 'Delete'}
        </button>
      </div>
    </div>

    {#if showEdit}
      <div class="container mx-auto mt-3 flex flex-wrap items-end gap-3 border-t pt-3">
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">Year</span>
          <input
            type="number"
            bind:value={editYear}
            placeholder="e.g. 2023"
            class="rounded-md border px-2 py-1.5 text-sm w-28 bg-background"
          />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">Month</span>
          <input
            type="number"
            bind:value={editMonth}
            placeholder="1–12"
            min="1"
            max="12"
            class="rounded-md border px-2 py-1.5 text-sm w-20 bg-background"
          />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-xs text-muted-foreground">Title</span>
          <input
            type="text"
            bind:value={editTitle}
            placeholder="Optional title"
            class="rounded-md border px-2 py-1.5 text-sm w-48 bg-background"
          />
        </label>
        <button
          class="rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium disabled:opacity-50"
          disabled={saving || (!editYear.trim() && !editMonth.trim() && !editTitle.trim())}
          onclick={handleSaveEdit}
        >
          {saving ? 'Saving...' : 'Save'}
        </button>
        <button
          class="rounded-md border px-4 py-2 text-sm font-medium hover:bg-muted"
          onclick={() => { showEdit = false; editYear = ''; editMonth = ''; editTitle = ''; }}
        >
          Cancel
        </button>
      </div>
    {/if}
  </div>
{/if}

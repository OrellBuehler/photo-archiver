<script lang="ts">
  import type { Image } from '$lib/types';
  import { imageFileUrl, updateImage, rotateImage, createBatchTask, getTask, getImage, getImageHistory } from '$lib/api';
  import type { ImageHistory } from '$lib/types';
  import { onMount } from 'svelte';
  import BeforeAfter from '$lib/components/BeforeAfter.svelte';
  import { toast } from 'svelte-sonner';

  let { image: initialImage }: { image: Image } = $props();

  let image = $state(initialImage);
  let editing = $state(false);
  let saving = $state(false);

  let editYear = $state(image.year ?? undefined);
  let editMonth = $state(image.month ?? undefined);
  let editTitle = $state(image.title ?? '');

  let activeVariant = $state<string>(initialImage.enhanced_path ? 'enhanced' : initialImage.organized_path ? 'organized' : 'source');
  let showCompare = $state(false);
  let rotating = $state(false);
  let processing = $state(false);
  let cacheBust = $state(Date.now());
  let busy = $derived(rotating || processing);
  let history = $state<ImageHistory[]>([]);

  async function loadHistory() {
    try { history = await getImageHistory(image.id); } catch {}
  }

  function switchToBestVariant() {
    if (image.enhanced_path) activeVariant = 'enhanced';
    else if (image.organized_path) activeVariant = 'organized';
  }

  onMount(() => { loadHistory(); });

  async function handleRotate(direction: 'left' | 'right') {
    rotating = true;
    toast.info(`Rotating ${direction}...`);
    try {
      image = await rotateImage(image.id, direction);
      cacheBust = Date.now();
      switchToBestVariant();
      toast.success('Rotated');
      loadHistory();
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'unknown error';
      toast.error(msg.includes('409') ? 'Image is currently being processed' : `Rotation failed: ${msg}`);
    } finally {
      rotating = false;
    }
  }

  async function handleProcess(steps: string[]) {
    const label = steps.map(s => processSteps.find(p => p.key === s)?.label ?? s).join(', ');
    processing = true;
    toast.info(`Starting ${label}...`);
    try {
      const task = await createBatchTask([image.id], steps);
      await pollTask(task.id);
      image = await getImage(image.id);
      cacheBust = Date.now();
      switchToBestVariant();
      toast.success(`${label} complete`);
      loadHistory();
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'unknown error';
      toast.error(msg.includes('409') ? 'Image is currently being processed' : `${label} failed: ${msg}`);
    } finally {
      processing = false;
    }
  }

  async function pollTask(taskId: number) {
    while (true) {
      await new Promise(r => setTimeout(r, 1000));
      const task = await getTask(taskId);
      if (task.status === 'completed') return;
      if (task.status === 'failed') throw new Error(task.error_message ?? 'Task failed');
      if (task.status === 'cancelled') throw new Error('Task cancelled');
    }
  }

  let variants = $derived(() => {
    const v: { key: string; label: string }[] = [{ key: 'source', label: 'Source' }];
    if (image.organized_path) v.push({ key: 'organized', label: 'Organized' });
    if (image.enhanced_path) v.push({ key: 'enhanced', label: 'Enhanced' });
    return v;
  });

  function startEdit() {
    editYear = image.year ?? undefined;
    editMonth = image.month ?? undefined;
    editTitle = image.title ?? '';
    editing = true;
  }

  async function saveEdit() {
    saving = true;
    try {
      const data: Record<string, any> = {};
      if (editYear !== undefined) data.year = editYear;
      if (editMonth !== undefined) data.month = editMonth;
      if (editTitle) data.title = editTitle;
      image = await updateImage(image.id, data);
      editing = false;
    } finally {
      saving = false;
    }
  }

  function formatSize(bytes: number | null): string {
    if (!bytes) return '—';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  const processSteps = [
    { key: 'organize', label: 'Organize' },
    { key: 'auto_orient', label: 'Orient' },
    { key: 'deskew', label: 'Deskew' },
    { key: 'restore_color', label: 'Restore Color' },
    { key: 'remove_dust', label: 'Remove Dust' },
    { key: 'enhance', label: 'Enhance' },
  ];
</script>

<div class="flex flex-col lg:flex-row gap-6 h-full py-4">
  <!-- Image viewer -->
  <div class="flex-1 min-w-0 flex flex-col min-h-0">
    {#if variants().length > 1}
      <div class="shrink-0 flex gap-1 mb-3">
        {#each variants() as v}
          <button
            class="rounded-md px-3 py-1.5 text-sm transition-colors {activeVariant === v.key && !showCompare ? 'bg-primary text-primary-foreground' : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'}"
            onclick={() => { activeVariant = v.key; showCompare = false; }}
          >{v.label}</button>
        {/each}
        {#if image.enhanced_path}
          <button
            class="rounded-md px-3 py-1.5 text-sm transition-colors {showCompare ? 'bg-primary text-primary-foreground' : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'}"
            onclick={() => showCompare = !showCompare}
          >Compare</button>
        {/if}
      </div>
    {/if}
    {#if showCompare && image.enhanced_path}
      <div class="flex-1 min-h-0">
        <BeforeAfter
          beforeSrc={imageFileUrl(image.id, 'source')}
          afterSrc={imageFileUrl(image.id, 'enhanced')}
          beforeLabel="Source"
          afterLabel="Enhanced"
        />
      </div>
    {:else}
      <div class="flex-1 min-h-0 flex items-center justify-center rounded-lg border overflow-hidden bg-muted">
        <img
          src="{imageFileUrl(image.id, activeVariant)}&t={cacheBust}"
          alt={image.title || image.filename}
          class="max-w-full max-h-full object-contain"
        />
      </div>
    {/if}
    <div class="shrink-0 flex gap-2 mt-3">
      <button
        class="rounded-md border px-3 py-1.5 text-sm hover:bg-secondary transition-colors disabled:opacity-50"
        disabled={busy}
        onclick={() => handleRotate('left')}
      >↶ Rotate Left</button>
      <button
        class="rounded-md border px-3 py-1.5 text-sm hover:bg-secondary transition-colors disabled:opacity-50"
        disabled={busy}
        onclick={() => handleRotate('right')}
      >↷ Rotate Right</button>
    </div>
  </div>

  <!-- Sidebar -->
  <div class="lg:w-80 shrink-0 overflow-y-auto space-y-4">
    <div>
      <h2 class="text-lg font-semibold">{image.title || image.scan_id || image.filename}</h2>
      <p class="text-sm text-muted-foreground">{image.source_path}</p>
    </div>

    <div class="rounded-lg border p-4 space-y-3">
      <h3 class="font-medium text-sm">Processing</h3>
      <div class="flex flex-wrap gap-1.5">
        {#each processSteps as step}
          <button
            class="rounded-md border px-2.5 py-1 text-xs hover:bg-secondary transition-colors disabled:opacity-50"
            disabled={busy}
            onclick={() => handleProcess([step.key])}
          >{step.label}</button>
        {/each}
      </div>
    </div>

    <div class="rounded-lg border p-4 space-y-3">
      <h3 class="font-medium text-sm">Details</h3>

      {#if !editing}
        <dl class="space-y-2 text-sm">
          <div class="flex justify-between">
            <dt class="text-muted-foreground">Year</dt>
            <dd>{image.year ?? '—'}</dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">Month</dt>
            <dd>{image.month ?? '—'}</dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">Title</dt>
            <dd>{image.title ?? '—'}</dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">Scan ID</dt>
            <dd>{image.scan_id ?? '—'}</dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">Dimensions</dt>
            <dd>{image.width && image.height ? `${image.width} × ${image.height}` : '—'}</dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">File size</dt>
            <dd>{formatSize(image.file_size)}</dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">Status</dt>
            <dd>
              <span class="px-2 py-0.5 rounded-full text-xs {image.status === 'enhanced' ? 'bg-green-100 text-green-700' : image.status === 'organized' ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-600'}">{image.status}</span>
            </dd>
          </div>
        </dl>
        <button
          class="w-full rounded-md border px-3 py-1.5 text-sm hover:bg-secondary transition-colors"
          onclick={startEdit}
        >Edit Metadata</button>
      {:else}
        <form onsubmit={(e) => { e.preventDefault(); saveEdit(); }} class="space-y-3">
          <div>
            <label class="text-sm text-muted-foreground" for="edit-year">Year</label>
            <input id="edit-year" type="number" class="w-full rounded-md border bg-background px-3 py-1.5 text-sm" bind:value={editYear} />
          </div>
          <div>
            <label class="text-sm text-muted-foreground" for="edit-month">Month</label>
            <input id="edit-month" type="number" min="1" max="12" class="w-full rounded-md border bg-background px-3 py-1.5 text-sm" bind:value={editMonth} />
          </div>
          <div>
            <label class="text-sm text-muted-foreground" for="edit-title">Title</label>
            <input id="edit-title" type="text" class="w-full rounded-md border bg-background px-3 py-1.5 text-sm" bind:value={editTitle} />
          </div>
          <div class="flex gap-2">
            <button type="submit" class="flex-1 rounded-md bg-primary text-primary-foreground px-3 py-1.5 text-sm disabled:opacity-50" disabled={saving}>
              {saving ? 'Saving...' : 'Save'}
            </button>
            <button type="button" class="flex-1 rounded-md border px-3 py-1.5 text-sm hover:bg-secondary" onclick={() => editing = false}>
              Cancel
            </button>
          </div>
        </form>
      {/if}
    </div>

    <div class="rounded-lg border p-4 space-y-3">
      <h3 class="font-medium text-sm">History</h3>
      {#if history.length === 0}
        <p class="text-xs text-muted-foreground">No processing history</p>
      {:else}
        <div class="space-y-1.5">
          {#each history as entry}
            <div class="flex justify-between text-xs">
              <span>{entry.step.replace(/_/g, ' ')}</span>
              <span class="text-muted-foreground">{entry.created_at ? new Date(entry.created_at + 'Z').toLocaleString() : ''}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <a href="/" class="block text-center text-sm text-muted-foreground hover:text-foreground transition-colors">
      ← Back to Gallery
    </a>
  </div>
</div>

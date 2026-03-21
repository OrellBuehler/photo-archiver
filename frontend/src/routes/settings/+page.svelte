<script lang="ts">
  import { onMount } from 'svelte';
  import { getSettings, updateSettings } from '$lib/api';
  import type { AppSettings } from '$lib/types';

  let settings = $state<AppSettings | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let saved = $state(false);

  let thumbnailSize = $state(0);
  let device = $state('cpu');

  async function load() {
    loading = true;
    try {
      settings = await getSettings();
      thumbnailSize = settings.thumbnail_size;
      device = settings.device;
    } finally {
      loading = false;
    }
  }

  async function save() {
    saving = true;
    try {
      settings = await updateSettings({ thumbnail_size: thumbnailSize, device });
      saved = true;
      setTimeout(() => { saved = false; }, 2000);
    } finally {
      saving = false;
    }
  }

  onMount(load);
</script>

<div class="py-4 space-y-8">
  <h1 class="text-2xl font-bold">Settings</h1>

  {#if loading}
    <div class="rounded-lg border p-4 space-y-3">
      <div class="h-5 w-32 bg-muted animate-pulse rounded"></div>
      <div class="h-9 bg-muted animate-pulse rounded"></div>
      <div class="h-9 bg-muted animate-pulse rounded"></div>
    </div>
  {:else if settings}
    <div class="rounded-lg border p-4 space-y-3">
      <h2 class="font-medium">Directories</h2>
      <div class="space-y-2">
        <div>
          <p class="text-sm text-muted-foreground mb-1">Source Directory</p>
          <p class="text-sm font-mono bg-muted rounded px-3 py-2">{settings.source_dir}</p>
        </div>
        <div>
          <p class="text-sm text-muted-foreground mb-1">Output Directory</p>
          <p class="text-sm font-mono bg-muted rounded px-3 py-2">{settings.output_dir}</p>
        </div>
      </div>
    </div>

    <div class="rounded-lg border p-4 space-y-4">
      <h2 class="font-medium">Processing</h2>
      <div class="space-y-3">
        <div>
          <label class="text-sm text-muted-foreground block mb-1" for="thumbnail-size">Thumbnail Size</label>
          <input
            id="thumbnail-size"
            type="number"
            min="100"
            max="1000"
            step="50"
            bind:value={thumbnailSize}
            class="rounded-md border bg-background px-3 py-2 text-sm w-40"
          />
        </div>
        <div>
          <label class="text-sm text-muted-foreground block mb-1" for="device">Device</label>
          <select
            id="device"
            bind:value={device}
            class="rounded-md border bg-background px-3 py-2 text-sm w-40"
          >
            <option value="cpu">cpu</option>
            <option value="cuda">cuda</option>
          </select>
        </div>
      </div>
      <div class="flex items-center gap-3 pt-1">
        <button
          class="rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium disabled:opacity-50"
          disabled={saving}
          onclick={save}
        >
          {saving ? 'Saving...' : 'Save'}
        </button>
        {#if saved}
          <span class="text-sm text-green-600">Saved!</span>
        {/if}
      </div>
    </div>
  {/if}
</div>

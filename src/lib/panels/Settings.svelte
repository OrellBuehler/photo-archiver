<script lang="ts">
  import { store } from '../store.svelte'

  let thumbSize = $state(store.settings?.thumbnail_size ?? 400)
  let saved = $state(false)

  $effect(() => {
    if (store.settings) thumbSize = store.settings.thumbnail_size
  })

  async function saveThumb() {
    await store.setThumbnailSize(Number(thumbSize))
    saved = true
    setTimeout(() => (saved = false), 1500)
  }
</script>

<div class="flex h-full flex-col gap-5 overflow-auto bg-base p-4 text-sm">
  <h2 class="text-xs font-semibold uppercase tracking-wide text-ink-dim">Settings</h2>

  <section class="flex flex-col gap-1">
    <span class="text-ink-dim">Source folder</span>
    <div class="flex items-center gap-2">
      <code class="flex-1 truncate rounded bg-surface px-2 py-1 text-xs">
        {store.settings?.source_dir ?? '— not set —'}
      </code>
      <button class="btn-sm" onclick={() => store.pickFolder()}>Change</button>
    </div>
  </section>

  <section class="flex flex-col gap-1">
    <span class="text-ink-dim">Output folder</span>
    <div class="flex items-center gap-2">
      <code class="flex-1 truncate rounded bg-surface px-2 py-1 text-xs">
        {store.settings?.output_dir ?? '— default —'}
      </code>
      <button class="btn-sm" onclick={() => store.pickOutput()}>Change</button>
    </div>
    <span class="text-xs text-ink-dim">Organized & enhanced copies are written here.</span>
  </section>

  <section class="flex flex-col gap-1">
    <span class="text-ink-dim">Thumbnail size (px)</span>
    <div class="flex items-center gap-2">
      <input
        type="number"
        min="100"
        max="1000"
        step="50"
        bind:value={thumbSize}
        class="w-28 rounded border border-line bg-surface px-2 py-1"
      />
      <button class="btn" onclick={saveThumb}>{saved ? 'Saved ✓' : 'Save'}</button>
    </div>
  </section>

  <section class="rounded border border-line bg-surface p-3 text-xs text-ink-dim">
    <p class="mb-1 font-medium text-ink">ML models</p>
    <p>
      Smart-orient, scan-line removal and enhancement run on-device via ONNX
      Runtime (CPU). Models download on first use into the app data folder.
      GPU acceleration (CUDA / DirectML) is a build-time option.
    </p>
  </section>
</div>

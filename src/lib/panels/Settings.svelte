<script lang="ts">
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'

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

<div class="flex h-full flex-col gap-6 overflow-auto bg-base p-4 text-sm">
  <h2 class="flex items-center gap-1.5 text-sm font-semibold text-ink">
    <Icon name="settings" size={15} class="text-ink-dim" />
    Settings
  </h2>

  <section class="flex flex-col gap-1.5">
    <span class="eyebrow">Source folder</span>
    <div class="flex items-center gap-2">
      <code class="flex-1 truncate rounded-[3px] border border-line bg-surface px-2 py-1 font-mono text-xs text-ink-dim">
        {store.settings?.source_dir ?? '— not set —'}
      </code>
      <button class="btn-sm" onclick={() => store.pickFolder()}>
        <Icon name="folder" size={13} />
        Change…
      </button>
    </div>
  </section>

  <section class="flex flex-col gap-1.5">
    <span class="eyebrow">Output folder</span>
    <div class="flex items-center gap-2">
      <code class="flex-1 truncate rounded-[3px] border border-line bg-surface px-2 py-1 font-mono text-xs text-ink-dim">
        {store.settings?.output_dir ?? '— default —'}
      </code>
      <button class="btn-sm" onclick={() => store.pickOutput()}>
        <Icon name="folder" size={13} />
        Change…
      </button>
    </div>
    <span class="text-xs text-ink-faint">Organized & enhanced copies are written here.</span>
  </section>

  <section class="flex flex-col gap-1.5">
    <span class="eyebrow">Thumbnail size (px)</span>
    <div class="flex items-center gap-2">
      <input type="number" min="100" max="1000" step="50" bind:value={thumbSize} class="input w-28" />
      <button class="btn" onclick={saveThumb}>
        {#if saved}<Icon name="check" size={14} />Saved{:else}Save{/if}
      </button>
    </div>
  </section>

  <section class="rounded-[8px] border border-line bg-surface p-3 text-xs text-ink-dim">
    <p class="mb-1.5 flex items-center gap-1.5 font-medium text-ink">
      <Icon name="download" size={14} class="text-ink-dim" />
      ML models
    </p>
    <p class="leading-relaxed">
      Smart-orient, scan-line removal and enhancement run on-device via ONNX
      Runtime (CPU). Models download on first use into the app data folder.
      GPU acceleration (CUDA / DirectML) is a build-time option.
    </p>
  </section>
</div>

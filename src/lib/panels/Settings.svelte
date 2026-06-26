<script lang="ts">
  import { listModels, logDir, modelsDir, openLogDir } from '../api'
  import { store } from '../store.svelte'
  import type { ModelDownload, ModelStatus } from '../types'
  import Icon from '../ui/Icon.svelte'

  let thumbSize = $state(store.settings?.thumbnail_size ?? 400)
  let saved = $state(false)

  let models = $state<ModelStatus[]>([])
  let dir = $state('')
  let logs = $state('')

  $effect(() => {
    if (store.settings) thumbSize = store.settings.thumbnail_size
  })

  loadModels()

  async function loadModels() {
    ;[models, dir, logs] = await Promise.all([listModels(), modelsDir(), logDir()])
  }

  async function saveThumb() {
    await store.setThumbnailSize(Number(thumbSize))
    saved = true
    setTimeout(() => (saved = false), 1500)
  }

  const missing = $derived(models.filter((m) => !m.downloaded))

  async function download() {
    if (store.downloadingModels || missing.length === 0) return
    await store.downloadModelFiles(null)
    await loadModels()
  }

  function sizeLabel(m: ModelStatus): string {
    if (m.downloaded && m.size_bytes != null) return `${Math.round(m.size_bytes / 1e6)} MB`
    return `~${m.approx_mb} MB`
  }

  function pct(p: ModelDownload): number {
    if (!p.total) return 0
    return Math.min(100, Math.round((p.downloaded / p.total) * 100))
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

  <section class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <span class="eyebrow">ML models</span>
      <button class="btn-sm" onclick={download} disabled={store.downloadingModels || missing.length === 0}>
        {#if store.downloadingModels}
          <Icon name="refresh" size={13} class="animate-spin" />
          Downloading…
        {:else if missing.length === 0}
          <Icon name="check" size={13} />
          All downloaded
        {:else}
          <Icon name="download" size={13} />
          Download {missing.length} missing
        {/if}
      </button>
    </div>

    <div class="flex flex-col divide-y divide-line overflow-hidden rounded-[8px] border border-line bg-surface">
      {#each models as m (m.key)}
        {@const p = store.modelProgress[m.key]}
        {@const active = store.downloadingModels && p && !p.done && !p.error}
        <div class="flex flex-col gap-1.5 p-3">
          <div class="flex items-center justify-between gap-2">
            <span class="font-medium text-ink">{m.label}</span>
            {#if p?.error}
              <span class="flex items-center gap-1 text-xs text-danger" title={p.error}>
                <Icon name="alertTriangle" size={13} />
                Failed
              </span>
            {:else if m.downloaded || p?.done}
              <span class="flex items-center gap-1 text-xs text-brand">
                <Icon name="checkCircle" size={13} />
                {sizeLabel(m)}
              </span>
            {:else if active}
              <span class="font-mono text-xs text-ink-dim">
                {p.total ? `${pct(p)}%` : `${Math.round(p.downloaded / 1e6)} MB`}
              </span>
            {:else}
              <span class="text-xs text-ink-faint">{sizeLabel(m)} · not downloaded</span>
            {/if}
          </div>

          {#if active}
            <div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-2">
              <div
                class="h-full rounded-full bg-brand transition-[width] duration-200"
                class:animate-pulse={!p.total}
                style="width:{p.total ? pct(p) : 100}%"
              ></div>
            </div>
          {:else if p?.error}
            <span class="truncate text-xs text-danger" title={p.error}>{p.error}</span>
          {/if}
        </div>
      {/each}
    </div>

    {#if dir}
      <code class="truncate rounded-[3px] border border-line bg-surface px-2 py-1 font-mono text-[11px] text-ink-faint" title={dir}>
        {dir}
      </code>
    {/if}

    <p class="flex items-start gap-1.5 text-xs leading-relaxed text-ink-faint">
      <Icon name="info" size={13} class="mt-0.5 shrink-0" />
      Models run on-device via ONNX Runtime (CPU) and download on first use, or
      pre-fetch them here. GPU acceleration (CUDA / DirectML) is a build-time option.
    </p>
  </section>

  <section class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <span class="eyebrow">Diagnostics</span>
      <button class="btn-sm" onclick={() => openLogDir()}>
        <Icon name="folderOpen" size={13} />
        Open logs folder
      </button>
    </div>
    {#if logs}
      <code class="truncate rounded-[3px] border border-line bg-surface px-2 py-1 font-mono text-[11px] text-ink-faint" title={logs}>
        {logs}
      </code>
    {/if}
    <p class="flex items-start gap-1.5 text-xs leading-relaxed text-ink-faint">
      <Icon name="info" size={13} class="mt-0.5 shrink-0" />
      Activity is written to <code class="font-mono">photo-archiver.log</code>. If something
      goes wrong, open this folder and send us that file.
    </p>
  </section>
</div>

<script lang="ts">
  import { listModels, logDir, modelsDir, openLogDir } from '../api'
  import { store } from '../store.svelte'
  import type { ModelDownload, ModelStatus } from '../types'
  import Icon from '../ui/Icon.svelte'
  import Spinner from '../ui/Spinner.svelte'
  import Progress from '../ui/Progress.svelte'
  import Modal from './Modal.svelte'

  let thumbSize = $state(store.settings?.thumbnail_size ?? 400)
  let models = $state<ModelStatus[]>([])
  let dir = $state('')
  let logs = $state('')

  load()

  async function load() {
    try {
      ;[models, dir, logs] = await Promise.all([listModels(), modelsDir(), logDir()])
    } catch (e) {
      store.fail(e)
    }
  }

  const missing = $derived(models.filter((m) => !m.downloaded))
  const thumbDirty = $derived(Number(thumbSize) !== store.settings?.thumbnail_size)

  async function saveThumb() {
    try {
      await store.setThumbnailSize(Number(thumbSize))
      store.toast('Thumbnail size saved — thumbnails will regenerate', 'success')
    } catch (e) {
      store.fail(e)
    }
  }

  async function download() {
    if (store.downloadingModels || missing.length === 0) return
    await store.downloadModelFiles(null)
    await load()
  }

  function size(m: ModelStatus) {
    if (m.downloaded && m.size_bytes != null) return `${Math.round(m.size_bytes / 1e6)} MB`
    return `~${m.approx_mb} MB`
  }

  function pct(p: ModelDownload) {
    return p.total ? Math.min(100, Math.round((p.downloaded / p.total) * 100)) : null
  }
</script>

{#snippet row(label: string, hint: string, control: import('svelte').Snippet)}
  <div class="grid grid-cols-[180px_1fr] items-start gap-4 py-3">
    <div>
      <div class="text-[13px] text-ink">{label}</div>
      <div class="mt-0.5 text-[11px] leading-snug text-ink-faint">{hint}</div>
    </div>
    <div class="min-w-0">{@render control()}</div>
  </div>
{/snippet}

{#snippet sourceCtl()}
  <div class="flex items-center gap-2">
    <code class="input flex min-w-0 flex-1 items-center truncate font-mono text-xs text-ink-dim" title={store.settings?.source_dir ?? ''}>
      {store.settings?.source_dir ?? 'Not set'}
    </code>
    <button class="btn-sm" onclick={() => store.pickFolder()}>Change…</button>
  </div>
{/snippet}

{#snippet outputCtl()}
  <div class="flex items-center gap-2">
    <code class="input flex min-w-0 flex-1 items-center truncate font-mono text-xs text-ink-dim" title={store.settings?.output_dir ?? ''}>
      {store.settings?.output_dir ?? 'Default (app data folder)'}
    </code>
    <button class="btn-sm" onclick={() => store.pickOutput()}>Change…</button>
  </div>
{/snippet}

{#snippet thumbCtl()}
  <div class="flex items-center gap-2">
    <input type="number" min="100" max="1000" step="50" bind:value={thumbSize} class="input w-24 tabular-nums" />
    <span class="text-xs text-ink-faint">px</span>
    {#if thumbDirty}<button class="btn-sm" onclick={saveThumb}>Save</button>{/if}
  </div>
{/snippet}

<Modal title="Settings" width={640} onclose={() => (store.settingsOpen = false)}>
  <div class="px-5 py-2">
    <h3 class="eyebrow mt-3">Folders</h3>
    <div class="divide-y divide-line">
      {@render row('Scans', 'Where your scanned photos live. Never modified.', sourceCtl)}
      {@render row('Output', 'Organized and restored copies are written here.', outputCtl)}
    </div>

    <h3 class="eyebrow mt-5">Library</h3>
    <div class="divide-y divide-line">
      {@render row('Thumbnail quality', 'Larger looks sharper in big grids but takes more disk.', thumbCtl)}
    </div>

    <div class="mt-5 flex items-center justify-between">
      <h3 class="eyebrow">AI models</h3>
      <button class="btn-sm" onclick={download} disabled={store.downloadingModels || missing.length === 0}>
        {#if store.downloadingModels}
          <Spinner size={11} />Downloading…
        {:else if missing.length === 0}
          <Icon name="check" size={13} />All downloaded
        {:else}
          <Icon name="download" size={13} />Download {missing.length} missing
        {/if}
      </button>
    </div>
    <div class="mt-2 divide-y divide-line overflow-hidden rounded-[8px] border border-line bg-base">
      {#each models as m (m.key)}
        {@const p = store.modelProgress[m.key]}
        <div class="px-3 py-2.5">
          <div class="flex items-center justify-between gap-2 text-xs">
            <span class="text-ink">{m.label}</span>
            {#if p?.error}
              <span class="flex items-center gap-1 text-danger" title={p.error}><Icon name="alertTriangle" size={12} />Failed</span>
            {:else if m.downloaded || p?.done}
              <span class="flex items-center gap-1 text-success"><Icon name="checkCircle" size={12} />{size(m)}</span>
            {:else if p}
              <span class="text-ink-dim tabular-nums">{pct(p) != null ? `${pct(p)}%` : `${Math.round(p.downloaded / 1e6)} MB`}</span>
            {:else}
              <span class="text-ink-faint">{size(m)}</span>
            {/if}
          </div>
          {#if p && !p.done && !p.error && store.downloadingModels}
            <div class="mt-1.5"><Progress value={pct(p)} height={3} /></div>
          {:else if p?.error}
            <p class="mt-1 truncate text-[11px] text-danger" title={p.error}>{p.error}</p>
          {/if}
        </div>
      {/each}
    </div>
    <p class="mt-2 text-[11px] leading-relaxed text-ink-faint">
      Models run on this computer (ONNX Runtime, CPU) and download on first use.
      {#if dir}Stored in <code class="font-mono select-text">{dir}</code>.{/if}
    </p>

    <h3 class="eyebrow mt-5">Diagnostics</h3>
    <div class="mt-2 mb-4 flex items-center gap-3 rounded-[8px] border border-line bg-base px-3 py-2.5">
      <div class="min-w-0 flex-1">
        <p class="text-xs text-ink">Activity log</p>
        <p class="truncate font-mono text-[11px] text-ink-faint select-text" title={logs}>{logs}</p>
      </div>
      <button class="btn-sm" onclick={() => openLogDir()}><Icon name="folderOpen" size={13} />Open</button>
    </div>
  </div>
</Modal>

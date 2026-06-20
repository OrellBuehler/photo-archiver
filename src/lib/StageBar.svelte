<script lang="ts">
  import { store } from './store.svelte'
  import Icon from './ui/Icon.svelte'
  import Badge from './ui/Badge.svelte'

  let { activePanelId = 'library' }: { activePanelId?: string } = $props()

  const STAGES = [
    { key: 'library', panel: 'library', label: 'Library', sub: 'Pick & triage' },
    { key: 'processing', panel: 'processing', label: 'Restore', sub: 'Choose & watch' },
    { key: 'viewer', panel: 'viewer', label: 'Review', sub: 'Before / after' },
  ]

  // Which stage each dockview panel belongs to.
  const STAGE_OF: Record<string, string> = {
    library: 'library',
    filters: 'library',
    duplicates: 'library',
    processing: 'processing',
    tasks: 'processing',
    viewer: 'viewer',
  }
  const activeStage = $derived(STAGE_OF[activePanelId] ?? null)

  function folderName(p: string) {
    return p.replace(/[/\\]+$/, '').split(/[/\\]/).pop() || p
  }

  const hasLib = $derived(!!store.settings?.source_dir && store.total > 0)
</script>

<div class="flex h-10 shrink-0 items-center gap-0.5 border-b border-line bg-base px-2">
  {#each STAGES as s, i (s.key)}
    {#if i > 0}
      <Icon name="chevronRight" size={14} class="mx-0.5 text-ink-faint" />
    {/if}
    <button
      type="button"
      onclick={() => store.activatePanel(s.panel)}
      class={`flex flex-col items-start gap-px rounded-[5px] border px-3 py-1 transition-colors duration-[110ms] ${
        activeStage === s.key
          ? 'border-brand/40 bg-brand-soft'
          : 'border-transparent hover:bg-surface'
      }`}
    >
      <span
        class="text-xs font-semibold"
        style={activeStage === s.key ? 'color: var(--color-brand);' : 'color: var(--color-ink);'}
      >{s.label}</span>
      <span class="text-[10px] text-ink-faint">{s.sub}</span>
    </button>
  {/each}

  {#if hasLib}
    <div class="ml-auto flex items-center gap-3.5 pr-1.5 text-xs text-ink-dim">
      {#if store.settings?.source_dir}
        <span class="flex items-center gap-1.5" title={store.settings.source_dir}>
          <Icon name="folderOpen" size={14} />
          <span class="max-w-[160px] truncate">{folderName(store.settings.source_dir)}</span>
        </span>
      {/if}
      <span class="flex items-center gap-2.5">
        <span><b class="font-medium text-ink">{store.total}</b> photos</span>
        <Badge status="enhanced" label={`${store.statusCount('enhanced')} done`} />
        <Badge status="new" label={`${store.statusCount('new')} to do`} />
      </span>
    </div>
  {/if}
</div>

<script lang="ts">
  import Icon from './Icon.svelte'

  let {
    icon,
    label,
    hint,
    model = false,
    selected = false,
    state = 'idle',
    onToggle,
  }: {
    icon: string
    label: string
    hint: string
    model?: boolean
    selected?: boolean
    state?: 'idle' | 'running' | 'done'
    onToggle?: () => void
  } = $props()
</script>

<button
  type="button"
  onclick={onToggle}
  class={`flex w-full items-start gap-3 rounded-[5px] border px-3 py-[11px] text-left transition-colors duration-[110ms] ${
    selected
      ? 'border-brand/50 bg-brand-soft'
      : 'border-line bg-surface hover:border-[var(--border-strong)] hover:bg-surface-2'
  }`}
>
  <!-- checkbox -->
  <span
    class="mt-0.5 grid h-4 w-4 shrink-0 place-items-center rounded-[3px] border-[1.5px] transition-colors duration-[110ms]"
    class:border-brand={selected}
    class:bg-brand={selected}
    style={selected ? 'color: var(--color-on-brand);' : 'border-color: var(--border-strong);'}
  >
    {#if selected}
      <svg width="10" height="10" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M2.5 6.5l2.5 2.5 4.5-5" /></svg>
    {/if}
  </span>

  <!-- icon tile -->
  <span
    class="grid h-[30px] w-[30px] shrink-0 place-items-center rounded-[5px]"
    style={selected
      ? 'background: color-mix(in srgb, var(--color-brand) 18%, transparent); color: var(--color-brand);'
      : 'background: var(--color-surface-2); color: var(--color-ink-dim);'}
  >
    <Icon name={icon} size={17} />
  </span>

  <!-- text -->
  <span class="min-w-0 flex-1">
    <span class="flex items-center gap-1.5">
      <span class="font-medium text-ink">{label}</span>
      {#if model}
        <span class="rounded-full border border-line px-1.5 py-px text-[10px] text-ink-faint">
          model
        </span>
      {/if}
    </span>
    <span class="mt-0.5 block text-xs leading-snug text-ink-dim">{hint}</span>
  </span>

  <!-- live state -->
  {#if state !== 'idle'}
    <span class="flex shrink-0 items-center self-center">
      {#if state === 'running'}
        <span
          class="h-[13px] w-[13px] rounded-full border-2"
          style="border-color: color-mix(in srgb, var(--color-warning) 35%, transparent); border-top-color: var(--color-warning); animation: pa-spin 0.7s linear infinite;"
        ></span>
      {:else}
        <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="var(--color-success)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 8.5l3.5 3.5L13 4.5" /></svg>
      {/if}
    </span>
  {/if}
</button>

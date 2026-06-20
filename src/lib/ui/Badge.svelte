<script lang="ts" module>
  // The scannable status language: new / organized / enhanced / processing /
  // running / completed / failed / cancelled.
  const STATUS_MAP: Record<string, { fg: string; bg: string }> = {
    new: { fg: 'var(--color-ink-dim)', bg: 'var(--color-surface-2)' },
    organized: { fg: 'var(--color-info)', bg: 'var(--color-info-soft)' },
    enhanced: { fg: 'var(--color-success)', bg: 'var(--color-success-soft)' },
    processing: { fg: 'var(--color-warning)', bg: 'var(--color-warning-soft)' },
    running: { fg: 'var(--color-warning)', bg: 'var(--color-warning-soft)' },
    completed: { fg: 'var(--color-success)', bg: 'var(--color-success-soft)' },
    failed: { fg: 'var(--color-danger)', bg: 'var(--color-danger-soft)' },
    cancelled: { fg: 'var(--color-warning)', bg: 'var(--color-warning-soft)' },
  }
</script>

<script lang="ts">
  let {
    status,
    dot = true,
    label,
  }: { status: string; dot?: boolean; label?: string } = $props()

  const preset = $derived(STATUS_MAP[status] ?? { fg: 'var(--color-ink-dim)', bg: 'var(--color-surface-2)' })
</script>

<span
  class="inline-flex items-center gap-1.5 whitespace-nowrap rounded-full px-2 py-0.5 text-[11px] font-semibold leading-tight"
  style="color: {preset.fg}; background: {preset.bg};"
>
  {#if dot}
    <span class="h-1.5 w-1.5 shrink-0 rounded-full" style="background: {preset.fg};"></span>
  {/if}
  {label ?? status}
</span>

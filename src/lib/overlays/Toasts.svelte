<script lang="ts">
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'

  const ICON = { info: 'info', success: 'checkCircle', error: 'alertTriangle' } as const
  const COLOR = { info: 'text-info', success: 'text-success', error: 'text-danger' } as const
</script>

<div class="pointer-events-none fixed right-4 bottom-10 z-[70] flex w-[340px] flex-col items-end gap-2">
  {#each store.toasts as t (t.id)}
    <div
      class="pop pointer-events-auto flex w-full items-start gap-2.5 rounded-[8px] border border-line bg-surface-2 px-3 py-2.5 shadow-[var(--shadow-md)]"
      role="status"
    >
      <Icon name={ICON[t.kind]} size={16} class="mt-px {COLOR[t.kind]}" />
      <p class="min-w-0 flex-1 text-[13px] break-words text-ink select-text">{t.text}</p>
      <button class="text-ink-faint hover:text-ink" aria-label="Dismiss" onclick={() => store.dismissToast(t.id)}>
        <Icon name="x" size={14} />
      </button>
    </div>
  {/each}
</div>

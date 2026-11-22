<script lang="ts">
  import type { Snippet } from 'svelte'
  import Icon from '../ui/Icon.svelte'

  let {
    title,
    width = 480,
    onclose,
    children,
    footer,
  }: { title: string; width?: number; onclose: () => void; children: Snippet; footer?: Snippet } = $props()

  $effect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        e.stopPropagation()
        onclose()
      }
    }
    window.addEventListener('keydown', onKey, true)
    return () => window.removeEventListener('keydown', onKey, true)
  })
</script>

<div class="fixed inset-0 z-50 grid place-items-center p-6">
  <button class="scrim absolute inset-0 cursor-default" aria-label="Close" onclick={onclose}></button>
  <div
    role="dialog"
    aria-modal="true"
    aria-label={title}
    class="pop relative flex max-h-full w-full flex-col overflow-hidden rounded-[12px] border border-line bg-surface shadow-[var(--shadow-lg)]"
    style="max-width: {width}px;"
  >
    <header class="flex h-12 shrink-0 items-center border-b border-line pr-2 pl-4">
      <h2 class="text-[14px] font-semibold text-ink">{title}</h2>
      <button class="icon-btn ml-auto" aria-label="Close" onclick={onclose}><Icon name="x" size={16} /></button>
    </header>
    <div class="min-h-0 flex-1 overflow-y-auto">{@render children()}</div>
    {#if footer}
      <footer class="flex shrink-0 justify-end gap-2 border-t border-line bg-base/50 px-4 py-3">{@render footer()}</footer>
    {/if}
  </div>
</div>

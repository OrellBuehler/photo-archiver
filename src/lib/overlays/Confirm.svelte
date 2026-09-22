<script lang="ts">
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'

  const req = $derived(store.confirmRequest)
  let confirmBtn = $state<HTMLButtonElement>()

  $effect(() => {
    if (!req) return
    confirmBtn?.focus()
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        e.stopPropagation()
        req.resolve(false)
      }
    }
    window.addEventListener('keydown', onKey, true)
    return () => window.removeEventListener('keydown', onKey, true)
  })
</script>

{#if req}
  <div class="fixed inset-0 z-[60] grid place-items-center p-6">
    <button class="scrim absolute inset-0 cursor-default" aria-label="Cancel" onclick={() => req.resolve(false)}></button>
    <div role="alertdialog" aria-modal="true" class="pop relative w-full max-w-[400px] rounded-[12px] border border-line bg-surface p-5 shadow-[var(--shadow-lg)]">
      <div class="flex gap-3">
        <span class="grid h-9 w-9 shrink-0 place-items-center rounded-full {req.danger ? 'bg-danger-soft text-danger' : 'bg-brand-soft text-brand'}">
          <Icon name={req.danger ? 'trash' : 'info'} size={17} />
        </span>
        <div>
          <h2 class="text-[15px] font-semibold text-ink">{req.title}</h2>
          <p class="mt-1 text-[13px] leading-relaxed text-ink-dim">{req.message}</p>
        </div>
      </div>
      <div class="mt-5 flex justify-end gap-2">
        <button class="btn-sm h-8 px-3" onclick={() => req.resolve(false)}>Cancel</button>
        <button bind:this={confirmBtn} class={req.danger ? 'btn-danger-solid' : 'btn'} onclick={() => req.resolve(true)}>
          {req.confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

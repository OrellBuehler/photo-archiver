<script lang="ts">
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'

  const menu = $derived(store.menu)
  let el = $state<HTMLDivElement>()
  let pos = $state({ x: 0, y: 0 })

  // Open at the pointer, nudged back inside the window.
  $effect(() => {
    if (!menu || !el) return
    const r = el.getBoundingClientRect()
    pos = {
      x: Math.max(4, Math.min(menu.x, window.innerWidth - r.width - 4)),
      y: Math.max(4, Math.min(menu.y, window.innerHeight - r.height - 4)),
    }
    el.querySelector<HTMLElement>('[role="menuitem"]')?.focus()
  })

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault()
      e.stopPropagation()
      store.menu = null
    } else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault()
      e.stopPropagation()
      const items = [...(el?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])]
      const i = items.indexOf(document.activeElement as HTMLElement)
      items[(i + (e.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length]?.focus()
    }
  }
</script>

{#if menu}
  <div
    class="fixed inset-0 z-[65]"
    role="presentation"
    onmousedown={() => (store.menu = null)}
    oncontextmenu={(e) => {
      e.preventDefault()
      store.menu = null
    }}
    onwheel={() => (store.menu = null)}
  ></div>
  <div
    bind:this={el}
    role="menu"
    tabindex="-1"
    class="pop fixed z-[66] min-w-[210px] rounded-[8px] border border-[var(--border-strong)] bg-surface-2 p-1 shadow-[var(--shadow-lg)] outline-none"
    style="left: {pos.x}px; top: {pos.y}px;"
    onkeydown={onKey}
  >
    {#each menu.items as item (item.label)}
      {#if item.separator}<div class="mx-1 my-1 h-px bg-line"></div>{/if}
      <button
        role="menuitem"
        class="flex h-7 w-full items-center gap-2.5 rounded-[5px] px-2 text-left text-[13px] outline-none {item.danger
          ? 'text-danger hover:bg-danger-soft focus:bg-danger-soft'
          : 'text-ink hover:bg-bar focus:bg-bar'}"
        onclick={() => {
          store.menu = null
          item.run()
        }}
      >
        {#if item.icon}<Icon name={item.icon} size={14} class={item.danger ? '' : 'text-ink-dim'} />{/if}
        <span class="min-w-0 flex-1 truncate">{item.label}</span>
        {#if item.hint}<span class="text-[11px] text-ink-faint">{item.hint}</span>{/if}
      </button>
    {/each}
  </div>
{/if}

<script lang="ts">
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'
  import Spinner from '../ui/Spinner.svelte'

  const STEPS = [
    { icon: 'folderOpen', title: 'Point at your scans', text: 'Any folder or mounted drive. Originals are never modified.' },
    { icon: 'wand', title: 'Restore', text: 'Crop, straighten, revive faded colour, remove dust and scan lines.' },
    { icon: 'folderTree', title: 'File away', text: 'Date your photos and copy them into tidy year / month folders.' },
  ]
</script>

<div class="flex h-full items-center justify-center overflow-y-auto p-8">
  <div class="flex w-full max-w-[640px] flex-col items-center text-center">
    <h1 class="font-serif text-[34px] leading-tight text-ink">Bring old photos back to life</h1>
    <p class="mt-2 max-w-md text-[14px] text-ink-dim">
      Restore and organize photos you've scanned from albums, shoeboxes and slides.
    </p>

    <button
      class="group mt-8 flex w-full flex-col items-center gap-3 rounded-[14px] border-2 border-dashed border-line bg-surface px-8 py-10 transition-colors hover:border-brand/60 hover:bg-brand-soft/30"
      onclick={() => store.pickFolder()}
      disabled={store.scanning}
    >
      <span class="grid h-14 w-14 place-items-center rounded-[12px] bg-brand-soft text-brand transition-transform group-hover:scale-105">
        {#if store.scanning}<Spinner size={24} />{:else}<Icon name="folderOpen" size={28} stroke={1.6} />{/if}
      </span>
      <span class="text-[15px] font-medium text-ink">{store.scanning ? 'Scanning folder…' : 'Choose a folder of scans'}</span>
      <span class="text-xs text-ink-faint">or drop a folder anywhere on this window</span>
    </button>

    <div class="mt-10 grid w-full grid-cols-3 gap-4 text-left">
      {#each STEPS as s, i (s.title)}
        <div class="flex flex-col gap-1.5">
          <div class="flex items-center gap-2">
            <span class="grid h-5 w-5 place-items-center rounded-full bg-surface-2 text-[10px] font-bold text-ink-dim">{i + 1}</span>
            <span class="text-[13px] font-medium text-ink">{s.title}</span>
          </div>
          <p class="text-xs leading-relaxed text-ink-dim">{s.text}</p>
        </div>
      {/each}
    </div>
  </div>
</div>

<script lang="ts">
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'
  import Spinner from '../ui/Spinner.svelte'

  const STEPS = [
    { title: 'Point at your scans', text: 'Any folder or mounted drive. The scans themselves are never changed.' },
    { title: 'Date and restore', text: 'Crop, straighten, revive faded colour, clear dust and scan lines.' },
    { title: 'Get a tidy archive', text: 'Clean copies land in year and event folders, ready for your photo library.' },
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
      <span class="text-[15px] font-medium text-ink">{store.scanning ? 'Looking for photos…' : 'Choose your folder of scans'}</span>
      <span class="text-xs text-ink-faint">or drop a folder anywhere on this window</span>
    </button>

    <div class="mt-3 flex w-full items-center gap-3 rounded-[10px] border border-line bg-surface px-4 py-3 text-left">
      <Icon name="archive" size={18} class="text-ink-faint" />
      <div class="min-w-0 flex-1">
        <p class="text-xs text-ink-dim">Restored copies are saved to</p>
        <p class="truncate font-mono text-[11px] text-ink select-text" title={store.settings?.output_dir ?? ''}>{store.settings?.output_dir ?? '—'}</p>
      </div>
      <button class="btn-sm" onclick={() => store.pickOutput()}>Change…</button>
    </div>

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

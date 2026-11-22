<script lang="ts">
  import { store } from '../store.svelte'
  import Icon from '../ui/Icon.svelte'
  import Spinner from '../ui/Spinner.svelte'
  import Single from './Single.svelte'
  import Batch from './Batch.svelte'
  import Overview from './Overview.svelte'
  import Restore from './Restore.svelte'

  const multi = $derived(store.selected.size > 1)
  const single = $derived(!multi ? store.focused : null)
</script>

<aside class="flex h-full w-[320px] shrink-0 flex-col border-l border-line bg-surface">
  <div class="flex h-11 shrink-0 items-center border-b border-line px-3">
    <div class="seg w-full">
      <button class="flex-1 justify-center" aria-pressed={store.inspectorTab === 'info'} onclick={() => (store.inspectorTab = 'info')}>
        <Icon name="info" size={13} />Info
      </button>
      <button class="flex-1 justify-center" aria-pressed={store.inspectorTab === 'restore'} onclick={() => (store.inspectorTab = 'restore')}>
        {#if store.busy}<Spinner size={11} />{:else}<Icon name="wand" size={13} />{/if}Restore
      </button>
    </div>
  </div>

  <div class="min-h-0 flex-1">
    {#if store.inspectorTab === 'restore'}
      <Restore />
    {:else if multi}
      <Batch />
    {:else if single}
      {#key single.id}
        <Single image={single} />
      {/key}
    {:else}
      <Overview />
    {/if}
  </div>
</aside>

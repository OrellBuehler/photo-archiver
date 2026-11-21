<script lang="ts">
  import { store } from '../store.svelte'
  import Modal from './Modal.svelte'

  const GROUPS = [
    {
      title: 'Browse',
      items: [
        [['←', '→', '↑', '↓'], 'Move between photos'],
        [['Enter'], 'Open large view'],
        [['Esc'], 'Back to grid / clear selection'],
        [['Right-click'], 'Everything you can do with a photo'],
        [['/'], 'Search'],
        [['Ctrl', 'scroll'], 'Thumbnail size'],
      ],
    },
    {
      title: 'Select',
      items: [
        [['Space'], 'Toggle photo'],
        [['Shift', '←→↑↓'], 'Extend selection'],
        [['Ctrl', 'A'], 'Select everything in view'],
      ],
    },
    {
      title: 'Date',
      items: [
        [['D'], 'Type the date'],
        [['Enter'], 'Save and go to the next photo'],
        [['.'], 'Same date as the last photo'],
      ],
    },
    {
      title: 'Edit',
      items: [
        [['['], 'Rotate left'],
        [[']'], 'Rotate right'],
        [['Del'], 'Remove from library'],
        [['R'], 'Restore panel'],
        [['I'], 'Info panel'],
      ],
    },
    {
      title: 'Large view',
      items: [
        [['O'], 'Hold to see the untouched scan'],
        [['C'], 'Slide between scan and result'],
        [['+', '−', '0'], 'Zoom in / out / fit'],
        [['Ctrl', 'Z'], 'Undo last step'],
        [['Ctrl', 'Shift', 'Z'], 'Redo'],
      ],
    },
    {
      title: 'Window',
      items: [
        [['Ctrl', 'B'], 'Toggle sidebar'],
        [['Ctrl', 'I'], 'Toggle inspector'],
        [['Ctrl', ','], 'Settings'],
        [['?'], 'This list'],
      ],
    },
  ] as const
</script>

<Modal title="Keyboard shortcuts" width={620} onclose={() => (store.shortcutsOpen = false)}>
  <div class="grid grid-cols-2 gap-x-8 gap-y-5 p-5">
    {#each GROUPS as g (g.title)}
      <section>
        <h3 class="eyebrow mb-2">{g.title}</h3>
        <dl class="flex flex-col gap-1.5">
          {#each g.items as [keys, label] (label)}
            <div class="flex items-center justify-between gap-3 text-xs">
              <dt class="text-ink-dim">{label}</dt>
              <dd class="flex gap-1">
                {#each keys as k (k)}<span class="kbd">{k}</span>{/each}
              </dd>
            </div>
          {/each}
        </dl>
      </section>
    {/each}
  </div>
</Modal>

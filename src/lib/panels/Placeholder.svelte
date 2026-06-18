<script lang="ts">
  let { title, hint }: { title: string; hint: string } = $props()
</script>

<div class="flex h-full flex-col bg-base">
  <div class="px-3 py-2 text-xs font-semibold tracking-wide text-ink-dim uppercase">
    {title}
  </div>
  <div class="flex flex-1 items-center justify-center p-6 text-center text-sm text-ink-dim">
    {hint}
  </div>
</div>

<script lang="ts">
  let { value = null, height = 4 }: { value?: number | null; height?: number } = $props()
</script>

<div class="relative w-full overflow-hidden rounded-full bg-surface-2" style="height: {height}px;">
  {#if value == null}
    <div
      class="absolute inset-y-0 w-2/5 rounded-full bg-brand"
      style="animation: pa-indeterminate 1.1s ease-in-out infinite;"
    ></div>
  {:else}
    <div class="h-full rounded-full bg-brand transition-[width] duration-200" style="width: {Math.min(100, Math.max(0, value))}%"></div>
  {/if}
</div>

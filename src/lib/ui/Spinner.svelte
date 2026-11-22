<script lang="ts">
  let { size = 14, color = 'var(--color-brand)' }: { size?: number; color?: string } = $props()
</script>

<span
  class="inline-block shrink-0 rounded-full border-2"
  style="width: {size}px; height: {size}px; border-color: color-mix(in srgb, {color} 28%, transparent); border-top-color: {color}; animation: pa-spin 0.7s linear infinite;"
></span>

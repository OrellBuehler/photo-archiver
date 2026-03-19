<script lang="ts">
  let { beforeSrc, afterSrc, beforeLabel = 'Before', afterLabel = 'After' }: {
    beforeSrc: string;
    afterSrc: string;
    beforeLabel?: string;
    afterLabel?: string;
  } = $props();

  let position = $state(50);
  let dragging = $state(false);
  let container: HTMLDivElement;

  function updatePosition(clientX: number) {
    const rect = container.getBoundingClientRect();
    const x = clientX - rect.left;
    position = Math.max(0, Math.min(100, (x / rect.width) * 100));
  }

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    updatePosition(e.clientX);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    updatePosition(e.clientX);
  }

  function onPointerUp() {
    dragging = false;
  }
</script>

<div
  bind:this={container}
  class="relative select-none overflow-hidden rounded-lg border cursor-col-resize"
  role="slider"
  aria-valuenow={Math.round(position)}
  aria-valuemin={0}
  aria-valuemax={100}
  tabindex="0"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onkeydown={(e) => {
    if (e.key === 'ArrowLeft') position = Math.max(0, position - 2);
    if (e.key === 'ArrowRight') position = Math.min(100, position + 2);
  }}
>
  <!-- After image (full width, behind) -->
  <img src={afterSrc} alt={afterLabel} class="block w-full h-auto" />

  <!-- Before image (clipped) -->
  <div
    class="absolute inset-0 overflow-hidden"
    style="width: {position}%"
  >
    <img
      src={beforeSrc}
      alt={beforeLabel}
      class="block h-full object-cover"
      style="width: {container?.clientWidth || 0}px"
    />
  </div>

  <!-- Divider line -->
  <div
    class="absolute top-0 bottom-0 w-0.5 bg-white shadow-lg"
    style="left: {position}%"
  >
    <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-8 h-8 rounded-full bg-white shadow-md flex items-center justify-center">
      <svg class="w-4 h-4 text-gray-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M8 5l-5 7 5 7M16 5l5 7-5 7" />
      </svg>
    </div>
  </div>

  <!-- Labels -->
  <div class="absolute top-3 left-3 px-2 py-1 rounded bg-black/50 text-white text-xs">{beforeLabel}</div>
  <div class="absolute top-3 right-3 px-2 py-1 rounded bg-black/50 text-white text-xs">{afterLabel}</div>
</div>

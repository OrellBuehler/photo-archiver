<script lang="ts" module>
  // Lucide-style line icons (1.75 stroke). The source app ships no icon
  // library; these mirror Lucide. "c<cx> <cy> <r>" segments render as circles.
  export const ICON_PATHS: Record<string, string> = {
    folder:
      'M4 20h16a1 1 0 0 0 1-1V8a1 1 0 0 0-1-1h-7.9a1 1 0 0 1-.8-.4L9.9 4.4a1 1 0 0 0-.8-.4H4a1 1 0 0 0-1 1v14a1 1 0 0 0 1 1z',
    folderOpen:
      'M4 19l2.5-7a1 1 0 0 1 1-.7H21a1 1 0 0 1 1 1.2L20 19a1 1 0 0 1-1 .7H4zM4 19V5a1 1 0 0 1 1-1h3.9a1 1 0 0 1 .8.4L11 6.5a1 1 0 0 0 .8.4H18a1 1 0 0 1 1 1v2',
    folderTree:
      'M3 4h5l1.5 2H15a1 1 0 0 1 1 1v2H3zM3 9h13v4a1 1 0 0 1-1 1H3zM18 6h3M18 13h3M16 6h2v7h-2',
    refresh: 'M3 12a9 9 0 0 1 15-6.7L21 8M21 3v5h-5M21 12a9 9 0 0 1-15 6.7L3 16m0 5v-5h5',
    palette:
      'M12 2a10 10 0 0 0 0 20 2.5 2.5 0 0 0 2-4 2.5 2.5 0 0 1 2-4h2a4 4 0 0 0 4-4 10 10 0 0 0-10-8z|c8.5 7.5 .6|c13.5 6.5 .6|c17.5 11.5 .6',
    crop: 'M6 2v14a2 2 0 0 0 2 2h14M2 6h14a2 2 0 0 1 2 2v14',
    scanLine:
      'M3 7V5a2 2 0 0 1 2-2h2M17 3h2a2 2 0 0 1 2 2v2M21 17v2a2 2 0 0 1-2 2h-2M7 21H5a2 2 0 0 1-2-2v-2M7 12h10',
    sparkles:
      'M12 3l1.6 4.4L18 9l-4.4 1.6L12 15l-1.6-4.4L6 9l4.4-1.6zM5 16l.8 2.2L8 19l-2.2.8L5 22l-.8-2.2L2 19l2.2-.8zM19 14l.6 1.6L21 16l-1.4.6L19 18l-.6-1.4L17 16l1.4-.6z',
    wand: 'M15 4V2M15 16v-2M8 9h2M20 9h2M17.8 11.8 19 13M17.8 6.2 19 5M3 21l9-9M12.2 6.2 11 5',
    rotateSquare: 'M3 12a9 9 0 1 0 9-9M3 5v4h4M9 12l3-3 3 3-3 3z',
    rotateCcw: 'M3 12a9 9 0 1 0 3-6.7M3 4v4h4',
    rotateCw: 'M21 12a9 9 0 1 1-3-6.7M21 4v4h-4',
    columns:
      'M12 3v18M5 4h14a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z',
    copy: 'M9 9h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V10a1 1 0 0 1 1-1zM5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1',
    listChecks: 'M3 5l2 2 3-3M3 13l2 2 3-3M11 6h10M11 14h10M11 20h7',
    settings:
      'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM19.4 13a7.9 7.9 0 0 0 0-2l2-1.5-2-3.4-2.3 1a8 8 0 0 0-1.7-1l-.3-2.6h-4l-.3 2.6a8 8 0 0 0-1.7 1l-2.3-1-2 3.4 2 1.5a7.9 7.9 0 0 0 0 2l-2 1.5 2 3.4 2.3-1a8 8 0 0 0 1.7 1l.3 2.6h4l.3-2.6a8 8 0 0 0 1.7-1l2.3 1 2-3.4z',
    checkCircle: 'M22 11.1V12a10 10 0 1 1-5.9-9.1M22 4 12 14.5l-3-3',
    alertTriangle:
      'M10.3 4 2.4 18a2 2 0 0 0 1.7 3h15.8a2 2 0 0 0 1.7-3L13.7 4a2 2 0 0 0-3.4 0zM12 9v4M12 17h0',
    info: 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zM12 16v-4M12 8h0',
    download: 'M12 3v12M7 10l5 5 5-5M5 21h14',
    search: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.3-4.3',
    sliders: 'M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6',
    x: 'M18 6 6 18M6 6l12 12',
    chevronLeft: 'M15 18l-6-6 6-6',
    chevronRight: 'M9 18l6-6-6-6',
    image:
      'M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM8.5 10a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zM21 15l-5-5L5 21',
    calendar:
      'M8 2v4M16 2v4M3 9h18M5 5h14a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1z',
    edit: 'M12 20h9M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z',
    check: 'M20 6 9 17l-5-5',
    flag: 'M4 22V4a1 1 0 0 1 1-1h13l-3 5 3 5H5M4 4v18',
    layers: 'M12 2 2 7l10 5 10-5zM2 17l10 5 10-5M2 12l10 5 10-5',
    trash: 'M3 6h18M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2M5 6l1 14a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1l1-14',
    user: 'M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2|c12 7 4',
    undo: 'M9 14 4 9l5-5M4 9h11a4 4 0 0 1 0 8h-4',
    redo: 'M15 14l5-5-5-5M20 9H9a4 4 0 0 0 0 8h4',
    zoomIn: 'M21 21l-4.3-4.3M11 8v6M8 11h6|c11 11 8',
    zoomOut: 'M21 21l-4.3-4.3M8 11h6|c11 11 8',
    maximize: 'M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3',
  }

  type Segment = { kind: 'path'; d: string } | { kind: 'circle'; cx: string; cy: string; r: string }

  export function iconSegments(name: string): Segment[] {
    const raw = ICON_PATHS[name]
    if (!raw) return []
    return raw.split('|').map((p) => {
      if (p[0] === 'c') {
        const [cx, cy, r] = p.slice(1).trim().split(/\s+/)
        return { kind: 'circle', cx, cy, r }
      }
      return { kind: 'path', d: p }
    })
  }
</script>

<script lang="ts">
  let {
    name,
    size = 18,
    stroke = 1.75,
    class: klass = '',
  }: { name: string; size?: number; stroke?: number; class?: string } = $props()

  const segments = $derived(iconSegments(name))
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  class="shrink-0 {klass}"
  aria-hidden="true"
>
  {#each segments as seg}
    {#if seg.kind === 'circle'}
      <circle cx={seg.cx} cy={seg.cy} r={seg.r} fill="currentColor" stroke="none" />
    {:else}
      <path d={seg.d} />
    {/if}
  {/each}
</svg>

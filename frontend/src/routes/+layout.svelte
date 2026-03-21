<script lang="ts">
  import '../app.css';
  import { Toaster } from 'svelte-sonner';
  import { page } from '$app/state';

  let { children } = $props();

  const navItems = [
    { href: '/', label: 'Gallery', match: (p: string) => p === '/' },
    { href: '/processing', label: 'Processing', match: (p: string) => p.startsWith('/processing') },
    { href: '/duplicates', label: 'Duplicates', match: (p: string) => p.startsWith('/duplicates') },
    { href: '/settings', label: 'Settings', match: (p: string) => p.startsWith('/settings') },
  ];
</script>

<Toaster position="bottom-right" richColors />
<div class="h-screen flex flex-col overflow-hidden">
  <header class="shrink-0 border-b bg-background">
    <div class="container mx-auto flex items-center justify-between px-4 py-2.5">
      <a href="/" class="text-lg font-semibold">Photo Archiver</a>
      <nav class="flex gap-1">
        {#each navItems as item}
          {@const active = item.match(page.url.pathname)}
          <a
            href={item.href}
            class="rounded-md px-3 py-1 text-sm transition-colors {active ? 'text-foreground font-medium bg-secondary' : 'text-muted-foreground hover:text-foreground hover:bg-secondary/50'}"
          >{item.label}</a>
        {/each}
      </nav>
    </div>
  </header>
  <main class="flex-1 min-h-0 container mx-auto px-4">
    {@render children()}
  </main>
</div>

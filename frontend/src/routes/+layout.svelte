<script lang="ts">
  import '../app.css';
  import { Toaster } from 'svelte-sonner';

  let { children } = $props();
</script>

<Toaster position="bottom-right" richColors />
<div class="h-screen flex flex-col overflow-hidden">
  <header class="shrink-0 border-b bg-background">
    <div class="container mx-auto flex items-center justify-between px-4 py-3">
      <a href="/" class="text-xl font-bold">Photo Archiver</a>
      <nav class="flex gap-4">
        <a href="/" class="text-sm text-muted-foreground hover:text-foreground transition-colors">Gallery</a>
        <a href="/processing" class="text-sm text-muted-foreground hover:text-foreground transition-colors">Processing</a>
        <a href="/duplicates" class="text-sm text-muted-foreground hover:text-foreground transition-colors">Duplicates</a>
        <a href="/settings" class="text-sm text-muted-foreground hover:text-foreground transition-colors">Settings</a>
      </nav>
    </div>
  </header>
  <main class="flex-1 min-h-0 container mx-auto px-4">
    {@render children()}
  </main>
</div>

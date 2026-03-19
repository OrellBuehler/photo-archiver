<script lang="ts">
  import '../app.css';

  let { children } = $props();
</script>

<div class="min-h-screen flex flex-col">
  <header class="border-b bg-background">
    <div class="container mx-auto flex items-center justify-between px-4 py-3">
      <a href="/" class="text-xl font-bold">Photo Archiver</a>
      <nav class="flex gap-4">
        <a href="/" class="text-sm text-muted-foreground hover:text-foreground transition-colors">Gallery</a>
        <a href="/processing" class="text-sm text-muted-foreground hover:text-foreground transition-colors">Processing</a>
      </nav>
    </div>
  </header>
  <main class="flex-1 container mx-auto px-4 py-6">
    {@render children()}
  </main>
</div>

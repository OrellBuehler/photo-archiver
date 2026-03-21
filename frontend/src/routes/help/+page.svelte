<div class="py-4 space-y-8 max-w-3xl h-full overflow-y-auto">
  <h1 class="text-2xl font-bold">Help</h1>

  <div class="rounded-lg border p-4 space-y-3">
    <h2 class="font-medium">Getting Started</h2>
    <div class="text-sm text-muted-foreground space-y-2">
      <p>Photo Archiver helps you restore and organize scanned photos. Place your scanned images in the source directory, then use the processing pipeline to clean them up and organize them into a structured folder layout.</p>
      <ol class="list-decimal list-inside space-y-1">
        <li>Put scanned photos into the source directory (see Settings)</li>
        <li>The app automatically detects images and extracts metadata from folder names and filenames</li>
        <li>Use the Gallery to browse, filter, and select images</li>
        <li>Select images and choose processing steps to run</li>
        <li>Review results in the image detail view</li>
      </ol>
    </div>
  </div>

  <div class="rounded-lg border p-4 space-y-3">
    <h2 class="font-medium">File Naming</h2>
    <div class="text-sm text-muted-foreground space-y-2">
      <p>The scanner expects files organized in folders by year and optionally month. Filenames should follow the pattern:</p>
      <p class="font-mono bg-muted rounded px-3 py-2 text-xs">{`{year}/{month}/SCAN_0001_Title.jpg`}</p>
      <p>Month folders can use German or English names (e.g. "Januar" or "January"). Year and month are used to set EXIF dates and organize the output.</p>
    </div>
  </div>

  <div class="rounded-lg border p-4 space-y-3">
    <h2 class="font-medium">Processing Steps</h2>
    <div class="text-sm text-muted-foreground space-y-1">
      <p>Steps run in the order you see them. Each step builds on the previous one. The recommended order is:</p>
    </div>
    <div class="space-y-4 mt-2">
      <div>
        <h3 class="text-sm font-medium">1. Organize</h3>
        <p class="text-sm text-muted-foreground">Copies the source image into a structured output folder (<code class="text-xs bg-muted rounded px-1">organized/year/year-month_name.jpg</code>). Sets EXIF date metadata and preserves a reference to the original file. All subsequent steps work on this copy, leaving the original untouched.</p>
      </div>
      <div>
        <h3 class="text-sm font-medium">2. Crop</h3>
        <p class="text-sm text-muted-foreground">Detects and removes white borders from scanning. Useful for images scanned on A4 where the photo is smaller than the page. Skips images that are already tightly cropped.</p>
      </div>
      <div>
        <h3 class="text-sm font-medium">3. Orient</h3>
        <p class="text-sm text-muted-foreground">Uses a neural network (ResNet50) to detect the correct orientation of the photo and rotates it automatically. Evaluates all four rotations and picks the most natural one.</p>
      </div>
      <div>
        <h3 class="text-sm font-medium">4. Deskew</h3>
        <p class="text-sm text-muted-foreground">Detects and corrects slight rotation from misaligned scanning. Uses edge detection to find the dominant angle and straightens the image.</p>
      </div>
      <div>
        <h3 class="text-sm font-medium">5. Restore Color</h3>
        <p class="text-sm text-muted-foreground">Enhances faded colors in old photos. Adjusts brightness using adaptive histogram equalization, removes color casts, and boosts saturation.</p>
      </div>
      <div>
        <h3 class="text-sm font-medium">6. Remove Dust</h3>
        <p class="text-sm text-muted-foreground">Detects and removes small dust particles and scratches from scanned images using adaptive thresholding and inpainting.</p>
      </div>
      <div>
        <h3 class="text-sm font-medium">7. Remove Lines</h3>
        <p class="text-sm text-muted-foreground">Detects and removes thin horizontal lines caused by scanner imperfections (debris on glass, sensor issues). Uses morphological line detection and inpainting.</p>
      </div>
      <div>
        <h3 class="text-sm font-medium">8. Enhance</h3>
        <p class="text-sm text-muted-foreground">AI-powered upscaling using RealESRGAN (4x resolution) with face restoration via GFPGAN. Creates a separate enhanced copy in the <code class="text-xs bg-muted rounded px-1">enhanced/</code> folder. This is the most resource-intensive step and benefits from a CUDA GPU.</p>
      </div>
    </div>
  </div>

  <div class="rounded-lg border p-4 space-y-3">
    <h2 class="font-medium">Gallery &amp; Filtering</h2>
    <div class="text-sm text-muted-foreground space-y-2">
      <p>The gallery shows all imported images with filter pills at the top. You can filter by:</p>
      <ul class="list-disc list-inside space-y-1">
        <li><strong>Year</strong> &mdash; filter by the photo's year</li>
        <li><strong>Month</strong> &mdash; filter by month</li>
        <li><strong>Status</strong> &mdash; source, organized, or enhanced</li>
        <li><strong>Step</strong> &mdash; show images that have been through a specific processing step</li>
      </ul>
      <p>Filters are saved in the URL, so you can bookmark filtered views and use the browser back button to return to your previous state.</p>
    </div>
  </div>

  <div class="rounded-lg border p-4 space-y-3">
    <h2 class="font-medium">Duplicates</h2>
    <div class="text-sm text-muted-foreground space-y-2">
      <p>The Duplicates page uses perceptual hashing to find visually similar images. This helps identify photos that were accidentally scanned twice or exist in multiple folders. You can adjust the similarity threshold and bulk-delete unwanted copies.</p>
    </div>
  </div>

  <div class="rounded-lg border p-4 space-y-3">
    <h2 class="font-medium">Tips</h2>
    <div class="text-sm text-muted-foreground space-y-2">
      <ul class="list-disc list-inside space-y-1">
        <li>Always run <strong>Organize</strong> first &mdash; other steps require it</li>
        <li>Run <strong>Crop</strong> before Orient and Deskew for better results</li>
        <li>You can select multiple images in the gallery and process them as a batch</li>
        <li>Use the image detail page to compare before/after by switching between source, organized, and enhanced variants</li>
        <li>Set year and month metadata before organizing if the folder structure doesn't contain that info</li>
        <li>Enable <strong>CUDA</strong> in Settings if you have a compatible GPU for faster enhancement</li>
      </ul>
    </div>
  </div>
</div>

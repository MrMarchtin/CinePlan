<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import * as pdfjs from 'pdfjs-dist';
  import workerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url';
  import { save } from '@tauri-apps/plugin-dialog';
  import { errorMessage, exportDocument, readDocument } from './api';
  import type { DocumentFile } from './types';

  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

  let { doc, onclose }: { doc: DocumentFile; onclose: () => void } = $props();

  let canvas = $state<HTMLCanvasElement>();
  let loading = $state(true);
  let error = $state('');
  let page = $state(1);
  let pages = $state(0);
  let zoom = $state(1.2);
  let imageUrl = $state('');

  let query = $state('');
  let hits = $state<number[]>([]);
  let searched = $state(false);
  let searching = $state(false);

  // Objets pdf.js : volontairement hors de l'état réactif.
  let pdf: pdfjs.PDFDocumentProxy | null = null;
  let renderTask: pdfjs.RenderTask | null = null;

  onMount(async () => {
    try {
      const buffer = await readDocument(doc.id);
      if (doc.kind === 'image') {
        imageUrl = URL.createObjectURL(new Blob([buffer], { type: doc.mimeType }));
      } else {
        pdf = await pdfjs.getDocument({ data: new Uint8Array(buffer) }).promise;
        pages = pdf.numPages;
        await draw();
      }
    } catch (e) {
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    renderTask?.cancel();
    void pdf?.destroy();
    if (imageUrl) URL.revokeObjectURL(imageUrl);
  });

  async function draw() {
    if (!pdf || !canvas) return;
    renderTask?.cancel();
    const p = await pdf.getPage(page);
    const ratio = window.devicePixelRatio || 1;
    const viewport = p.getViewport({ scale: zoom * ratio });
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.style.width = `${viewport.width / ratio}px`;
    canvas.style.height = `${viewport.height / ratio}px`;
    const context = canvas.getContext('2d');
    if (!context) return;
    const task = p.render({ canvasContext: context, viewport });
    renderTask = task;
    try {
      await task.promise;
    } catch (e) {
      const name = (e as { name?: string })?.name;
      if (name !== 'RenderingCancelledException') error = errorMessage(e);
    }
  }

  function goTo(n: number) {
    if (!pdf) return;
    page = Math.min(Math.max(1, Math.round(n) || 1), pages);
    void draw();
  }

  function onPageInput(event: Event) {
    goTo(Number((event.currentTarget as HTMLInputElement).value));
  }

  function setZoom(value: number) {
    zoom = Math.min(4, Math.max(0.4, Math.round(value * 10) / 10));
    void draw();
  }

  async function search(event: SubmitEvent) {
    event.preventDefault();
    const q = query.trim().toLowerCase();
    hits = [];
    searched = false;
    if (!pdf || !q) return;
    searching = true;
    try {
      const found: number[] = [];
      for (let i = 1; i <= pdf.numPages; i++) {
        const p = await pdf.getPage(i);
        const content = await p.getTextContent();
        const text = content.items
          .map((item) => ('str' in item ? item.str : ''))
          .join(' ')
          .toLowerCase();
        if (text.includes(q)) found.push(i);
      }
      hits = found;
      searched = true;
    } catch (e) {
      error = errorMessage(e);
    } finally {
      searching = false;
    }
  }

  async function saveCopy() {
    error = '';
    try {
      const dest = await save({ defaultPath: doc.originalName });
      if (dest) await exportDocument(doc.id, dest);
    } catch (e) {
      error = errorMessage(e);
    }
  }
</script>

<div class="viewer">
  <div class="bar row">
    <button onclick={onclose}>← Retour aux documents</button>
    <b class="title">{doc.name}</b>
    <button onclick={saveCopy}>Enregistrer une copie</button>
  </div>

  {#if doc.kind === 'pdf' && !loading && !error}
    <div class="bar row">
      <button onclick={() => goTo(page - 1)} disabled={page <= 1} aria-label="Page précédente">←</button>
      <label>Page
        <input type="number" min="1" max={pages} value={page} onchange={onPageInput} style="width: 70px" />
        / {pages}
      </label>
      <button onclick={() => goTo(page + 1)} disabled={page >= pages} aria-label="Page suivante">→</button>
      <span class="sep"></span>
      <button onclick={() => setZoom(zoom - 0.2)} aria-label="Dézoomer">−</button>
      <span>{Math.round(zoom * 100)} %</span>
      <button onclick={() => setZoom(zoom + 0.2)} aria-label="Zoomer">+</button>
      <span class="sep"></span>
      <form class="row" onsubmit={search}>
        <input placeholder="Rechercher dans le document" bind:value={query} style="min-width: 220px" />
        <button disabled={searching}>{searching ? 'Recherche…' : 'Rechercher'}</button>
      </form>
    </div>
    {#if searched}
      <div class="bar row small">
        {#if hits.length === 0}
          Aucun résultat (les PDF scannés ne contiennent pas de texte recherchable).
        {:else}
          Trouvé page(s) :
          {#each hits as h (h)}<button class="link" onclick={() => goTo(h)}>{h}</button>{/each}
        {/if}
      </div>
    {/if}
  {/if}

  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if loading}<p class="empty">Ouverture du document…</p>{/if}

  <div class="stage">
    <canvas bind:this={canvas} hidden={doc.kind !== 'pdf' || loading || !!error}></canvas>
    {#if doc.kind === 'image' && imageUrl}
      <img src={imageUrl} alt={doc.name} style="max-width: 100%" />
    {/if}
  </div>
</div>

<style>
  .viewer { display: flex; flex-direction: column; gap: 8px; }
  .bar { gap: 8px; }
  .title { flex: 1; min-width: 120px; }
  .sep { width: 1px; height: 22px; background: var(--line); }
  .small { font-size: 13px; color: var(--mute); }
  .stage { overflow: auto; max-height: calc(100vh - 220px); background: #6b7478; border-radius: 8px; padding: 12px; text-align: center; }
  canvas { background: #fff; box-shadow: 0 2px 10px rgba(0, 0, 0, 0.35); }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    createFolder, deleteDocument, deleteFolder, errorMessage, importDocument,
    listDocuments, listFolders, listScenes, renameFolder, updateDocument,
  } from './api';
  import PdfViewer from './PdfViewer.svelte';
  import { fmtSize } from './time';
  import {
    CATEGORY_LABELS, type DocCategory, type DocumentFile, type Folder, type Project, type Scene,
  } from './types';

  let { project }: { project: Project } = $props();

  const categories = Object.entries(CATEGORY_LABELS) as [DocCategory, string][];

  let folders = $state<Folder[]>([]);
  let docs = $state<DocumentFile[]>([]);
  let scenes = $state<Scene[]>([]);
  let category = $state<DocCategory>('scenario');
  let folderId = $state<string | null>(null);
  let search = $state('');
  let error = $state('');
  let info = $state('');
  let viewing = $state<DocumentFile | null>(null);

  let newFolderName = $state('');
  let renamingId = $state<string | null>(null);
  let renameText = $state('');
  let renamingDocId = $state<string | null>(null);
  let docRenameText = $state('');

  async function refresh() {
    try {
      [folders, docs] = await Promise.all([listFolders(project.id), listDocuments(project.id)]);
    } catch (e) {
      error = errorMessage(e);
    }
  }

  onMount(async () => {
    try {
      scenes = await listScenes(project.id);
    } catch (e) {
      error = errorMessage(e);
    }
    await refresh();
  });

  const folderById = (id: string | null) => folders.find((f) => f.id === id) ?? null;

  /** Fil d'Ariane : du dossier racine jusqu'au dossier courant. */
  const crumbs = $derived.by(() => {
    const chain: Folder[] = [];
    let cursor = folderById(folderId);
    while (cursor) {
      chain.unshift(cursor);
      cursor = folderById(cursor.parentId);
    }
    return chain;
  });

  const subfolders = $derived(folders.filter((f) => f.category === category && f.parentId === folderId));
  const here = $derived(docs.filter((d) => d.category === category && d.folderId === folderId));
  const query = $derived(search.trim().toLowerCase());
  const results = $derived(query ? docs.filter((d) => d.name.toLowerCase().includes(query)) : []);
  const countOf = (c: DocCategory) => docs.filter((d) => d.category === c).length;

  function folderPath(f: Folder): string {
    const names: string[] = [];
    let cursor: Folder | null = f;
    while (cursor) {
      names.unshift(cursor.name);
      cursor = folderById(cursor.parentId);
    }
    return names.join(' / ');
  }

  /** Destinations possibles pour « Déplacer vers… » : `catégorie|dossier`. */
  const destinations = $derived.by(() => {
    const list: { value: string; label: string }[] = [];
    for (const [c, label] of categories) {
      list.push({ value: `${c}|`, label });
      for (const f of folders.filter((x) => x.category === c)) {
        list.push({ value: `${c}|${f.id}`, label: `${label} / ${folderPath(f)}` });
      }
    }
    return list;
  });

  function locationOf(d: DocumentFile): string {
    const f = folderById(d.folderId);
    return f ? `${CATEGORY_LABELS[d.category]} / ${folderPath(f)}` : CATEGORY_LABELS[d.category];
  }

  function chooseCategory(c: DocCategory) {
    category = c;
    folderId = null;
    search = '';
  }

  function enter(f: Folder | null) {
    folderId = f ? f.id : null;
  }

  async function run(action: () => Promise<unknown>) {
    error = '';
    info = '';
    try {
      await action();
      await refresh();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function addFiles() {
    error = '';
    info = '';
    let picked: string | string[] | null;
    try {
      picked = await open({
        multiple: true,
        filters: [{ name: 'PDF et images', extensions: ['pdf', 'png', 'jpg', 'jpeg', 'webp'] }],
      });
    } catch (e) {
      error = errorMessage(e);
      return;
    }
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    const failures: string[] = [];
    for (const path of paths) {
      try {
        await importDocument({
          projectId: project.id, sourcePath: path, category, folderId, sceneId: null, memberId: null,
        });
      } catch (e) {
        const name = path.split(/[\\/]/).pop() ?? path;
        failures.push(`${name} : ${errorMessage(e)}`);
      }
    }
    if (failures.length) error = failures.join(' — ');
    else info = `${paths.length} document(s) ajouté(s).`;
    await refresh();
  }

  function addFolder(event: SubmitEvent) {
    event.preventDefault();
    if (!newFolderName.trim()) return;
    void run(async () => {
      await createFolder({ projectId: project.id, parentId: folderId, category, name: newFolderName });
      newFolderName = '';
    });
  }

  function startRename(f: Folder) {
    renamingId = f.id;
    renameText = f.name;
  }

  function commitRename(f: Folder, event: SubmitEvent) {
    event.preventDefault();
    void run(async () => {
      await renameFolder({ id: f.id, expectedVersion: f.version, name: renameText });
      renamingId = null;
    });
  }

  function removeFolder(f: Folder) {
    if (!confirm(`Supprimer le dossier « ${f.name} » ?`)) return;
    void run(() => deleteFolder(f.id));
  }

  function startDocRename(d: DocumentFile) {
    renamingDocId = d.id;
    docRenameText = d.name;
  }

  function docUpdate(d: DocumentFile, changes: Partial<{ name: string; category: DocCategory; folderId: string | null; sceneId: string | null }>) {
    return updateDocument({
      id: d.id,
      expectedVersion: d.version,
      name: d.name,
      category: d.category,
      folderId: d.folderId,
      sceneId: d.sceneId,
      memberId: d.memberId,
      ...changes,
    });
  }

  function commitDocRename(d: DocumentFile, event: SubmitEvent) {
    event.preventDefault();
    void run(async () => {
      await docUpdate(d, { name: docRenameText });
      renamingDocId = null;
    });
  }

  function moveDoc(d: DocumentFile, event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    if (!value) return;
    const [c, f] = value.split('|');
    void run(() => docUpdate(d, { category: c as DocCategory, folderId: f || null }));
  }

  function linkScene(d: DocumentFile, event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    void run(() => docUpdate(d, { sceneId: value || null }));
  }

  function removeDoc(d: DocumentFile) {
    if (!confirm(`Supprimer « ${d.name} » ? Le fichier sera effacé.`)) return;
    void run(() => deleteDocument(d.id));
  }
</script>

{#if viewing}
  <PdfViewer doc={viewing} onclose={() => (viewing = null)} />
{:else}
  <h2>Documents</h2>
  <p class="lead">Scénario, storyboard, plans, autorisations… tout est rangé par catégorie et s’ouvre dans CinePlan.</p>

  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if info}<p class="info" role="status">{info}</p>{/if}

  <div class="layout">
    <nav class="cats" aria-label="Catégories">
      {#each categories as [c, label] (c)}
        <button class="cat" class:on={c === category && !query} onclick={() => chooseCategory(c)}>
          <span>{label}</span><span class="count">{countOf(c)}</span>
        </button>
      {/each}
    </nav>

    <section>
      <div class="pane">
        <div class="row">
          <button class="primary" onclick={addFiles}>Ajouter des documents</button>
          <input type="search" placeholder="Rechercher un document" bind:value={search} style="flex: 1; min-width: 180px" />
        </div>
        <p class="note">PDF et images (PNG, JPEG, WebP), 100 Mo maximum par fichier. Une copie est conservée dans CinePlan.</p>
      </div>

      {#if query}
        <div class="pane">
          <h3>Résultats pour « {search.trim()} »</h3>
          {#if results.length === 0}<p class="empty">Aucun document trouvé.</p>{/if}
          {#each results as d (d.id)}
            <div class="doc">
              <button class="link name" onclick={() => (viewing = d)}>{d.name}</button>
              <span class="meta">{locationOf(d)}</span>
            </div>
          {/each}
        </div>
      {:else}
        <div class="pane">
          <div class="row crumbs">
            <button class="link" onclick={() => enter(null)}>{CATEGORY_LABELS[category]}</button>
            {#each crumbs as f (f.id)}
              <span>/</span><button class="link" onclick={() => enter(f)}>{f.name}</button>
            {/each}
          </div>

          <form class="row" onsubmit={addFolder} style="margin: 8px 0">
            <input placeholder="Nouveau dossier ici" bind:value={newFolderName} />
            <button>Créer le dossier</button>
          </form>

          {#each subfolders as f (f.id)}
            <div class="doc">
              {#if renamingId === f.id}
                <form class="row" onsubmit={(e) => commitRename(f, e)}>
                  <input bind:value={renameText} required />
                  <button class="primary">Valider</button>
                  <button type="button" onclick={() => (renamingId = null)}>Annuler</button>
                </form>
              {:else}
                <button class="link name" onclick={() => enter(f)}>📁 {f.name}</button>
                <span class="meta"></span>
                <span class="actions">
                  <button class="link" onclick={() => startRename(f)}>Renommer</button>
                  <button class="link danger" onclick={() => removeFolder(f)}>Supprimer</button>
                </span>
              {/if}
            </div>
          {/each}

          {#each here as d (d.id)}
            <div class="doc">
              {#if renamingDocId === d.id}
                <form class="row" onsubmit={(e) => commitDocRename(d, e)}>
                  <input bind:value={docRenameText} required />
                  <button class="primary">Valider</button>
                  <button type="button" onclick={() => (renamingDocId = null)}>Annuler</button>
                </form>
              {:else}
                <button class="link name" onclick={() => (viewing = d)}>
                  {d.kind === 'pdf' ? '📄' : '🖼️'} {d.name}
                </button>
                <span class="meta">{fmtSize(d.sizeBytes)} · {new Date(d.createdAt).toLocaleDateString('fr-FR')}</span>
                <span class="actions">
                  <select value={d.sceneId ?? ''} onchange={(e) => linkScene(d, e)} aria-label="Scène liée">
                    <option value="">Aucune scène liée</option>
                    {#each scenes as s (s.id)}<option value={s.id}>Scène {s.number}</option>{/each}
                  </select>
                  <select value="" onchange={(e) => moveDoc(d, e)} aria-label="Déplacer vers">
                    <option value="">Déplacer vers…</option>
                    {#each destinations as dest (dest.value)}<option value={dest.value}>{dest.label}</option>{/each}
                  </select>
                  <button class="link" onclick={() => startDocRename(d)}>Renommer</button>
                  <button class="link danger" onclick={() => removeDoc(d)}>Supprimer</button>
                </span>
              {/if}
            </div>
          {/each}

          {#if subfolders.length === 0 && here.length === 0}
            <p class="empty">Rien ici pour l’instant. Ajoutez un document ou créez un dossier.</p>
          {/if}
        </div>
      {/if}
    </section>
  </div>
{/if}

<style>
  .layout { display: grid; grid-template-columns: 230px 1fr; gap: 16px; align-items: start; }
  .cats { display: flex; flex-direction: column; gap: 2px; background: var(--pane); border: 1px solid var(--line); border-radius: 8px; padding: 6px; }
  .cat { display: flex; justify-content: space-between; gap: 8px; text-align: left; border: 0; border-left: 3px solid transparent; background: none; padding: 7px 8px; border-radius: 5px; }
  .cat:hover { background: var(--bg); }
  .cat.on { background: var(--bg); border-left-color: var(--acc); font-weight: 700; }
  .count { color: var(--mute); font-size: 12px; }
  .crumbs { gap: 6px; color: var(--mute); }
  .doc { display: flex; flex-wrap: wrap; align-items: center; gap: 6px 12px; padding: 8px 4px; border-bottom: 1px solid var(--line); }
  .name { font-weight: 700; color: var(--ink); text-align: left; }
  .meta { color: var(--mute); font-size: 13px; flex: 1; }
  .actions { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
  .actions select { max-width: 170px; font-size: 13px; }
  .note { color: var(--mute); font-size: 13px; margin: 8px 0 0; }
  .info { color: #1b7a43; font-weight: 600; margin: 0 0 12px; }
  @media (max-width: 860px) { .layout { grid-template-columns: 1fr; } }
</style>

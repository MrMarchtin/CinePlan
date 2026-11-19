<script lang="ts">
  import { onMount } from 'svelte';
  import { createProject, deleteProject, errorMessage, listProjects } from './api';
  import { STATUS_LABELS, type Project } from './types';

  let { onopen }: { onopen: (project: Project) => void } = $props();

  let projects = $state<Project[]>([]);
  let loading = $state(true);
  let error = $state('');
  let title = $state('');
  let genre = $state('');
  let description = $state('');

  async function refresh() {
    try {
      projects = await listProjects();
    } catch (e) {
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  async function add(event: SubmitEvent) {
    event.preventDefault();
    error = '';
    try {
      const project = await createProject({
        title,
        description,
        genre: genre || null,
        estimatedDurationMin: null,
        startDate: null,
        endDate: null,
      });
      title = genre = description = '';
      onopen(project);
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function remove(project: Project) {
    if (!confirm(`Supprimer le projet « ${project.title} » ?`)) return;
    error = '';
    try {
      await deleteProject(project.id);
      await refresh();
    } catch (e) {
      error = errorMessage(e);
    }
  }
</script>

<h2>Projets</h2>
<p class="lead">Une production = un espace de travail.</p>

{#if error}<p class="error" role="alert">{error}</p>{/if}

<form class="pane" onsubmit={add}>
  <h3>Nouveau projet</h3>
  <div class="row">
    <input placeholder="Titre du film" bind:value={title} required style="flex: 2; min-width: 200px" />
    <input placeholder="Genre" bind:value={genre} style="flex: 1; min-width: 120px" />
  </div>
  <div class="row" style="margin-top: 8px">
    <input placeholder="Description (facultatif)" bind:value={description} style="flex: 1" />
    <button class="primary">Créer le projet</button>
  </div>
</form>

<div class="pane">
  {#if loading}
    <p class="empty">Chargement…</p>
  {:else if projects.length === 0}
    <p class="empty">Aucun projet pour l’instant. Créez le premier ci-dessus.</p>
  {:else}
    <table>
      <thead><tr><th>Titre</th><th>Genre</th><th>Statut</th><th></th></tr></thead>
      <tbody>
        {#each projects as project (project.id)}
          <tr>
            <td><button class="link" onclick={() => onopen(project)}><b>{project.title}</b></button></td>
            <td>{project.genre ?? '—'}</td>
            <td><span class="badge">{STATUS_LABELS[project.status]}</span></td>
            <td style="text-align: right">
              <button class="link danger" onclick={() => remove(project)}>Supprimer</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

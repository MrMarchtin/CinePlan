<script lang="ts">
  import { onMount } from 'svelte';
  import { createScene, deleteScene, errorMessage, listScenes, updateProject } from './api';
  import {
    INT_EXT_LABELS,
    STATUS_LABELS,
    TIME_LABELS,
    type IntExt,
    type Project,
    type ProjectStatus,
    type Scene,
    type TimeOfDay,
  } from './types';

  let {
    project,
    onback,
    onupdate,
  }: {
    project: Project;
    onback: () => void;
    onupdate: (project: Project) => void;
  } = $props();

  let scenes = $state<Scene[]>([]);
  let error = $state('');

  let number = $state(1);
  let title = $state('');
  let intExt = $state<IntExt>('int');
  let timeOfDay = $state<TimeOfDay>('jour');
  let location = $state('');
  let duration = $state<number | null>(null);

  const nextNumber = () => Math.max(0, ...scenes.map((s) => s.number)) + 1;
  const totalMinutes = $derived(scenes.reduce((sum, s) => sum + (s.estimatedDurationMin ?? 0), 0));

  async function refresh() {
    try {
      scenes = await listScenes(project.id);
      number = nextNumber();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  onMount(refresh);

  async function addScene(event: SubmitEvent) {
    event.preventDefault();
    error = '';
    try {
      await createScene({
        projectId: project.id,
        number,
        title,
        intExt,
        timeOfDay,
        locationName: location || null,
        summary: '',
        estimatedDurationMin: duration,
      });
      title = location = '';
      duration = null;
      await refresh();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function removeScene(scene: Scene) {
    if (!confirm(`Supprimer la scène ${scene.number} ?`)) return;
    error = '';
    try {
      await deleteScene(scene.id);
      await refresh();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function changeStatus(status: ProjectStatus) {
    error = '';
    try {
      const updated = await updateProject({
        id: project.id,
        expectedVersion: project.version,
        title: project.title,
        description: project.description,
        genre: project.genre,
        estimatedDurationMin: project.estimatedDurationMin,
        startDate: project.startDate,
        endDate: project.endDate,
        status,
      });
      onupdate(updated);
    } catch (e) {
      error = errorMessage(e);
    }
  }
</script>

<button class="link" onclick={onback}>← Projets</button>
<h2>{project.title}</h2>
<p class="lead">
  {project.genre ?? 'Genre non renseigné'} · {scenes.length} scène(s)
  {#if totalMinutes > 0}· {totalMinutes} min estimées{/if}
</p>

{#if error}<p class="error" role="alert">{error}</p>{/if}

<div class="pane row">
  <label for="status">Statut</label>
  <select
    id="status"
    value={project.status}
    onchange={(e) => changeStatus(e.currentTarget.value as ProjectStatus)}
  >
    {#each Object.entries(STATUS_LABELS) as [value, label]}
      <option {value}>{label}</option>
    {/each}
  </select>
</div>

<form class="pane" onsubmit={addScene}>
  <h3>Ajouter une scène</h3>
  <div class="row">
    <input type="number" min="1" bind:value={number} aria-label="Numéro" style="width: 80px" />
    <input placeholder="Titre" bind:value={title} required style="flex: 2; min-width: 160px" />
    <select bind:value={intExt} aria-label="Intérieur ou extérieur">
      {#each Object.entries(INT_EXT_LABELS) as [value, label]}<option {value}>{label}</option>{/each}
    </select>
    <select bind:value={timeOfDay} aria-label="Moment de la journée">
      {#each Object.entries(TIME_LABELS) as [value, label]}<option {value}>{label}</option>{/each}
    </select>
  </div>
  <div class="row" style="margin-top: 8px">
    <input placeholder="Lieu" bind:value={location} style="flex: 1" />
    <input type="number" min="1" placeholder="Durée (min)" bind:value={duration} style="width: 120px" />
    <button class="primary">Ajouter</button>
  </div>
</form>

<div class="pane">
  <h3>Scènes</h3>
  {#if scenes.length === 0}
    <p class="empty">Aucune scène. Ajoutez la première ci-dessus.</p>
  {:else}
    <table>
      <thead><tr><th>N°</th><th>Titre</th><th>Décor</th><th>Lieu</th><th>Durée</th><th></th></tr></thead>
      <tbody>
        {#each scenes as scene (scene.id)}
          <tr>
            <td><b>{scene.number}</b></td>
            <td>{scene.title}</td>
            <td>{INT_EXT_LABELS[scene.intExt]} {TIME_LABELS[scene.timeOfDay]}</td>
            <td>{scene.locationName ?? '—'}</td>
            <td>{scene.estimatedDurationMin ? `${scene.estimatedDurationMin} min` : '—'}</td>
            <td style="text-align: right">
              <button class="link danger" onclick={() => removeScene(scene)}>Supprimer</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

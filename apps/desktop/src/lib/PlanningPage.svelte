<script lang="ts">
  import { onMount } from 'svelte';
  import { createDay, errorMessage, listDays, listMembers, listScenes } from './api';
  import DayEditor from './DayEditor.svelte';
  import { fmtDate, fromInput, hm, isoDate, toInput } from './time';
  import type { Project, Scene, ShootingDay, TeamMember } from './types';

  let { project }: { project: Project } = $props();

  const MONTHS = ['janvier', 'février', 'mars', 'avril', 'mai', 'juin', 'juillet', 'août', 'septembre', 'octobre', 'novembre', 'décembre'];
  const WEEKDAYS = ['lun.', 'mar.', 'mer.', 'jeu.', 'ven.', 'sam.', 'dim.'];

  const today = new Date();
  let year = $state(today.getFullYear());
  let month = $state(today.getMonth());

  let days = $state<ShootingDay[]>([]);
  let scenes = $state<Scene[]>([]);
  let members = $state<TeamMember[]>([]);
  let selectedId = $state<string | null>(null);
  let creatingDate = $state<string | null>(null);
  let newStart = $state('08:00');
  let newEnd = $state('18:00');
  let newLocation = $state('');
  let error = $state('');

  const byDate = $derived(new Map(days.map((d) => [d.date, d])));

  const cells = $derived.by(() => {
    const offset = (new Date(year, month, 1).getDay() + 6) % 7; // lundi = 0
    const count = new Date(year, month + 1, 0).getDate();
    const list: (number | null)[] = [];
    for (let i = 0; i < offset; i++) list.push(null);
    for (let d = 1; d <= count; d++) list.push(d);
    while (list.length % 7 !== 0) list.push(null);
    return list;
  });

  async function refreshDays() {
    try {
      days = await listDays(project.id);
    } catch (e) {
      error = errorMessage(e);
    }
  }

  onMount(async () => {
    try {
      [scenes, members] = await Promise.all([listScenes(project.id), listMembers(project.id)]);
    } catch (e) {
      error = errorMessage(e);
    }
    await refreshDays();
  });

  function shiftMonth(delta: number) {
    const d = new Date(year, month + delta, 1);
    year = d.getFullYear();
    month = d.getMonth();
  }

  function clickCell(day: number) {
    const iso = isoDate(year, month, day);
    const existing = byDate.get(iso);
    error = '';
    if (existing) {
      selectedId = existing.id;
      creatingDate = null;
    } else {
      selectedId = null;
      creatingDate = iso;
    }
  }

  async function create(event: SubmitEvent) {
    event.preventDefault();
    if (!creatingDate) return;
    error = '';
    try {
      const day = await createDay({
        projectId: project.id,
        date: creatingDate,
        startMin: fromInput(newStart),
        endMin: fromInput(newEnd),
        location: newLocation || null,
        notes: '',
        memberIds: [],
      });
      creatingDate = null;
      newLocation = '';
      await refreshDays();
      selectedId = day.id;
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function closeEditor() {
    selectedId = null;
    await refreshDays();
  }
</script>

<h2>Planning de tournage</h2>
<p class="lead">Cliquez sur une date pour créer une journée de tournage, ou sur une journée existante pour l’organiser.</p>

{#if error}<p class="error" role="alert">{error}</p>{/if}

<div class="pane">
  <div class="row cal-head">
    <button onclick={() => shiftMonth(-1)} aria-label="Mois précédent">←</button>
    <b class="cal-title">{MONTHS[month]} {year}</b>
    <button onclick={() => shiftMonth(1)} aria-label="Mois suivant">→</button>
  </div>
  <div class="cal">
    {#each WEEKDAYS as w (w)}<div class="cal-wd">{w}</div>{/each}
    {#each cells as day, i (i)}
      {#if day === null}
        <div class="cal-cell empty-cell"></div>
      {:else}
        {@const iso = isoDate(year, month, day)}
        {@const shoot = byDate.get(iso)}
        <button
          class="cal-cell"
          class:shoot={!!shoot}
          class:sel={shoot?.id === selectedId || iso === creatingDate}
          onclick={() => clickCell(day)}
        >
          <span class="cal-num">{day}</span>
          {#if shoot}
            <span class="cal-tag">{hm(shoot.startMin)}–{hm(shoot.endMin)}</span>
            {#if shoot.location}<span class="cal-loc">{shoot.location}</span>{/if}
          {/if}
        </button>
      {/if}
    {/each}
  </div>
</div>

{#if creatingDate}
  <form class="pane" onsubmit={create}>
    <h3>Nouvelle journée : {fmtDate(creatingDate)}</h3>
    <div class="row">
      <label for="d-start">Début</label>
      <input id="d-start" type="time" bind:value={newStart} required />
      <label for="d-end">Fin prévue</label>
      <input id="d-end" type="time" bind:value={newEnd} required />
      <input placeholder="Lieu (facultatif)" bind:value={newLocation} style="flex: 1; min-width: 160px" />
      <button class="primary">Créer la journée</button>
      <button type="button" onclick={() => (creatingDate = null)}>Annuler</button>
    </div>
  </form>
{/if}

{#if selectedId}
  {#key selectedId}
    <DayEditor
      dayId={selectedId}
      {scenes}
      {members}
      onchanged={refreshDays}
      onclose={closeEditor}
    />
  {/key}
{/if}

<style>
  .cal-head { justify-content: space-between; margin-bottom: 10px; }
  .cal-title { text-transform: capitalize; font-size: 17px; }
  .cal { display: grid; grid-template-columns: repeat(7, 1fr); gap: 4px; }
  .cal-wd { text-align: center; color: var(--mute); font-size: 12px; padding: 2px 0; }
  .cal-cell { min-height: 74px; display: flex; flex-direction: column; align-items: flex-start; gap: 2px; padding: 6px; text-align: left; border-radius: 6px; border: 1px solid var(--line); background: var(--pane); }
  .cal-cell:hover:not(.empty-cell) { border-color: var(--acc); }
  .empty-cell { border: 0; background: none; }
  .cal-cell.shoot { background: #f6e27a; border-color: #c9ae2c; color: #16252b; }
  .cal-cell.sel { outline: 2px solid var(--acc); outline-offset: 1px; }
  .cal-num { font-weight: 700; }
  .cal-tag { font-size: 12px; }
  .cal-loc { font-size: 11px; opacity: 0.8; }
</style>

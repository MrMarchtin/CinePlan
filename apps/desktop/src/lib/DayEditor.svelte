<script lang="ts">
  import { onMount } from 'svelte';
  import {
    addBreakItem, addSceneItem, deleteDay, errorMessage, getDayView,
    removeItem, reorderItems, updateDay, updateItem,
  } from './api';
  import { fmtDate, fromInput, hm, toInput } from './time';
  import type { DayItem, DayView, Scene, TeamMember } from './types';

  let {
    dayId,
    scenes,
    members,
    onchanged,
    onclose,
  }: {
    dayId: string;
    scenes: Scene[];
    members: TeamMember[];
    onchanged: () => void;
    onclose: () => void;
  } = $props();

  let view = $state<DayView | null>(null);
  let error = $state('');

  // en-tête de journée
  let startStr = $state('08:00');
  let endStr = $state('18:00');
  let location = $state('');
  let notes = $state('');
  let memberIds = $state<string[]>([]);

  // ajout d'éléments
  let sceneToAdd = $state('');
  let sceneDuration = $state<number | null>(null);
  let breakLabel = $state('Déjeuner');
  let breakDuration = $state(60);
  let breakFixed = $state(false);
  let breakTime = $state('12:00');

  const BREAK_PRESETS = ['Déjeuner', 'Pause technique', 'Installation', 'Changement de décor', 'Autre pause'];

  const freeScenes = $derived(
    view ? scenes.filter((s) => !view!.items.some((i) => i.sceneId === s.id)) : [],
  );

  function applyView(v: DayView) {
    view = v;
    startStr = toInput(v.day.startMin);
    endStr = toInput(v.day.endMin);
    location = v.day.location ?? '';
    notes = v.day.notes;
    memberIds = [...v.day.memberIds];
  }

  async function reload() {
    applyView(await getDayView(dayId));
    onchanged();
  }

  async function run(action: () => Promise<unknown>) {
    error = '';
    try {
      await action();
      await reload();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  onMount(async () => {
    try {
      applyView(await getDayView(dayId));
    } catch (e) {
      error = errorMessage(e);
    }
  });

  const sceneOf = (item: DayItem) => scenes.find((s) => s.id === item.sceneId);

  function titleOf(item: DayItem): string {
    if (item.kind === 'break') return item.label;
    const s = sceneOf(item);
    return s ? `Scène ${s.number} — ${s.title}` : 'Scène supprimée';
  }

  function memberName(id: string): string {
    const m = members.find((x) => x.id === id);
    return m ? `${m.firstName} ${m.lastName}` : 'Membre inconnu';
  }

  function toggleMember(id: string) {
    memberIds = memberIds.includes(id) ? memberIds.filter((x) => x !== id) : [...memberIds, id];
  }

  function saveHeader(event: SubmitEvent) {
    event.preventDefault();
    if (!view) return;
    const v = view;
    void run(() =>
      updateDay({
        id: v.day.id,
        expectedVersion: v.day.version,
        date: v.day.date,
        startMin: fromInput(startStr),
        endMin: fromInput(endStr),
        location: location || null,
        notes,
        memberIds,
      }),
    );
  }

  function patch(item: DayItem, changes: Partial<{ durationMin: number; delayMin: number; fixedStartMin: number | null }>) {
    return {
      id: item.id,
      durationMin: item.durationMin,
      delayMin: item.delayMin,
      label: item.kind === 'break' ? item.label : null,
      fixedStartMin: item.fixedStartMin,
      ...changes,
    };
  }

  function bump(item: DayItem, minutes: number) {
    void run(() => updateItem(patch(item, { delayMin: item.delayMin + minutes })));
  }

  function numberFrom(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value) || 0;
  }

  function setDelay(item: DayItem, event: Event) {
    void run(() => updateItem(patch(item, { delayMin: numberFrom(event) })));
  }

  function setDuration(item: DayItem, event: Event) {
    void run(() => updateItem(patch(item, { durationMin: numberFrom(event) })));
  }

  function toggleFixed(item: DayItem, plannedStart: number) {
    const next = item.fixedStartMin === null ? Math.min(plannedStart, 1439) : null;
    void run(() => updateItem(patch(item, { fixedStartMin: next })));
  }

  function setFixedTime(item: DayItem, event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value;
    if (!value) return;
    void run(() => updateItem(patch(item, { fixedStartMin: fromInput(value) })));
  }

  function move(index: number, direction: -1 | 1) {
    if (!view) return;
    const ids = view.items.map((i) => i.id);
    const target = index + direction;
    if (target < 0 || target >= ids.length) return;
    [ids[index], ids[target]] = [ids[target], ids[index]];
    void run(() => reorderItems(dayId, ids));
  }

  function onSceneChosen() {
    const s = scenes.find((x) => x.id === sceneToAdd);
    sceneDuration = s?.estimatedDurationMin ?? 30;
  }

  function addScene(event: SubmitEvent) {
    event.preventDefault();
    if (!sceneToAdd) return;
    void run(async () => {
      await addSceneItem({ dayId, sceneId: sceneToAdd, durationMin: sceneDuration });
      sceneToAdd = '';
      sceneDuration = null;
    });
  }

  function addBreak(event: SubmitEvent) {
    event.preventDefault();
    void run(() =>
      addBreakItem({
        dayId,
        label: breakLabel,
        durationMin: breakDuration,
        fixedStartMin: breakFixed ? fromInput(breakTime) : null,
      }),
    );
  }

  function removeDay() {
    if (!view || !confirm(`Supprimer la journée du ${fmtDate(view.day.date)} ?`)) return;
    error = '';
    deleteDay(dayId).then(onclose, (e) => (error = errorMessage(e)));
  }

  function changed(slot: { plannedStart: number; actualStart: number; plannedEnd: number; actualEnd: number }) {
    return slot.plannedStart !== slot.actualStart || slot.plannedEnd !== slot.actualEnd;
  }
</script>

<div class="pane">
  {#if error}<p class="error" role="alert">{error}</p>{/if}

  {#if !view}
    <p class="empty">Chargement…</p>
  {:else}
    {@const summary = view.schedule.summary}
    <div class="row" style="justify-content: space-between">
      <h3 style="margin: 0; text-transform: capitalize">{fmtDate(view.day.date)}</h3>
      <div class="row">
        <button class="link danger" onclick={removeDay}>Supprimer la journée</button>
        <button onclick={onclose}>Fermer</button>
      </div>
    </div>

    <form onsubmit={saveHeader} style="margin-top: 12px">
      <div class="row">
        <label for="e-start">Début</label>
        <input id="e-start" type="time" bind:value={startStr} required />
        <label for="e-end">Fin prévue</label>
        <input id="e-end" type="time" bind:value={endStr} required />
        <input placeholder="Lieu" bind:value={location} style="flex: 1; min-width: 160px" />
      </div>
      <div class="row" style="margin-top: 8px">
        <input placeholder="Notes de la journée" bind:value={notes} style="flex: 1" />
      </div>
      <fieldset class="members">
        <legend>Personnes convoquées</legend>
        {#if members.length === 0}
          <span class="empty">Ajoutez d’abord des membres dans « Équipe ».</span>
        {/if}
        {#each members as m (m.id)}
          <label class="member">
            <input type="checkbox" checked={memberIds.includes(m.id)} onchange={() => toggleMember(m.id)} />
            {m.firstName} {m.lastName}
            {#if view.unavailableMemberIds.includes(m.id) && memberIds.includes(m.id)}
              <span class="badge bad">indisponible ce jour</span>
            {/if}
          </label>
        {/each}
      </fieldset>
      <button class="primary" style="margin-top: 8px">Enregistrer la journée</button>
    </form>

    {#if view.unavailableMemberIds.length > 0}
      <p class="error" role="alert">
        Conflit de disponibilité : {view.unavailableMemberIds.map(memberName).join(', ')}
        {view.unavailableMemberIds.length > 1 ? 'sont indisponibles' : 'est indisponible'} à cette date.
      </p>
    {/if}

    <h3 style="margin-top: 20px">Déroulé de la journée</h3>

    <div class="summary" class:over={summary.overtimeMin > 0}>
      <span>Fin prévue de la journée : <b>{hm(summary.dayEnd)}</b></span>
      <span>Fin selon le prévisionnel : <b>{hm(summary.plannedEnd)}</b></span>
      <span>Fin recalculée : <b>{hm(summary.actualEnd)}</b></span>
    </div>
    {#if summary.overtimeMin > 0}
      <p class="error" role="alert">
        La journée dépasse l’heure de fin prévue de {summary.overtimeMin} min.
      </p>
    {/if}

    {#if view.items.length === 0}
      <p class="empty">Aucune scène ni pause. Ajoutez-en ci-dessous.</p>
    {/if}

    {#each view.items as item, i (item.id)}
      {@const slot = view.schedule.slots[i]}
      <div class="slot" class:is-break={item.kind === 'break'} class:late={slot.shiftMin > 0}>
        <div class="slot-time">
          {#if changed(slot)}
            <s>{hm(slot.plannedStart)}–{hm(slot.plannedEnd)}</s>
            <b class="recalc">{hm(slot.actualStart)}–{hm(slot.actualEnd)}</b>
          {:else}
            <b>{hm(slot.plannedStart)}–{hm(slot.plannedEnd)}</b>
          {/if}
        </div>
        <div class="slot-main">
          <b>{titleOf(item)}</b>
          <div class="row small">
            <label>Durée prévue
              <input type="number" min="1" value={item.durationMin} onchange={(e) => setDuration(item, e)} style="width: 70px" /> min
            </label>
            <label>Retard (+) / avance (−)
              <input type="number" step="5" value={item.delayMin} onchange={(e) => setDelay(item, e)} style="width: 70px" /> min
            </label>
            <button type="button" onclick={() => bump(item, 5)}>+5</button>
            <button type="button" onclick={() => bump(item, 15)}>+15</button>
            <button type="button" onclick={() => bump(item, -5)}>−5</button>
            {#if item.delayMin !== 0}
              <button type="button" class="link" onclick={() => run(() => updateItem(patch(item, { delayMin: 0 })))}>Remettre à zéro</button>
            {/if}
          </div>
          {#if item.kind === 'break'}
            <div class="row small">
              <label>
                <input type="checkbox" checked={item.fixedStartMin !== null} onchange={() => toggleFixed(item, slot.plannedStart)} />
                Heure fixe
              </label>
              {#if item.fixedStartMin !== null}
                <input type="time" value={toInput(item.fixedStartMin)} onchange={(e) => setFixedTime(item, e)} />
              {/if}
            </div>
          {/if}
          {#if slot.conflict === 'fixed_break_pushed'}
            <p class="error small">
              Le retard fait empiéter la suite sur cette pause fixe ({slot.conflictMin} min). Décalez la pause
              (décochez « Heure fixe »), raccourcissez une scène ou déplacez une scène.
            </p>
          {:else if slot.conflict === 'fixed_break_overlap'}
            <p class="error small">
              Même sans retard, le déroulé prévu dépasse l’heure fixe de cette pause ({slot.conflictMin} min).
            </p>
          {/if}
        </div>
        <div class="slot-actions">
          <button type="button" onclick={() => move(i, -1)} disabled={i === 0} aria-label="Monter">↑</button>
          <button type="button" onclick={() => move(i, 1)} disabled={i === view.items.length - 1} aria-label="Descendre">↓</button>
          <button type="button" class="link danger" onclick={() => run(() => removeItem(item.id))}>Retirer</button>
        </div>
      </div>
    {/each}

    <div class="adders">
      <form class="row" onsubmit={addScene}>
        <select bind:value={sceneToAdd} onchange={onSceneChosen} aria-label="Scène à ajouter">
          <option value="">Ajouter une scène…</option>
          {#each freeScenes as s (s.id)}<option value={s.id}>Scène {s.number} — {s.title}</option>{/each}
        </select>
        <input type="number" min="1" placeholder="Durée (min)" bind:value={sceneDuration} style="width: 110px" />
        <button class="primary" disabled={!sceneToAdd}>Ajouter la scène</button>
      </form>

      <form class="row" onsubmit={addBreak}>
        <select bind:value={breakLabel} aria-label="Type de pause">
          {#each BREAK_PRESETS as p (p)}<option>{p}</option>{/each}
        </select>
        <input type="number" min="1" bind:value={breakDuration} style="width: 80px" aria-label="Durée de la pause" /> min
        <label><input type="checkbox" bind:checked={breakFixed} /> Heure fixe</label>
        {#if breakFixed}<input type="time" bind:value={breakTime} />{/if}
        <button>Ajouter la pause</button>
      </form>
    </div>
  {/if}
</div>

<style>
  .members { border: 1px solid var(--line); border-radius: 6px; margin: 10px 0 0; display: flex; flex-wrap: wrap; gap: 6px 18px; }
  .member { display: inline-flex; gap: 6px; align-items: center; }
  .summary { display: flex; flex-wrap: wrap; gap: 6px 22px; padding: 8px 12px; border-radius: 6px; background: var(--bg); margin-bottom: 10px; }
  .summary.over { outline: 2px solid var(--bad); }
  .slot { display: grid; grid-template-columns: 150px 1fr auto; gap: 12px; padding: 10px; border: 1px solid var(--line); border-left: 4px solid var(--acc); border-radius: 6px; margin-bottom: 8px; }
  .slot.is-break { border-left-color: var(--mute); background: var(--bg); }
  .slot-time { display: flex; flex-direction: column; font-variant-numeric: tabular-nums; }
  .slot-time s { color: var(--mute); font-size: 13px; }
  .recalc { color: #b36b00; }
  .slot-actions { display: flex; flex-direction: column; gap: 4px; align-items: flex-end; }
  .small { font-size: 13px; margin: 4px 0 0; }
  .adders { display: grid; gap: 10px; margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--line); }
  @media (max-width: 760px) { .slot { grid-template-columns: 1fr; } }
</style>

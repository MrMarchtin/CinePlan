<script lang="ts">
  import { onMount } from 'svelte';
  import { createMember, deleteMember, errorMessage, listMembers, updateMember } from './api';
  import { fmtDate } from './time';
  import { ROLE_LABELS, type MemberRole, type Project, type TeamMember } from './types';

  let { project }: { project: Project } = $props();

  let members = $state<TeamMember[]>([]);
  let error = $state('');
  let filter = $state<MemberRole | 'all'>('all');

  // formulaire (création ou modification)
  let editing = $state<TeamMember | null>(null);
  let firstName = $state('');
  let lastName = $state('');
  let role = $state<MemberRole>('acteur');
  let roleDetail = $state('');
  let phone = $state('');
  let email = $state('');
  let notes = $state('');
  let dates = $state<string[]>([]);
  let newDate = $state('');

  const shown = $derived(filter === 'all' ? members : members.filter((m) => m.role === filter));
  const roleOptions = Object.entries(ROLE_LABELS) as [MemberRole, string][];

  async function refresh() {
    try {
      members = await listMembers(project.id);
    } catch (e) {
      error = errorMessage(e);
    }
  }
  onMount(refresh);

  function reset() {
    editing = null;
    firstName = lastName = roleDetail = phone = email = notes = newDate = '';
    role = 'acteur';
    dates = [];
  }

  function edit(m: TeamMember) {
    editing = m;
    firstName = m.firstName;
    lastName = m.lastName;
    role = m.role;
    roleDetail = m.roleDetail ?? '';
    phone = m.phone ?? '';
    email = m.email ?? '';
    notes = m.notes;
    dates = [...m.unavailableDates];
    newDate = '';
  }

  function addDate() {
    if (newDate && !dates.includes(newDate)) dates = [...dates, newDate].sort();
    newDate = '';
  }

  function removeDate(d: string) {
    dates = dates.filter((x) => x !== d);
  }

  async function save(event: SubmitEvent) {
    event.preventDefault();
    error = '';
    const common = {
      firstName,
      lastName,
      role,
      roleDetail: roleDetail || null,
      phone: phone || null,
      email: email || null,
      notes,
      unavailableDates: dates,
    };
    try {
      if (editing) {
        await updateMember({ id: editing.id, expectedVersion: editing.version, ...common });
      } else {
        await createMember({ projectId: project.id, ...common });
      }
      reset();
      await refresh();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function remove(m: TeamMember) {
    if (!confirm(`Retirer ${m.firstName} ${m.lastName} de l’équipe ?`)) return;
    error = '';
    try {
      await deleteMember(m.id);
      if (editing?.id === m.id) reset();
      await refresh();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  function roleText(m: TeamMember): string {
    return m.role === 'autre' && m.roleDetail ? m.roleDetail : ROLE_LABELS[m.role];
  }
</script>

<h2>Équipe</h2>
<p class="lead">{members.length} personne(s) sur « {project.title} ».</p>

{#if error}<p class="error" role="alert">{error}</p>{/if}

<form class="pane" onsubmit={save}>
  <h3>{editing ? `Modifier ${editing.firstName} ${editing.lastName}` : 'Ajouter un membre'}</h3>
  <div class="row">
    <input placeholder="Prénom" bind:value={firstName} required />
    <input placeholder="Nom" bind:value={lastName} required />
    <select bind:value={role} aria-label="Rôle">
      {#each roleOptions as [value, label] (value)}<option {value}>{label}</option>{/each}
    </select>
    {#if role === 'autre'}
      <input placeholder="Précisez la fonction" bind:value={roleDetail} />
    {/if}
  </div>
  <div class="row" style="margin-top: 8px">
    <input placeholder="Téléphone" bind:value={phone} />
    <input type="email" placeholder="E-mail" bind:value={email} style="min-width: 220px" />
    <input placeholder="Notes" bind:value={notes} style="flex: 1; min-width: 160px" />
  </div>
  <div class="row" style="margin-top: 8px">
    <label for="unavail">Indisponible le</label>
    <input id="unavail" type="date" bind:value={newDate} />
    <button type="button" onclick={addDate}>Ajouter la date</button>
    {#each dates as d (d)}
      <span class="badge bad">
        {fmtDate(d)}
        <button type="button" class="chip-x" aria-label="Retirer {d}" onclick={() => removeDate(d)}>×</button>
      </span>
    {/each}
  </div>
  <div class="row" style="margin-top: 12px">
    <button class="primary">{editing ? 'Enregistrer' : 'Ajouter à l’équipe'}</button>
    {#if editing}<button type="button" onclick={reset}>Annuler</button>{/if}
  </div>
</form>

<div class="pane">
  <div class="row" style="margin-bottom: 8px">
    <label for="role-filter">Afficher</label>
    <select id="role-filter" bind:value={filter}>
      <option value="all">Tous les rôles</option>
      {#each roleOptions as [value, label] (value)}<option {value}>{label}</option>{/each}
    </select>
  </div>
  {#if shown.length === 0}
    <p class="empty">Aucun membre pour l’instant.</p>
  {:else}
    <table>
      <thead><tr><th>Nom</th><th>Rôle</th><th>Contact</th><th>Indisponible</th><th></th></tr></thead>
      <tbody>
        {#each shown as m (m.id)}
          <tr>
            <td><b>{m.firstName} {m.lastName}</b></td>
            <td><span class="badge">{roleText(m)}</span></td>
            <td>
              {m.phone ?? ''}{#if m.phone && m.email}<br />{/if}{m.email ?? ''}
              {#if !m.phone && !m.email}—{/if}
            </td>
            <td>
              {#each m.unavailableDates as d (d)}<span class="badge bad">{d}</span> {/each}
              {#if m.unavailableDates.length === 0}—{/if}
            </td>
            <td style="text-align: right; white-space: nowrap">
              <button class="link" onclick={() => edit(m)}>Modifier</button>
              <button class="link danger" onclick={() => remove(m)}>Retirer</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<script lang="ts">
  import ProjectsPage from './lib/ProjectsPage.svelte';
  import ProjectPage from './lib/ProjectPage.svelte';
  import TeamPage from './lib/TeamPage.svelte';
  import PlanningPage from './lib/PlanningPage.svelte';
  import DocumentsPage from './lib/DocumentsPage.svelte';
  import type { Project } from './lib/types';

  type Section = 'scenes' | 'team' | 'planning' | 'documents';

  const SECTIONS: { id: Section; label: string }[] = [
    { id: 'scenes', label: 'Scènes' },
    { id: 'team', label: 'Équipe' },
    { id: 'planning', label: 'Planning' },
    { id: 'documents', label: 'Documents' },
  ];

  let selected = $state<Project | null>(null);
  let section = $state<Section>('scenes');

  function open(project: Project) {
    selected = project;
    section = 'scenes';
  }
</script>

<div class="shell">
  <aside>
    <h1>CinePlan</h1>
    <p class="sub">Préparation de tournage</p>
    <nav>
      {#if selected}
        <button class="nav" onclick={() => (selected = null)}>← Tous les projets</button>
        <p class="project-name">{selected.title}</p>
        {#each SECTIONS as s (s.id)}
          <button class="nav" class:on={section === s.id} onclick={() => (section = s.id)}>
            {s.label}
          </button>
        {/each}
      {:else}
        <button class="nav on">Projets</button>
      {/if}
    </nav>
  </aside>

  <main>
    {#if !selected}
      <ProjectsPage onopen={open} />
    {:else}
      {#key `${selected.id}-${section}`}
        {#if section === 'scenes'}
          <ProjectPage
            project={selected}
            onback={() => (selected = null)}
            onupdate={(p) => (selected = p)}
          />
        {:else if section === 'team'}
          <TeamPage project={selected} />
        {:else if section === 'planning'}
          <PlanningPage project={selected} />
        {:else}
          <DocumentsPage project={selected} />
        {/if}
      {/key}
    {/if}
  </main>
</div>

<style>
  .shell { display: grid; grid-template-columns: 220px 1fr; min-height: 100vh; }
  aside { background: var(--side); color: var(--side-ink); padding: 20px 12px; }
  h1 { color: #fff; font-size: 20px; margin: 0 8px 2px; }
  .sub { margin: 0 8px 18px; font-size: 12px; opacity: 0.7; }
  nav { display: flex; flex-direction: column; gap: 2px; }
  .nav { display: block; width: 100%; text-align: left; padding: 8px 10px; border: 0; border-left: 3px solid transparent; border-radius: 6px; background: none; color: inherit; }
  .nav:hover { background: rgba(255, 255, 255, 0.07); }
  .nav.on { background: rgba(255, 255, 255, 0.1); color: #fff; font-weight: 700; border-left-color: var(--acc); }
  .project-name { margin: 14px 10px 6px; font-weight: 700; color: #fff; font-size: 14px; }
  main { padding: 28px 32px; max-width: 1100px; width: 100%; }
</style>

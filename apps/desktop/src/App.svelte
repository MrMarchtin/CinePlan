<script lang="ts">
  import ProjectsPage from './lib/ProjectsPage.svelte';
  import ProjectPage from './lib/ProjectPage.svelte';
  import type { Project } from './lib/types';

  let selected = $state<Project | null>(null);
</script>

<div class="shell">
  <aside>
    <h1>CinePlan</h1>
    <p class="sub">Préparation de tournage</p>
    <nav>
      <button class="nav on" onclick={() => (selected = null)}>Projets</button>
      <span class="nav off">Équipe</span>
      <span class="nav off">Planning</span>
      <span class="nav off">Budget</span>
    </nav>
  </aside>

  <main>
    {#if selected}
      <ProjectPage
        project={selected}
        onback={() => (selected = null)}
        onupdate={(p) => (selected = p)}
      />
    {:else}
      <ProjectsPage onopen={(p) => (selected = p)} />
    {/if}
  </main>
</div>

<style>
  .shell { display: grid; grid-template-columns: 220px 1fr; min-height: 100vh; }
  aside { background: var(--side); color: var(--side-ink); padding: 20px 12px; }
  h1 { color: #fff; font-size: 20px; margin: 0 8px 2px; }
  .sub { margin: 0 8px 18px; font-size: 12px; opacity: 0.7; }
  nav { display: flex; flex-direction: column; gap: 2px; }
  .nav { display: block; text-align: left; padding: 8px 10px; border: 0; border-left: 3px solid transparent; border-radius: 6px; background: none; color: inherit; }
  .nav.on { background: rgba(255, 255, 255, 0.1); color: #fff; font-weight: 700; border-left-color: var(--acc); }
  .nav.off { opacity: 0.4; }
  main { padding: 28px 32px; max-width: 1000px; width: 100%; }
</style>

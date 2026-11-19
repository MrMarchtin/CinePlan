import { invoke } from '@tauri-apps/api/core';
import type {
  NewProject,
  NewScene,
  Project,
  ProjectUpdate,
  Scene,
  SceneUpdate,
} from './types';

// Projets
export const listProjects = () => invoke<Project[]>('list_projects');
export const getProject = (id: string) => invoke<Project>('get_project', { id });
export const createProject = (input: NewProject) => invoke<Project>('create_project', { input });
export const updateProject = (input: ProjectUpdate) => invoke<Project>('update_project', { input });
export const deleteProject = (id: string) => invoke<void>('delete_project', { id });

// Scènes
export const listScenes = (projectId: string) => invoke<Scene[]>('list_scenes', { projectId });
export const createScene = (input: NewScene) => invoke<Scene>('create_scene', { input });
export const updateScene = (input: SceneUpdate) => invoke<Scene>('update_scene', { input });
export const deleteScene = (id: string) => invoke<void>('delete_scene', { id });

/** Extrait un message lisible d'une erreur renvoyée par une commande Tauri. */
export function errorMessage(e: unknown): string {
  if (typeof e === 'object' && e !== null && 'message' in e) {
    return String((e as { message: unknown }).message);
  }
  return String(e);
}

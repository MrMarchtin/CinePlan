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

// ---------- équipe ----------
import type {
  DayItem, DayUpdate, DayView, DocumentFile, DocumentUpdate, Folder, ItemUpdate,
  MemberUpdate, NewDay, NewMember, ShootingDay, TeamMember, DocCategory,
} from './types';

export const listMembers = (projectId: string) => invoke<TeamMember[]>('list_members', { projectId });
export const createMember = (input: NewMember) => invoke<TeamMember>('create_member', { input });
export const updateMember = (input: MemberUpdate) => invoke<TeamMember>('update_member', { input });
export const deleteMember = (id: string) => invoke<void>('delete_member', { id });

// ---------- planning ----------
export const listDays = (projectId: string) => invoke<ShootingDay[]>('list_days', { projectId });
export const getDayView = (id: string) => invoke<DayView>('get_day_view', { id });
export const createDay = (input: NewDay) => invoke<ShootingDay>('create_day', { input });
export const updateDay = (input: DayUpdate) => invoke<ShootingDay>('update_day', { input });
export const deleteDay = (id: string) => invoke<void>('delete_day', { id });
export const addSceneItem = (input: { dayId: string; sceneId: string; durationMin: number | null }) =>
  invoke<DayItem>('add_scene_item', { input });
export const addBreakItem = (input: {
  dayId: string; label: string; durationMin: number; fixedStartMin: number | null;
}) => invoke<DayItem>('add_break_item', { input });
export const updateItem = (input: ItemUpdate) => invoke<DayItem>('update_item', { input });
export const reorderItems = (dayId: string, ordered: string[]) =>
  invoke<void>('reorder_items', { dayId, ordered });
export const removeItem = (id: string) => invoke<void>('remove_item', { id });

// ---------- documents ----------
export const listFolders = (projectId: string) => invoke<Folder[]>('list_folders', { projectId });
export const createFolder = (input: {
  projectId: string; parentId: string | null; category: DocCategory; name: string;
}) => invoke<Folder>('create_folder', { input });
export const renameFolder = (input: { id: string; expectedVersion: number; name: string }) =>
  invoke<Folder>('rename_folder', { input });
export const deleteFolder = (id: string) => invoke<void>('delete_folder', { id });
export const listDocuments = (projectId: string) =>
  invoke<DocumentFile[]>('list_documents', { projectId });
export const importDocument = (input: {
  projectId: string; sourcePath: string; category: DocCategory; folderId: string | null;
  sceneId: string | null; memberId: string | null;
}) => invoke<DocumentFile>('import_document', { input });
export const updateDocument = (input: DocumentUpdate) =>
  invoke<DocumentFile>('update_document', { input });
export const deleteDocument = (id: string) => invoke<void>('delete_document', { id });
export const readDocument = (id: string) => invoke<ArrayBuffer>('read_document', { id });
export const exportDocument = (id: string, dest: string) =>
  invoke<void>('export_document', { id, dest });

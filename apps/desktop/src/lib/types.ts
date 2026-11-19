// Miroir TypeScript des modèles de `cineplan-core` (JSON en camelCase).

export type ProjectStatus = 'preparation' | 'tournage' | 'termine' | 'archive';
export type IntExt = 'int' | 'ext';
export type TimeOfDay = 'jour' | 'nuit' | 'aube' | 'crepuscule';

export interface Project {
  id: string;
  title: string;
  description: string;
  genre: string | null;
  estimatedDurationMin: number | null;
  status: ProjectStatus;
  startDate: string | null;
  endDate: string | null;
  createdAt: string;
  updatedAt: string;
  version: number;
}

export interface NewProject {
  title: string;
  description: string;
  genre: string | null;
  estimatedDurationMin: number | null;
  startDate: string | null;
  endDate: string | null;
}

export interface ProjectUpdate extends NewProject {
  id: string;
  expectedVersion: number;
  status: ProjectStatus;
}

export interface Scene {
  id: string;
  projectId: string;
  number: number;
  title: string;
  intExt: IntExt;
  timeOfDay: TimeOfDay;
  locationName: string | null;
  summary: string;
  estimatedDurationMin: number | null;
  createdAt: string;
  updatedAt: string;
  version: number;
}

export interface NewScene {
  projectId: string;
  number: number;
  title: string;
  intExt: IntExt;
  timeOfDay: TimeOfDay;
  locationName: string | null;
  summary: string;
  estimatedDurationMin: number | null;
}

export interface SceneUpdate extends Omit<NewScene, 'projectId'> {
  id: string;
  expectedVersion: number;
}

export interface CommandError {
  kind: 'validation' | 'not_found' | 'conflict' | 'storage';
  message: string;
}

export const STATUS_LABELS: Record<ProjectStatus, string> = {
  preparation: 'Préparation',
  tournage: 'Tournage',
  termine: 'Terminé',
  archive: 'Archivé',
};

export const INT_EXT_LABELS: Record<IntExt, string> = { int: 'INT.', ext: 'EXT.' };

export const TIME_LABELS: Record<TimeOfDay, string> = {
  jour: 'Jour',
  nuit: 'Nuit',
  aube: 'Aube',
  crepuscule: 'Crépuscule',
};

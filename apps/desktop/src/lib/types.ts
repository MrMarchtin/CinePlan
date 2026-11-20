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

// ---------- équipe ----------

export type MemberRole =
  | 'realisateur' | 'assistant_realisateur' | 'cadreur' | 'chef_operateur'
  | 'ingenieur_son' | 'decorateur' | 'accessoiriste' | 'costumier'
  | 'acteur' | 'figurant' | 'producteur' | 'autre';

export const ROLE_LABELS: Record<MemberRole, string> = {
  realisateur: 'Réalisateur·rice',
  assistant_realisateur: 'Assistant·e réalisateur·rice',
  cadreur: 'Cadreur·se',
  chef_operateur: 'Chef·fe opérateur·rice',
  ingenieur_son: 'Ingénieur·e du son',
  decorateur: 'Décorateur·rice',
  accessoiriste: 'Accessoiriste',
  costumier: 'Costumier·ère',
  acteur: 'Acteur·rice',
  figurant: 'Figurant·e',
  producteur: 'Producteur·rice',
  autre: 'Autre',
};

export interface TeamMember {
  id: string;
  projectId: string;
  firstName: string;
  lastName: string;
  role: MemberRole;
  roleDetail: string | null;
  phone: string | null;
  email: string | null;
  notes: string;
  unavailableDates: string[];
  createdAt: string;
  updatedAt: string;
  version: number;
}

export interface NewMember {
  projectId: string;
  firstName: string;
  lastName: string;
  role: MemberRole;
  roleDetail: string | null;
  phone: string | null;
  email: string | null;
  notes: string;
  unavailableDates: string[];
}

export interface MemberUpdate extends Omit<NewMember, 'projectId'> {
  id: string;
  expectedVersion: number;
}

// ---------- planning ----------

export interface ShootingDay {
  id: string;
  projectId: string;
  date: string;
  startMin: number;
  endMin: number;
  location: string | null;
  notes: string;
  memberIds: string[];
  createdAt: string;
  updatedAt: string;
  version: number;
}

export interface NewDay {
  projectId: string;
  date: string;
  startMin: number;
  endMin: number;
  location: string | null;
  notes: string;
  memberIds: string[];
}

export interface DayUpdate extends Omit<NewDay, 'projectId'> {
  id: string;
  expectedVersion: number;
}

export interface DayItem {
  id: string;
  dayId: string;
  position: number;
  kind: 'scene' | 'break';
  sceneId: string | null;
  label: string;
  durationMin: number;
  delayMin: number;
  fixedStartMin: number | null;
}

export interface ItemUpdate {
  id: string;
  durationMin: number;
  delayMin: number;
  label: string | null;
  fixedStartMin: number | null;
}

export interface Slot {
  itemId: string;
  plannedStart: number;
  plannedEnd: number;
  actualStart: number;
  actualEnd: number;
  shiftMin: number;
  conflict: 'fixed_break_pushed' | 'fixed_break_overlap' | null;
  conflictMin: number;
}

export interface DayView {
  day: ShootingDay;
  items: DayItem[];
  schedule: {
    slots: Slot[];
    summary: { plannedEnd: number; actualEnd: number; dayEnd: number; overtimeMin: number };
  };
  unavailableMemberIds: string[];
}

// ---------- documents ----------

export type DocCategory =
  | 'scenario' | 'storyboard' | 'preparation' | 'planning' | 'decors'
  | 'costumes' | 'materiel' | 'administratif' | 'autres';

export const CATEGORY_LABELS: Record<DocCategory, string> = {
  scenario: 'Scénario',
  storyboard: 'Storyboard',
  preparation: 'Préparation technique',
  planning: 'Planning et feuilles de service',
  decors: 'Décors et lieux',
  costumes: 'Costumes et accessoires',
  materiel: 'Matériel',
  administratif: 'Documents administratifs',
  autres: 'Autres documents',
};

export interface Folder {
  id: string;
  projectId: string;
  parentId: string | null;
  category: DocCategory;
  name: string;
  version: number;
}

export interface DocumentFile {
  id: string;
  projectId: string;
  folderId: string | null;
  category: DocCategory;
  name: string;
  originalName: string;
  kind: 'pdf' | 'image';
  mimeType: string;
  sizeBytes: number;
  sceneId: string | null;
  memberId: string | null;
  createdAt: string;
  version: number;
}

export interface DocumentUpdate {
  id: string;
  expectedVersion: number;
  name: string;
  category: DocCategory;
  folderId: string | null;
  sceneId: string | null;
  memberId: string | null;
}

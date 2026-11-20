# CinePlan

Logiciel de préparation de tournages. Ce dépôt contient le **premier livrable technique** du cahier des charges :
workspace Cargo, application Tauri 2 + Svelte, crate `core` (Projet, Scène), SQLite avec migrations, commandes Tauri.

## Structure

```
cineplan/
├── Cargo.toml                  # workspace
├── migrations/0001_init.sql    # schéma SQLite versionné
├── crates/
│   ├── core/                   # entités, validations, traits de stockage, services (aucune dépendance Tauri/SQL)
│   └── database/               # implémentation SQLite (SQLx) + migrations embarquées + tests
└── apps/desktop/
    ├── src/                    # interface Svelte 5 + TypeScript
    └── src-tauri/              # Tauri 2 : état, commandes, configuration
```

Flux : Svelte → `invoke()` → commande Tauri → service du crate `core` → trait `…Repository` → `SqliteStore`.

## Prérequis

- Rust stable (≥ 1.77) : https://rustup.rs
- Node.js ≥ 20
- Dépendances système de Tauri 2 : https://v2.tauri.app/start/prerequisites/ (Windows : WebView2 et Build Tools C++)

## Lancer

```bash
# 1. Tests Rust (règles métier + accès aux données, base SQLite en mémoire)
cargo test -p cineplan-core -p cineplan-database

# 2. Application
cd apps/desktop
npm install
npm run tauri icon app-icon.png   # une seule fois : génère src-tauri/icons à partir d'une image carrée
npm run tauri dev
```

La base est créée automatiquement au premier lancement dans le dossier de données de l'application
(`cineplan.db`), et les migrations sont appliquées au démarrage.

## Ajouter une migration

Créer `migrations/0002_<description>.sql`. Ne jamais modifier une migration déjà appliquée.

## Conventions

- Identifiants : UUID générés côté client (compatibles hors connexion et synchronisation future).
- `version` incrémentée à chaque modification ; une écriture sur une version périmée renvoie un conflit explicite.
- Suppression d'un projet = corbeille (`deleted_at`), pas d'effacement définitif.

## Distribuer à des utilisateurs non techniques

Les utilisateurs finaux n'installent ni Rust ni Node : ils reçoivent **un seul fichier `.exe`** (voir `INSTALLATION.txt`).

L'installateur est construit automatiquement par GitHub (`.github/workflows/installateur.yml`) :

1. Déposer ce dossier dans un dépôt GitHub (privé possible).
2. Onglet **Actions** > **Installateur Windows** > **Run workflow** : au bout de ~10 minutes, télécharger
   `CinePlan-installateur` (contient `CinePlan_0.2.0_x64-setup.exe`).
3. Pour une version officielle : créer l'étiquette `v0.1.0` ; l'installateur est joint à la page **Releases**,
   avec un lien de téléchargement à partager.

Sur un PC de développement déjà configuré, la commande équivalente est :
`cd apps/desktop && npm run tauri icon app-icon.png && npm run tauri build -- --bundles nsis`
(résultat dans `target/release/bundle/nsis/`).

L'installateur : installation dans le profil de l'utilisateur (sans droits administrateur), en français,
avec le composant WebView2 inclus si Windows ne l'a pas.

## Version 0.2.0 — lot A : équipe, planning, documents

- **Équipe** : fiches (nom, prénom, rôle, contact, notes, jours d'indisponibilité), filtre par rôle.
- **Planning** : calendrier mensuel ; par journée, horaires de début et de fin, scènes ordonnées, pauses,
  convocations. Un retard (ou une avance) saisi sur une scène recalcule les horaires de la suite et la fin de
  journée ; le prévisionnel reste affiché à côté du recalculé. Une pause à **heure fixe** n'est jamais décalée
  d'office : si un retard l'atteint, un conflit est signalé et l'utilisateur décide. Calcul dans
  `crates/core/src/schedule.rs` (testé).
- **Documents** : PDF et images rangés par catégories et dossiers (4 niveaux), renommage, déplacement, lien avec
  une scène, recherche par nom. Les fichiers sont copiés dans le dossier de données de l'application.
  Visionneuse intégrée (pdf.js) : pages, zoom, recherche de texte (liste des pages contenant le mot),
  enregistrement d'une copie.
- Menu : les entrées grisées ont disparu ; seuls les modules réels sont affichés.

Migrations ajoutées : `0002_team.sql`, `0003_planning.sql`, `0004_documents.sql` (les données existantes sont conservées).

Limites connues : la recherche dans un PDF n'affiche pas le mot en surbrillance ; les PDF scannés (images) ne sont pas
recherchables ; les membres sont des fiches, pas encore des comptes (prévu avec le serveur).

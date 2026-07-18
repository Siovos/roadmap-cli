//! CLI definitions

use clap::{Parser, Subcommand};
use crate::commands::GenerateType;

#[derive(Parser)]
#[command(name = "roadmap")]
#[command(version, about = "CLI pour gérer les roadmaps projet")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialiser une roadmap dans le projet courant
    Init,
    /// Ajouter une nouvelle phase
    Add {
        /// ID de la phase (ex: 9, 19a)
        id: String,
        /// Nom de la phase
        name: String,
        /// Phase parente (pour les sous-phases)
        #[arg(long)]
        parent: Option<String>,
        /// Dépendances (IDs de phases pré-requises)
        #[arg(long)]
        depends_on: Option<Vec<String>>,
    },
    /// Modifier une phase
    Edit {
        /// ID de la phase
        id: String,
        /// Nouveau nom
        #[arg(long)]
        name: Option<String>,
        /// Nouvelle description
        #[arg(long)]
        description: Option<String>,
        /// Ajouter une dépendance (ID de phase pré-requise)
        #[arg(long)]
        depends_on: Option<Vec<String>>,
    },
    /// Lister toutes les phases
    List {
        /// Afficher en tableau formaté
        #[arg(long)]
        table: bool,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
        /// Filtrer par tag (tâches ayant ce tag)
        #[arg(long)]
        tag: Option<String>,
        /// Filtrer par statut (pending, in_progress, done, blocked)
        #[arg(long)]
        status: Option<String>,
        /// Filtrer par assignee
        #[arg(long)]
        assignee: Option<String>,
        /// Afficher les tâches avec échéance dépassée
        #[arg(long)]
        overdue: bool,
    },
    /// Afficher l'arborescence complète
    Tree {
        /// Sortie JSON
        #[arg(long)]
        json: bool,
        /// Masquer les phases et tâches terminées
        #[arg(long)]
        hide_done: bool,
    },
    /// Afficher les détails d'une phase
    Show {
        /// ID de la phase
        id: String,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Gérer les tâches
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },
    /// Gérer les bugs / incidents
    Bug {
        #[command(subcommand)]
        action: BugCommands,
    },
    /// Gérer les demandes de fonctionnalités
    Feature {
        #[command(subcommand)]
        action: FeatureCommands,
    },
    /// Changer la priorité d'une phase
    Priority {
        /// ID de la phase
        id: String,
        /// Nouvelle priorité (1 = haute)
        #[arg(long)]
        set: u32,
    },
    /// Changer le statut d'une phase
    Status {
        /// ID de la phase
        id: String,
        /// Nouveau statut (pending, in_progress, done, blocked)
        #[arg(long)]
        set: String,
    },
    /// Ajouter une note à une phase
    Note {
        /// ID de la phase
        id: String,
        /// Contenu de la note
        content: String,
    },
    /// Générer les fichiers Markdown
    Export,
    /// Rapport de progression (vue planning)
    Report {
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Gérer le workflow d'une tâche
    Workflow {
        /// ID de la tâche
        task_id: String,
        /// Avancer à l'étape suivante
        #[arg(long)]
        advance: bool,
        /// Forcer une étape spécifique
        #[arg(long)]
        set: Option<String>,
    },
    /// Interface interactive (TUI)
    Ui,
    /// Lancer le serveur web
    Serve {
        /// Port du serveur (défaut: 7878)
        #[arg(long, short, default_value = "7878")]
        port: u16,
        /// Ouvrir le navigateur automatiquement
        #[arg(long)]
        open: bool,
    },
    /// Mettre à jour roadmap-cli
    Update {
        /// Vérifier seulement, sans installer
        #[arg(long)]
        check: bool,
    },
    /// Afficher la prochaine tâche à faire
    Next {
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Contexte complet pour LLM (optimisé pour AI assistants)
    Context {
        /// Inclure les tâches terminées
        #[arg(long)]
        include_done: bool,
        /// Filtrer sur une phase spécifique
        #[arg(long)]
        phase: Option<String>,
    },
    /// Rechercher un mot-clé dans les phases et tâches
    Search {
        /// Terme à rechercher
        query: String,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Vérifier la cohérence entre le code et la roadmap
    Sync {
        /// Pattern glob pour les fichiers source (défaut: **/*.rs)
        #[arg(long, default_value = "**/*.rs")]
        glob: String,
        /// Corriger automatiquement (supprimer refs cassées)
        #[arg(long)]
        fix: bool,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Scanner le code pour trouver les TODO/FIXME
    Scan {
        /// Pattern glob pour filtrer les fichiers (défaut: **/*.rs)
        #[arg(long, default_value = "**/*.rs")]
        glob: String,
        /// Créer des tâches automatiquement
        #[arg(long)]
        create: bool,
        /// Phase cible pour les tâches créées
        #[arg(long)]
        phase: Option<String>,
        /// Inclure les répertoires cachés
        #[arg(long)]
        hidden: bool,
    },
    /// Analyser la couverture API (routes backend vs appels frontend)
    Coverage {
        /// Pattern glob pour les fichiers backend (ex: src/api/**/*.ts)
        #[arg(long, short)]
        backend: String,
        /// Pattern glob pour les fichiers frontend (ex: src/app/**/*.tsx)
        #[arg(long, short)]
        frontend: String,
        /// Pattern glob pour la couche BFF/intermédiaire (ex: src/app/api/**/route.ts)
        #[arg(long)]
        bff: Option<String>,
        /// Préfixe des routes API (défaut: /api)
        #[arg(long, default_value = "/api")]
        prefix: String,
        /// Préfixe des routes backend (pour BFF, défaut: même que prefix)
        #[arg(long)]
        backend_prefix: Option<String>,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Générer un changelog depuis les commits git
    Changelog {
        /// Nombre de commits à inclure (défaut: 50)
        #[arg(long, default_value = "50")]
        limit: usize,
        /// Tag de départ (ex: v0.1.0)
        #[arg(long)]
        from: Option<String>,
        /// Tag de fin (défaut: HEAD)
        #[arg(long)]
        to: Option<String>,
        /// Format de sortie (markdown, json)
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    /// Gérer les git hooks (auto-export)
    Hooks {
        /// Installer le hook pre-commit
        #[arg(long)]
        install: bool,
        /// Désinstaller le hook
        #[arg(long)]
        uninstall: bool,
    },
    /// Créer une phase depuis un template prédéfini
    Template {
        /// Nom du template (feature, bug, api, infra, release, onboarding)
        #[arg(default_value = "")]
        name: String,
        /// ID de la phase à créer
        #[arg(default_value = "")]
        phase_id: String,
        /// Lister les templates disponibles
        #[arg(long)]
        list: bool,
    },
    /// Historique des changements (tâches terminées, etc.)
    Log {
        /// Nombre d'entrées max (défaut: 20)
        #[arg(long, short, default_value = "20")]
        limit: usize,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Se connecter à l'app web (ouvre le navigateur)
    Login {
        /// URL du serveur (défaut: http://localhost:7878)
        #[arg(long, default_value = "http://localhost:7878")]
        server: String,
    },
    /// Se déconnecter
    Logout,
    /// Pousser les données locales vers la base de données
    Push {
        /// Nom du projet (défaut: nom du dossier)
        #[arg(long)]
        project: Option<String>,
    },
    /// Vérifier l'intégrité des fichiers YAML de la roadmap
    Doctor,
    /// Générer les fichiers man et completions shell
    Generate {
        /// Type: man, completions, all
        #[arg(value_enum)]
        what: GenerateType,
        /// Répertoire de sortie (défaut: ./generated)
        #[arg(long, short, default_value = "generated")]
        output: String,
    },
}

#[derive(Subcommand)]
pub enum TaskCommands {
    /// Ajouter une tâche à une phase
    Add {
        /// ID de la phase
        phase_id: String,
        /// Nom de la tâche
        name: String,
        /// Description détaillée
        #[arg(long, alias = "desc")]
        description: Option<String>,
        /// Tâche parente (pour les sous-tâches)
        #[arg(long)]
        parent: Option<String>,
        /// Marquer comme optionnelle
        #[arg(long)]
        optional: bool,
        /// Fichiers liés à cette tâche
        #[arg(long, value_delimiter = ',')]
        files: Option<Vec<String>>,
        /// Tags/labels pour catégoriser la tâche
        #[arg(long, value_delimiter = ',')]
        tag: Option<Vec<String>>,
        /// Personne assignée
        #[arg(long)]
        assignee: Option<String>,
        /// Date d'échéance (YYYY-MM-DD)
        #[arg(long)]
        due: Option<String>,
    },
    /// Marquer une ou plusieurs tâches comme terminées
    Done {
        /// IDs des tâches (ex: 9.1 9.2 9.3)
        task_ids: Vec<String>,
    },
    /// Marquer une ou plusieurs tâches comme en cours
    Start {
        /// IDs des tâches (ex: 9.1 9.2)
        task_ids: Vec<String>,
    },
    /// Modifier une tâche
    Edit {
        /// ID de la tâche
        task_id: String,
        /// Nouveau nom
        #[arg(long)]
        name: Option<String>,
        /// Nouvelle description
        #[arg(long, alias = "desc")]
        description: Option<String>,
        /// Marquer comme optionnelle
        #[arg(long)]
        optional: Option<bool>,
        /// Fichiers liés à cette tâche (remplace les existants)
        #[arg(long, value_delimiter = ',')]
        files: Option<Vec<String>>,
        /// Tags/labels (remplace les existants)
        #[arg(long, value_delimiter = ',')]
        tag: Option<Vec<String>>,
        /// Personne assignée
        #[arg(long)]
        assignee: Option<String>,
        /// Date d'échéance (YYYY-MM-DD)
        #[arg(long)]
        due: Option<String>,
    },
    /// Déplacer une tâche vers une autre phase
    Move {
        /// ID de la tâche
        task_id: String,
        /// Phase de destination
        #[arg(long = "to")]
        to_phase: String,
    },
    /// Définir qu'une tâche en bloque une autre
    Blocks {
        /// ID de la tâche bloquante
        task_id: String,
        /// ID de la tâche bloquée
        blocked_id: String,
    },
    /// Retirer une dépendance
    Unblocks {
        /// ID de la tâche bloquante
        task_id: String,
        /// ID de la tâche à débloquer
        blocked_id: String,
    },
}

#[derive(Subcommand)]
pub enum BugCommands {
    /// Déclarer un nouveau bug
    Add {
        /// Titre du bug
        title: String,
        /// Sévérité (blocking, major, minor)
        #[arg(long, default_value = "major")]
        severity: String,
        /// Phase concernée
        #[arg(long)]
        phase: Option<String>,
        /// Description / étapes de reproduction
        #[arg(long, alias = "desc")]
        description: Option<String>,
        /// Équipe/personne assignée
        #[arg(long)]
        assignee: Option<String>,
        /// Rapporté par
        #[arg(long)]
        reported_by: Option<String>,
        /// Projet cible (chemin vers un autre projet roadmap)
        #[arg(long)]
        target: Option<String>,
    },
    /// Lister les bugs
    List {
        /// Filtrer par sévérité
        #[arg(long)]
        severity: Option<String>,
        /// Filtrer par statut (open, in_progress, resolved, wontfix)
        #[arg(long)]
        status: Option<String>,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Afficher le détail d'un bug
    Show {
        /// ID du bug
        id: u32,
    },
    /// Marquer un bug comme résolu
    Resolve {
        /// ID du bug
        id: u32,
        /// Description de la résolution
        #[arg(long, alias = "desc")]
        description: Option<String>,
        /// Commit de résolution
        #[arg(long)]
        commit: Option<String>,
    },
    /// Modifier un bug
    Update {
        /// ID du bug
        id: u32,
        /// Nouveau titre
        #[arg(long)]
        title: Option<String>,
        /// Nouvelle description
        #[arg(long, alias = "desc")]
        description: Option<String>,
        /// Phase concernée
        #[arg(long)]
        phase: Option<String>,
        /// Nouveau statut
        #[arg(long)]
        status: Option<String>,
        /// Nouvelle sévérité
        #[arg(long)]
        severity: Option<String>,
        /// Nouvelle assignation
        #[arg(long)]
        assignee: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum FeatureCommands {
    /// Proposer une nouvelle feature
    Add {
        /// Titre de la feature
        title: String,
        /// Priorité (critical, high, medium, low)
        #[arg(long, default_value = "medium")]
        priority: String,
        /// Phase concernée
        #[arg(long)]
        phase: Option<String>,
        /// Description détaillée
        #[arg(long, alias = "desc")]
        description: Option<String>,
        /// Personne assignée
        #[arg(long)]
        assignee: Option<String>,
        /// Demandée par
        #[arg(long)]
        requested_by: Option<String>,
        /// Projet cible (chemin vers un autre projet)
        #[arg(long)]
        target: Option<String>,
    },
    /// Lister les feature requests
    List {
        /// Filtrer par priorité
        #[arg(long)]
        priority: Option<String>,
        /// Filtrer par statut (proposed, accepted, in_progress, implemented, rejected)
        #[arg(long)]
        status: Option<String>,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Afficher le détail d'une feature
    Show {
        /// ID de la feature
        id: u32,
    },
    /// Marquer une feature comme implémentée
    Implement {
        /// ID de la feature
        id: u32,
        /// Description de l'implémentation
        #[arg(long, alias = "desc")]
        description: Option<String>,
        /// Commit
        #[arg(long)]
        commit: Option<String>,
    },
    /// Modifier une feature
    Update {
        /// ID de la feature
        id: u32,
        /// Nouveau titre
        #[arg(long)]
        title: Option<String>,
        /// Nouvelle description
        #[arg(long, alias = "desc")]
        description: Option<String>,
        /// Phase concernée
        #[arg(long)]
        phase: Option<String>,
        /// Nouveau statut
        #[arg(long)]
        status: Option<String>,
        /// Nouvelle priorité
        #[arg(long)]
        priority: Option<String>,
        /// Nouvelle assignation
        #[arg(long)]
        assignee: Option<String>,
    },
}

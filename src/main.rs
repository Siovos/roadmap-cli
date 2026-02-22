mod config;
mod phase;
mod tui;

use std::fs;
use std::path::Path;
use clap::{Parser, Subcommand};
use colored::Colorize;
use phase::Task;
use tabled::{Table, Tabled, settings::Style};

#[derive(Parser)]
#[command(name = "roadmap")]
#[command(version, about = "CLI pour gérer les roadmaps projet")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    },
    /// Lister toutes les phases
    List {
        /// Afficher en tableau formaté
        #[arg(long)]
        table: bool,
        /// Sortie JSON
        #[arg(long)]
        json: bool,
    },
    /// Afficher l'arborescence complète
    Tree {
        /// Sortie JSON
        #[arg(long)]
        json: bool,
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
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Ajouter une tâche à une phase
    Add {
        /// ID de la phase
        phase_id: String,
        /// Nom de la tâche
        name: String,
        /// Tâche parente (pour les sous-tâches)
        #[arg(long)]
        parent: Option<String>,
        /// Marquer comme optionnelle
        #[arg(long)]
        optional: bool,
    },
    /// Marquer une tâche comme terminée
    Done {
        /// ID de la tâche (ex: 9.1, 9.2.1)
        task_id: String,
    },
    /// Marquer une tâche comme en cours
    Start {
        /// ID de la tâche
        task_id: String,
    },
    /// Modifier une tâche
    Edit {
        /// ID de la tâche
        task_id: String,
        /// Nouveau nom
        #[arg(long)]
        name: Option<String>,
        /// Nouvelle description
        #[arg(long)]
        description: Option<String>,
        /// Marquer comme optionnelle
        #[arg(long)]
        optional: Option<bool>,
    },
    /// Déplacer une tâche vers une autre phase
    Move {
        /// ID de la tâche
        task_id: String,
        /// Phase de destination
        #[arg(long = "to")]
        to_phase: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Add { id, name, parent } => cmd_add(id, name, parent),
        Commands::Edit { id, name, description } => cmd_edit(id, name, description),
        Commands::List { table, json } => cmd_list(table, json),
        Commands::Tree { json } => cmd_tree(json),
        Commands::Show { id, json } => cmd_show(id, json),
        Commands::Task { action } => match action {
            TaskCommands::Add { phase_id, name, parent, optional } => {
                cmd_task_add(phase_id, name, parent, optional)
            }
            TaskCommands::Done { task_id } => cmd_task_done(task_id),
            TaskCommands::Start { task_id } => cmd_task_start(task_id),
            TaskCommands::Edit { task_id, name, description, optional } => {
                cmd_task_edit(task_id, name, description, optional)
            }
            TaskCommands::Move { task_id, to_phase } => cmd_task_move(task_id, to_phase),
        },
        Commands::Priority { id, set } => cmd_priority(id, set),
        Commands::Status { id, set } => cmd_status(id, set),
        Commands::Note { id, content } => cmd_note(id, content),
        Commands::Export => cmd_export(),
        Commands::Report { json } => cmd_report(json),
        Commands::Workflow { task_id, advance, set } => cmd_workflow(task_id, advance, set),
        Commands::Ui => cmd_ui(),
    }
}

fn cmd_ui() {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let entries = match fs::read_dir(phases_dir) {
        Ok(entries) => entries,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phases: Vec<phase::Phase> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        if !filename.starts_with("phase-") || !filename.ends_with(".yml") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let p: phase::Phase = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };

        phases.push(p);
    }

    phases.sort_by(|a, b| a.priority.cmp(&b.priority));

    if let Err(e) = tui::run_tui(phases) {
        println!("{} {}", "Erreur TUI:".red(), e);
    }
}

// Struct pour l'affichage en tableau
#[derive(Tabled)]
struct PhaseRow {
    #[tabled(rename = "")]
    status: String,
    #[tabled(rename = "Phase")]
    id: String,
    #[tabled(rename = "Nom")]
    name: String,
    #[tabled(rename = "P")]
    priority: u32,
    #[tabled(rename = "Progression")]
    progress: String,
}

fn cmd_init() {
    let phases_dir = Path::new(".phases");

    if phases_dir.exists() {
        println!("{}", "Erreur: .phases/ existe déjà".red());
        return;
    }

    if let Err(e) = fs::create_dir(phases_dir) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    let config = config::Config::default();
    let yaml = serde_yaml::to_string(&config).expect("Erreur sérialisation");

    let config_path = phases_dir.join("config.yml");
    if let Err(e) = fs::write(&config_path, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!("{}", "✓ Roadmap initialisée !".green());
    println!("  Créé: {}", ".phases/config.yml".cyan());
    println!(
        "\n  Prochaine étape: {} pour créer une phase",
        "roadmap add <id> <nom>".yellow()
    );
}

fn cmd_add(id: String, name: String, parent: Option<String>) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let phase_file = phases_dir.join(format!("phase-{}.yml", id));
    if phase_file.exists() {
        println!(
            "{} La phase {} existe déjà",
            "Erreur:".red(),
            id.yellow()
        );
        return;
    }

    if let Some(ref parent_id) = parent {
        let parent_file = phases_dir.join(format!("phase-{}.yml", parent_id));
        if !parent_file.exists() {
            println!(
                "{} La phase parente {} n'existe pas",
                "Erreur:".red(),
                parent_id.yellow()
            );
            return;
        }
    }

    let mut phase = phase::Phase::new(id.clone(), name.clone());
    phase.parent = parent.clone();

    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    if let Some(parent_id) = parent {
        println!(
            "{} Phase {} créée (sous-phase de {})",
            "✓".green(),
            id.cyan(),
            parent_id.cyan()
        );
    } else {
        println!("{} Phase {} créée", "✓".green(), id.cyan());
    }
    println!("  Fichier: {}", format!(".phases/phase-{}.yml", id).cyan());
}

fn load_phases() -> Option<Vec<phase::Phase>> {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return None;
    }

    let entries = match fs::read_dir(phases_dir) {
        Ok(entries) => entries,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return None;
        }
    };

    let mut phases: Vec<phase::Phase> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        if !filename.starts_with("phase-") || !filename.ends_with(".yml") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let p: phase::Phase = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };

        phases.push(p);
    }

    phases.sort_by(|a, b| a.priority.cmp(&b.priority));
    Some(phases)
}

fn get_status_icon(status: &str) -> &'static str {
    match status {
        "done" => "✅",
        "in_progress" => "🔄",
        "blocked" => "🚫",
        _ => "⬜",
    }
}

fn cmd_list(table: bool, json: bool) {
    let phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    if phases.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("Aucune phase trouvée.");
            println!(
                "Crée une phase avec: {}",
                "roadmap add <id> <nom>".yellow()
            );
        }
        return;
    }

    if json {
        // Sortie JSON
        let output = serde_json::to_string_pretty(&phases).expect("Erreur sérialisation JSON");
        println!("{}", output);
    } else if table {
        // Affichage tableau
        let rows: Vec<PhaseRow> = phases
            .iter()
            .map(|phase| {
                let total = phase.tasks.len();
                let done = phase.tasks.iter().filter(|t| t.status == "done").count();
                let progress = if total > 0 {
                    format!("{}/{}", done, total)
                } else {
                    "-".to_string()
                };

                PhaseRow {
                    status: get_status_icon(&phase.status).to_string(),
                    id: phase.id.clone(),
                    name: phase.name.clone(),
                    priority: phase.priority,
                    progress,
                }
            })
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{}", table);
    } else {
        // Affichage classique
        println!("{}", "Phases:".bold());
        println!();

        for phase in phases {
            let status_icon = get_status_icon(&phase.status);

            let parent_info = match phase.parent {
                Some(ref p) => format!(" (sous-phase de {})", p),
                None => String::new(),
            };

            println!(
                "  {} {} {} - {}{}",
                status_icon,
                format!("[{}]", phase.priority).dimmed(),
                phase.id.cyan(),
                phase.name,
                parent_info.dimmed()
            );

            let total = phase.tasks.len();
            let done = phase.tasks.iter().filter(|t| t.status == "done").count();
            if total > 0 {
                println!("      {} tâches ({} terminées)", total, done);
            }
        }
    }
}

fn cmd_tree(json: bool) {
    let phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    if phases.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("Aucune phase trouvée.");
        }
        return;
    }

    if json {
        // Structure arborescente JSON
        #[derive(serde::Serialize)]
        struct TreeNode {
            id: String,
            name: String,
            #[serde(rename = "type")]
            node_type: String,
            status: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            priority: Option<u32>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            children: Vec<TreeNode>,
        }

        fn build_task_tree(tasks: &[Task], parent: Option<&str>) -> Vec<TreeNode> {
            tasks
                .iter()
                .filter(|t| t.parent.as_deref() == parent)
                .map(|t| TreeNode {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    node_type: if t.optional { "task_optional".to_string() } else { "task".to_string() },
                    status: t.status.clone(),
                    priority: None,
                    children: build_task_tree(tasks, Some(&t.id)),
                })
                .collect()
        }

        let main_phases: Vec<_> = phases.iter().filter(|p| p.parent.is_none()).collect();
        let sub_phases: Vec<_> = phases.iter().filter(|p| p.parent.is_some()).collect();

        let tree: Vec<TreeNode> = main_phases
            .iter()
            .map(|phase| {
                let mut children: Vec<TreeNode> = sub_phases
                    .iter()
                    .filter(|sp| sp.parent.as_ref() == Some(&phase.id))
                    .map(|sp| TreeNode {
                        id: sp.id.clone(),
                        name: sp.name.clone(),
                        node_type: "subphase".to_string(),
                        status: sp.status.clone(),
                        priority: Some(sp.priority),
                        children: build_task_tree(&sp.tasks, None),
                    })
                    .collect();

                children.extend(build_task_tree(&phase.tasks, None));

                TreeNode {
                    id: phase.id.clone(),
                    name: phase.name.clone(),
                    node_type: "phase".to_string(),
                    status: phase.status.clone(),
                    priority: Some(phase.priority),
                    children,
                }
            })
            .collect();

        let output = serde_json::to_string_pretty(&tree).expect("Erreur sérialisation JSON");
        println!("{}", output);
    } else {
        // Affichage texte
        println!("{}", "📋 Roadmap".bold());
        println!();

        let main_phases: Vec<_> = phases.iter().filter(|p| p.parent.is_none()).collect();
        let sub_phases: Vec<_> = phases.iter().filter(|p| p.parent.is_some()).collect();

        for (i, phase) in main_phases.iter().enumerate() {
            let is_last_phase = i == main_phases.len() - 1 && sub_phases.iter().filter(|sp| sp.parent.as_ref() == Some(&phase.id)).count() == 0;
            let prefix = if is_last_phase { "└── " } else { "├── " };
            let status_icon = get_status_icon(&phase.status);

            println!(
                "{}{} Phase {} — {} (P{})",
                prefix, status_icon, phase.id, phase.name, phase.priority
            );

            // Sous-phases de cette phase
            let children: Vec<_> = sub_phases.iter().filter(|sp| sp.parent.as_ref() == Some(&phase.id)).collect();
            for (j, sub) in children.iter().enumerate() {
                let sub_prefix = if is_last_phase { "    " } else { "│   " };
                let sub_branch = if j == children.len() - 1 { "└── " } else { "├── " };
                let sub_icon = get_status_icon(&sub.status);

                println!(
                    "{}{}{} Phase {} — {}",
                    sub_prefix, sub_branch, sub_icon, sub.id, sub.name
                );
            }

            // Tâches de cette phase
            let task_prefix = if is_last_phase { "    " } else { "│   " };
            print_tree_tasks(&phase.tasks, None, task_prefix, &[]);
        }
    }
}

fn print_tree_tasks(tasks: &[Task], parent: Option<&str>, prefix: &str, parent_is_last: &[bool]) {
    let filtered: Vec<_> = tasks.iter().filter(|t| t.parent.as_deref() == parent).collect();

    for (i, task) in filtered.iter().enumerate() {
        let is_last = i == filtered.len() - 1;
        let branch = if is_last { "└── " } else { "├── " };
        let status_icon = get_status_icon(&task.status);
        let optional = if task.optional { " (opt)" } else { "" };

        println!(
            "{}{}{} {} — {}{}",
            prefix, branch, status_icon, task.id, task.name, optional
        );

        // Sous-tâches
        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        let mut new_is_last = parent_is_last.to_vec();
        new_is_last.push(is_last);
        print_tree_tasks(tasks, Some(&task.id), &child_prefix, &new_is_last);
    }
}

fn cmd_show(id: String, json: bool) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        if json {
            println!(r#"{{"error": "Roadmap non initialisée"}}"#);
        } else {
            println!(
                "{} Roadmap non initialisée. Lance d'abord: {}",
                "Erreur:".red(),
                "roadmap init".yellow()
            );
        }
        return;
    }

    let phase_file = phases_dir.join(format!("phase-{}.yml", id));
    if !phase_file.exists() {
        if json {
            println!(r#"{{"error": "Phase non trouvée", "id": "{}"}}"#, id);
        } else {
            println!("{} Phase {} non trouvée", "Erreur:".red(), id.yellow());
        }
        return;
    }

    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!(r#"{{"error": "{}"}}"#, e);
            } else {
                println!("{} {}", "Erreur:".red(), e);
            }
            return;
        }
    };

    let phase: phase::Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            if json {
                println!(r#"{{"error": "YAML invalide: {}"}}"#, e);
            } else {
                println!("{} YAML invalide: {}", "Erreur:".red(), e);
            }
            return;
        }
    };

    if json {
        let output = serde_json::to_string_pretty(&phase).expect("Erreur sérialisation JSON");
        println!("{}", output);
        return;
    }

    let status_icon = match phase.status.as_str() {
        "done" => "✅",
        "in_progress" => "🔄",
        "blocked" => "🚫",
        _ => "⬜",
    };

    println!();
    println!(
        "{} {} - {}",
        status_icon,
        format!("Phase {}", phase.id).cyan().bold(),
        phase.name.bold()
    );

    if !phase.description.is_empty() {
        println!("  {}", phase.description.dimmed());
    }

    println!();
    println!("  Priorité:   {}", phase.priority);
    println!("  Statut:     {}", phase.status);
    if let Some(ref parent) = phase.parent {
        println!("  Parent:     {}", parent);
    }
    println!("  Créée le:   {}", phase.created_at);
    println!("  Modifiée:   {}", phase.updated_at);

    if !phase.tasks.is_empty() {
        println!();
        println!("  {}:", "Tâches".bold());
        print_tasks(&phase.tasks, None, 4);
    }

    if !phase.notes.is_empty() {
        println!();
        println!("  {}:", "Notes".bold());
        for note in &phase.notes {
            println!("    {} - {}", note.date.dimmed(), note.content);
        }
    }

    println!();
}

/// Affiche les tâches avec indentation pour les sous-tâches
fn print_tasks(tasks: &[Task], parent: Option<&str>, indent: usize) {
    let spaces = " ".repeat(indent);

    for task in tasks {
        // Filtre par parent
        let task_parent = task.parent.as_deref();
        if task_parent != parent {
            continue;
        }

        let task_icon = match task.status.as_str() {
            "done" => "✅",
            "in_progress" => "🔄",
            "blocked" => "🚫",
            _ => "⬜",
        };

        let optional_tag = if task.optional {
            " (optionnel)".dimmed().to_string()
        } else {
            String::new()
        };

        let stage_info = match &task.workflow_stage {
            Some(stage) => format!(" [{}]", stage).dimmed().to_string(),
            None => String::new(),
        };

        println!(
            "{}{} {} - {}{}{}",
            spaces, task_icon, task.id.cyan(), task.name, stage_info, optional_tag
        );

        // Affiche les sous-tâches récursivement
        print_tasks(tasks, Some(&task.id), indent + 2);
    }
}

fn cmd_task_add(phase_id: String, name: String, parent: Option<String>, optional: bool) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let phase_file = phases_dir.join(format!("phase-{}.yml", phase_id));
    if !phase_file.exists() {
        println!("{} Phase {} non trouvée", "Erreur:".red(), phase_id.yellow());
        return;
    }

    // Lit la phase
    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phase: phase::Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    // Génère l'ID de la tâche
    let task_id = if let Some(ref parent_id) = parent {
        // Vérifie que la tâche parente existe
        if !phase.tasks.iter().any(|t| t.id == *parent_id) {
            println!(
                "{} Tâche parente {} non trouvée",
                "Erreur:".red(),
                parent_id.yellow()
            );
            return;
        }
        // Compte les sous-tâches de ce parent
        let count = phase.tasks.iter().filter(|t| t.parent.as_ref() == Some(parent_id)).count();
        format!("{}.{}", parent_id, count + 1)
    } else {
        // Compte les tâches racines
        let count = phase.tasks.iter().filter(|t| t.parent.is_none()).count();
        format!("{}.{}", phase_id, count + 1)
    };

    // Crée la tâche
    let task = Task {
        id: task_id.clone(),
        name: name.clone(),
        description: None,
        status: String::from("pending"),
        parent,
        workflow_stage: None,
        optional,
        completed_at: None,
    };

    phase.tasks.push(task);

    // Met à jour la date de modification
    phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Sauvegarde
    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!(
        "{} Tâche {} ajoutée à la phase {}",
        "✓".green(),
        task_id.cyan(),
        phase_id.cyan()
    );
}

fn cmd_task_done(task_id: String) {
    update_task_status(&task_id, "done");
}

fn cmd_task_start(task_id: String) {
    update_task_status(&task_id, "in_progress");
}

fn update_task_status(task_id: &str, new_status: &str) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    // Extrait l'ID de la phase depuis l'ID de la tâche (ex: "9.1" -> "9")
    let phase_id = task_id.split('.').next().unwrap();
    let phase_file = phases_dir.join(format!("phase-{}.yml", phase_id));

    if !phase_file.exists() {
        println!("{} Phase {} non trouvée", "Erreur:".red(), phase_id.yellow());
        return;
    }

    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phase: phase::Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    // Trouve et met à jour la tâche
    let mut found = false;
    for task in &mut phase.tasks {
        if task.id == task_id {
            task.status = new_status.to_string();
            if new_status == "done" {
                task.completed_at = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
            }
            found = true;
            break;
        }
    }

    if !found {
        println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
        return;
    }

    // Met à jour la date de modification
    phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Sauvegarde
    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    let status_icon = match new_status {
        "done" => "✅",
        "in_progress" => "🔄",
        _ => "⬜",
    };

    println!(
        "{} Tâche {} marquée comme {}",
        status_icon,
        task_id.cyan(),
        new_status.green()
    );
}

fn cmd_priority(id: String, priority: u32) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let phase_file = phases_dir.join(format!("phase-{}.yml", id));
    if !phase_file.exists() {
        println!("{} Phase {} non trouvée", "Erreur:".red(), id.yellow());
        return;
    }

    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phase: phase::Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    let old_priority = phase.priority;
    phase.priority = priority;
    phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();

    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!(
        "{} Phase {} : priorité {} → {}",
        "✓".green(),
        id.cyan(),
        old_priority.to_string().dimmed(),
        priority.to_string().green()
    );
}

fn cmd_status(id: String, status: String) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    // Valide le statut
    let valid_statuses = ["pending", "in_progress", "done", "blocked"];
    if !valid_statuses.contains(&status.as_str()) {
        println!(
            "{} Statut invalide. Valeurs possibles: {}",
            "Erreur:".red(),
            valid_statuses.join(", ").yellow()
        );
        return;
    }

    let phase_file = phases_dir.join(format!("phase-{}.yml", id));
    if !phase_file.exists() {
        println!("{} Phase {} non trouvée", "Erreur:".red(), id.yellow());
        return;
    }

    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phase: phase::Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    let old_status = phase.status.clone();
    phase.status = status.clone();
    phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();

    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    let status_icon = match status.as_str() {
        "done" => "✅",
        "in_progress" => "🔄",
        "blocked" => "🚫",
        _ => "⬜",
    };

    println!(
        "{} Phase {} : {} → {}",
        status_icon,
        id.cyan(),
        old_status.dimmed(),
        status.green()
    );
}

fn cmd_note(id: String, content: String) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let phase_file = phases_dir.join(format!("phase-{}.yml", id));
    if !phase_file.exists() {
        println!("{} Phase {} non trouvée", "Erreur:".red(), id.yellow());
        return;
    }

    let file_content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phase: phase::Phase = match serde_yaml::from_str(&file_content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    let note = phase::Note {
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        content,
    };

    phase.notes.push(note);
    phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();

    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!("{} Note ajoutée à la phase {}", "✓".green(), id.cyan());
}

fn cmd_edit(id: String, name: Option<String>, description: Option<String>) {
    if name.is_none() && description.is_none() {
        println!(
            "{} Spécifie au moins --name ou --description",
            "Erreur:".red()
        );
        return;
    }

    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let phase_file = phases_dir.join(format!("phase-{}.yml", id));
    if !phase_file.exists() {
        println!("{} Phase {} non trouvée", "Erreur:".red(), id.yellow());
        return;
    }

    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phase: phase::Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut changes = Vec::new();

    if let Some(new_name) = name {
        changes.push(format!("nom: {}", new_name));
        phase.name = new_name;
    }

    if let Some(new_desc) = description {
        changes.push(format!("description: {}", new_desc));
        phase.description = new_desc;
    }

    phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();

    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!(
        "{} Phase {} modifiée: {}",
        "✓".green(),
        id.cyan(),
        changes.join(", ")
    );
}

fn cmd_task_edit(
    task_id: String,
    name: Option<String>,
    description: Option<String>,
    optional: Option<bool>,
) {
    if name.is_none() && description.is_none() && optional.is_none() {
        println!(
            "{} Spécifie au moins --name, --description ou --optional",
            "Erreur:".red()
        );
        return;
    }

    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let phase_id = task_id.split('.').next().unwrap();
    let phase_file = phases_dir.join(format!("phase-{}.yml", phase_id));

    if !phase_file.exists() {
        println!("{} Phase {} non trouvée", "Erreur:".red(), phase_id.yellow());
        return;
    }

    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phase: phase::Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut found = false;
    let mut changes = Vec::new();

    for task in &mut phase.tasks {
        if task.id == task_id {
            if let Some(ref new_name) = name {
                changes.push(format!("nom: {}", new_name));
                task.name = new_name.clone();
            }
            if let Some(ref new_desc) = description {
                changes.push(format!("description: {}", new_desc));
                task.description = Some(new_desc.clone());
            }
            if let Some(opt) = optional {
                changes.push(format!("optionnel: {}", opt));
                task.optional = opt;
            }
            found = true;
            break;
        }
    }

    if !found {
        println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
        return;
    }

    phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();

    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!(
        "{} Tâche {} modifiée: {}",
        "✓".green(),
        task_id.cyan(),
        changes.join(", ")
    );
}

fn cmd_task_move(task_id: String, to_phase: String) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let source_phase_id = task_id.split('.').next().unwrap();
    let source_file = phases_dir.join(format!("phase-{}.yml", source_phase_id));
    let dest_file = phases_dir.join(format!("phase-{}.yml", to_phase));

    if !source_file.exists() {
        println!(
            "{} Phase source {} non trouvée",
            "Erreur:".red(),
            source_phase_id.yellow()
        );
        return;
    }

    if !dest_file.exists() {
        println!(
            "{} Phase destination {} non trouvée",
            "Erreur:".red(),
            to_phase.yellow()
        );
        return;
    }

    // Lire la phase source
    let source_content = fs::read_to_string(&source_file).expect("Erreur lecture");
    let mut source_phase: phase::Phase =
        serde_yaml::from_str(&source_content).expect("YAML invalide");

    // Trouver et retirer la tâche
    let task_index = source_phase.tasks.iter().position(|t| t.id == task_id);
    let task = match task_index {
        Some(i) => source_phase.tasks.remove(i),
        None => {
            println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };

    // Lire la phase destination
    let dest_content = fs::read_to_string(&dest_file).expect("Erreur lecture");
    let mut dest_phase: phase::Phase =
        serde_yaml::from_str(&dest_content).expect("YAML invalide");

    // Générer nouvel ID
    let new_task_count = dest_phase.tasks.iter().filter(|t| t.parent.is_none()).count();
    let new_task_id = format!("{}.{}", to_phase, new_task_count + 1);

    // Créer la nouvelle tâche avec le nouvel ID
    let new_task = Task {
        id: new_task_id.clone(),
        name: task.name,
        description: task.description,
        status: task.status,
        parent: None, // Reset parent quand on déplace
        workflow_stage: task.workflow_stage,
        optional: task.optional,
        completed_at: task.completed_at,
    };

    dest_phase.tasks.push(new_task);

    // Mettre à jour les dates
    let now = chrono::Local::now().format("%Y-%m-%d").to_string();
    source_phase.updated_at = now.clone();
    dest_phase.updated_at = now;

    // Sauvegarder les deux phases
    let source_yaml = serde_yaml::to_string(&source_phase).expect("Erreur sérialisation");
    let dest_yaml = serde_yaml::to_string(&dest_phase).expect("Erreur sérialisation");

    fs::write(&source_file, source_yaml).expect("Erreur écriture");
    fs::write(&dest_file, dest_yaml).expect("Erreur écriture");

    println!(
        "{} Tâche {} → {} (nouvel ID: {})",
        "✓".green(),
        task_id.cyan(),
        to_phase.cyan(),
        new_task_id.green()
    );
}

fn cmd_workflow(task_id: String, advance: bool, set: Option<String>) {
    if !advance && set.is_none() {
        println!(
            "{} Spécifie --advance ou --set <stage>",
            "Erreur:".red()
        );
        return;
    }

    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let phase_id = task_id.split('.').next().unwrap();
    let phase_file = phases_dir.join(format!("phase-{}.yml", phase_id));

    if !phase_file.exists() {
        println!("{} Phase {} non trouvée", "Erreur:".red(), phase_id.yellow());
        return;
    }

    let content = fs::read_to_string(&phase_file).expect("Erreur lecture");
    let mut phase: phase::Phase = serde_yaml::from_str(&content).expect("YAML invalide");

    // Vérifier que la phase a un workflow
    let workflow = match &phase.workflow {
        Some(w) if w.enabled && !w.stages.is_empty() => w.clone(),
        _ => {
            // Si pas de workflow sur la phase, utiliser le workflow par défaut de la config
            let config_path = phases_dir.join("config.yml");
            if config_path.exists() {
                let config_content = fs::read_to_string(&config_path).expect("Erreur lecture config");
                let config: config::Config =
                    serde_yaml::from_str(&config_content).unwrap_or_default();
                if config.default_workflow.enabled && !config.default_workflow.stages.is_empty() {
                    phase::PhaseWorkflow {
                        enabled: true,
                        stages: config
                            .default_workflow
                            .stages
                            .iter()
                            .map(|s| phase::WorkflowStage {
                                id: s.id.clone(),
                                name: s.name.clone(),
                            })
                            .collect(),
                    }
                } else {
                    println!(
                        "{} Aucun workflow configuré pour cette phase",
                        "Erreur:".red()
                    );
                    return;
                }
            } else {
                println!(
                    "{} Aucun workflow configuré pour cette phase",
                    "Erreur:".red()
                );
                return;
            }
        }
    };

    // Trouver la tâche
    let task = match phase.tasks.iter_mut().find(|t| t.id == task_id) {
        Some(t) => t,
        None => {
            println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };

    let stage_ids: Vec<&str> = workflow.stages.iter().map(|s| s.id.as_str()).collect();

    if let Some(target_stage) = set {
        // Forcer une étape spécifique
        if !stage_ids.contains(&target_stage.as_str()) {
            println!(
                "{} Étape '{}' invalide. Étapes disponibles: {}",
                "Erreur:".red(),
                target_stage.yellow(),
                stage_ids.join(", ").cyan()
            );
            return;
        }

        let old_stage = task.workflow_stage.clone().unwrap_or_else(|| "-".to_string());
        task.workflow_stage = Some(target_stage.clone());

        phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();
        let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
        fs::write(&phase_file, yaml).expect("Erreur écriture");

        println!(
            "{} Tâche {} : {} → {}",
            "✓".green(),
            task_id.cyan(),
            old_stage.dimmed(),
            target_stage.green()
        );
    } else if advance {
        // Avancer à l'étape suivante
        let current_index = task
            .workflow_stage
            .as_ref()
            .and_then(|s| stage_ids.iter().position(|&id| id == s));

        let next_index = match current_index {
            Some(i) if i + 1 < stage_ids.len() => i + 1,
            Some(i) if i + 1 >= stage_ids.len() => {
                println!(
                    "{} Tâche {} déjà à la dernière étape ({})",
                    "ℹ".blue(),
                    task_id.cyan(),
                    stage_ids.last().unwrap().green()
                );
                return;
            }
            None => 0, // Pas encore d'étape, commencer à la première
            _ => return,
        };

        let old_stage = task.workflow_stage.clone().unwrap_or_else(|| "-".to_string());
        let new_stage = stage_ids[next_index].to_string();
        task.workflow_stage = Some(new_stage.clone());

        phase.updated_at = chrono::Local::now().format("%Y-%m-%d").to_string();
        let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
        fs::write(&phase_file, yaml).expect("Erreur écriture");

        println!(
            "{} Tâche {} : {} → {}",
            "→".green(),
            task_id.cyan(),
            old_stage.dimmed(),
            new_stage.green()
        );
    }
}

fn cmd_report(json: bool) {
    let phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    if phases.is_empty() {
        if json {
            println!("{{}}");
        } else {
            println!("Aucune phase trouvée.");
        }
        return;
    }

    // Collecter toutes les tâches avec leur contexte
    #[derive(serde::Serialize)]
    struct TaskInfo {
        id: String,
        name: String,
        phase_id: String,
        phase_name: String,
        status: String,
        optional: bool,
        workflow_stage: Option<String>,
    }

    let mut all_tasks: Vec<TaskInfo> = Vec::new();
    let mut phases_in_progress: Vec<&phase::Phase> = Vec::new();
    let mut phases_pending: Vec<&phase::Phase> = Vec::new();
    let mut phases_done: Vec<&phase::Phase> = Vec::new();
    let mut phases_blocked: Vec<&phase::Phase> = Vec::new();

    for phase in &phases {
        match phase.status.as_str() {
            "in_progress" => phases_in_progress.push(phase),
            "done" => phases_done.push(phase),
            "blocked" => phases_blocked.push(phase),
            _ => phases_pending.push(phase),
        }

        for task in &phase.tasks {
            all_tasks.push(TaskInfo {
                id: task.id.clone(),
                name: task.name.clone(),
                phase_id: phase.id.clone(),
                phase_name: phase.name.clone(),
                status: task.status.clone(),
                optional: task.optional,
                workflow_stage: task.workflow_stage.clone(),
            });
        }
    }

    // Catégoriser les tâches
    let tasks_done: Vec<_> = all_tasks.iter().filter(|t| t.status == "done").collect();
    let tasks_in_progress: Vec<_> = all_tasks.iter().filter(|t| t.status == "in_progress").collect();
    let tasks_pending: Vec<_> = all_tasks.iter().filter(|t| t.status == "pending" && !t.optional).collect();
    let tasks_optional: Vec<_> = all_tasks.iter().filter(|t| t.optional && t.status != "done").collect();
    let tasks_blocked: Vec<_> = all_tasks.iter().filter(|t| t.status == "blocked").collect();

    if json {
        #[derive(serde::Serialize)]
        struct Report {
            summary: Summary,
            phases_in_progress: Vec<PhaseInfo>,
            next_tasks: Vec<TaskInfo>,
            optional_tasks: Vec<TaskInfo>,
            blocked: Blocked,
        }

        #[derive(serde::Serialize)]
        struct Summary {
            total_phases: usize,
            phases_done: usize,
            phases_in_progress: usize,
            phases_pending: usize,
            phases_blocked: usize,
            total_tasks: usize,
            tasks_done: usize,
            tasks_in_progress: usize,
            tasks_pending: usize,
            tasks_optional: usize,
            tasks_blocked: usize,
            progress_percent: f32,
        }

        #[derive(serde::Serialize)]
        struct PhaseInfo {
            id: String,
            name: String,
            priority: u32,
            progress: String,
        }

        #[derive(serde::Serialize)]
        struct Blocked {
            phases: Vec<String>,
            tasks: Vec<TaskInfo>,
        }

        let total_required = tasks_done.len() + tasks_in_progress.len() + tasks_pending.len() + tasks_blocked.len();
        let progress_percent = if total_required > 0 {
            (tasks_done.len() as f32 / total_required as f32) * 100.0
        } else {
            0.0
        };

        let report = Report {
            summary: Summary {
                total_phases: phases.len(),
                phases_done: phases_done.len(),
                phases_in_progress: phases_in_progress.len(),
                phases_pending: phases_pending.len(),
                phases_blocked: phases_blocked.len(),
                total_tasks: all_tasks.len(),
                tasks_done: tasks_done.len(),
                tasks_in_progress: tasks_in_progress.len(),
                tasks_pending: tasks_pending.len(),
                tasks_optional: tasks_optional.len(),
                tasks_blocked: tasks_blocked.len(),
                progress_percent,
            },
            phases_in_progress: phases_in_progress
                .iter()
                .map(|p| {
                    let total = p.tasks.len();
                    let done = p.tasks.iter().filter(|t| t.status == "done").count();
                    PhaseInfo {
                        id: p.id.clone(),
                        name: p.name.clone(),
                        priority: p.priority,
                        progress: format!("{}/{}", done, total),
                    }
                })
                .collect(),
            next_tasks: tasks_pending
                .iter()
                .take(10)
                .map(|t| TaskInfo {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    phase_id: t.phase_id.clone(),
                    phase_name: t.phase_name.clone(),
                    status: t.status.clone(),
                    optional: t.optional,
                    workflow_stage: t.workflow_stage.clone(),
                })
                .collect(),
            optional_tasks: tasks_optional
                .iter()
                .map(|t| TaskInfo {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    phase_id: t.phase_id.clone(),
                    phase_name: t.phase_name.clone(),
                    status: t.status.clone(),
                    optional: t.optional,
                    workflow_stage: t.workflow_stage.clone(),
                })
                .collect(),
            blocked: Blocked {
                phases: phases_blocked.iter().map(|p| format!("{} — {}", p.id, p.name)).collect(),
                tasks: tasks_blocked
                    .iter()
                    .map(|t| TaskInfo {
                        id: t.id.clone(),
                        name: t.name.clone(),
                        phase_id: t.phase_id.clone(),
                        phase_name: t.phase_name.clone(),
                        status: t.status.clone(),
                        optional: t.optional,
                        workflow_stage: t.workflow_stage.clone(),
                    })
                    .collect(),
            },
        };

        let output = serde_json::to_string_pretty(&report).expect("Erreur sérialisation JSON");
        println!("{}", output);
    } else {
        // Affichage texte
        println!();
        println!("{}", "═══════════════════════════════════════════════════════════".dimmed());
        println!("{}", "                    📊 RAPPORT DE PROGRESSION              ".bold());
        println!("{}", "═══════════════════════════════════════════════════════════".dimmed());
        println!();

        // Résumé global
        let total_required = tasks_done.len() + tasks_in_progress.len() + tasks_pending.len() + tasks_blocked.len();
        let progress_percent = if total_required > 0 {
            (tasks_done.len() as f32 / total_required as f32) * 100.0
        } else {
            0.0
        };

        println!("{}", "📈 RÉSUMÉ".bold().cyan());
        println!("   Phases: {} terminées / {} total",
            phases_done.len().to_string().green(),
            phases.len()
        );
        println!("   Tâches: {} / {} ({:.0}%)",
            tasks_done.len().to_string().green(),
            total_required,
            progress_percent
        );

        // Barre de progression
        let bar_width = 30;
        let filled = ((progress_percent / 100.0) * bar_width as f32) as usize;
        let bar = format!(
            "[{}{}]",
            "█".repeat(filled).green(),
            "░".repeat(bar_width - filled).dimmed()
        );
        println!("   {}", bar);
        println!();

        // Phases en cours
        if !phases_in_progress.is_empty() {
            println!("{}", "🔄 PHASES EN COURS".bold().yellow());
            for phase in &phases_in_progress {
                let total = phase.tasks.len();
                let done = phase.tasks.iter().filter(|t| t.status == "done").count();
                println!(
                    "   [P{}] {} — {} ({}/{})",
                    phase.priority,
                    phase.id.cyan(),
                    phase.name,
                    done,
                    total
                );
            }
            println!();
        }

        // Tâches en cours
        if !tasks_in_progress.is_empty() {
            println!("{}", "⚡ EN COURS MAINTENANT".bold().yellow());
            for task in &tasks_in_progress {
                let stage = task.workflow_stage.as_ref().map(|s| format!(" [{}]", s)).unwrap_or_default();
                println!(
                    "   {} {} — {}{}",
                    task.id.cyan(),
                    task.name,
                    task.phase_name.dimmed(),
                    stage.dimmed()
                );
            }
            println!();
        }

        // Prochaines tâches
        if !tasks_pending.is_empty() {
            println!("{}", "📋 PROCHAINES TÂCHES".bold());
            for task in tasks_pending.iter().take(10) {
                println!(
                    "   ⬜ {} {} — {}",
                    task.id.cyan(),
                    task.name,
                    task.phase_name.dimmed()
                );
            }
            if tasks_pending.len() > 10 {
                println!("   ... et {} autres", tasks_pending.len() - 10);
            }
            println!();
        }

        // Tâches optionnelles
        if !tasks_optional.is_empty() {
            println!("{}", "💡 OPTIONNEL (mise de côté)".bold().blue());
            for task in &tasks_optional {
                println!(
                    "   ◇ {} {} — {}",
                    task.id.cyan(),
                    task.name,
                    task.phase_name.dimmed()
                );
            }
            println!();
        }

        // Bloqué
        if !phases_blocked.is_empty() || !tasks_blocked.is_empty() {
            println!("{}", "🚫 BLOQUÉ".bold().red());
            for phase in &phases_blocked {
                println!("   Phase {} — {}", phase.id.red(), phase.name);
            }
            for task in &tasks_blocked {
                println!(
                    "   {} {} — {}",
                    task.id.red(),
                    task.name,
                    task.phase_name.dimmed()
                );
            }
            println!();
        }

        // Phases à venir
        if !phases_pending.is_empty() {
            println!("{}", "📅 PHASES À VENIR".bold().dimmed());
            for phase in phases_pending.iter().take(5) {
                println!(
                    "   [P{}] {} — {}",
                    phase.priority,
                    phase.id,
                    phase.name
                );
            }
            if phases_pending.len() > 5 {
                println!("   ... et {} autres", phases_pending.len() - 5);
            }
            println!();
        }

        println!("{}", "═══════════════════════════════════════════════════════════".dimmed());
    }
}

fn cmd_export() {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    // Lit la config
    let config_path = phases_dir.join("config.yml");
    let config: config::Config = if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        config::Config::default()
    };

    // Lit toutes les phases
    let entries = match fs::read_dir(phases_dir) {
        Ok(entries) => entries,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phases: Vec<phase::Phase> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        if !filename.starts_with("phase-") || !filename.ends_with(".yml") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let p: phase::Phase = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };

        phases.push(p);
    }

    phases.sort_by(|a, b| a.priority.cmp(&b.priority));

    // Génère le ROADMAP.md
    let mut md = String::new();
    md.push_str(&format!("# {} - Roadmap\n\n", config.project.name));
    md.push_str(&format!("{}\n\n", config.project.description));
    md.push_str("---\n\n");
    md.push_str("## Phases\n\n");
    md.push_str("| Phase | Nom | Statut | Priorité | Progression |\n");
    md.push_str("|-------|-----|--------|----------|-------------|\n");

    for phase in &phases {
        let status_icon = match phase.status.as_str() {
            "done" => "✅",
            "in_progress" => "🔄",
            "blocked" => "🚫",
            _ => "⬜",
        };

        let total = phase.tasks.len();
        let done = phase.tasks.iter().filter(|t| t.status == "done").count();
        let progress = if total > 0 {
            format!("{}/{}", done, total)
        } else {
            "-".to_string()
        };

        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            phase.id, phase.name, status_icon, phase.priority, progress
        ));
    }

    md.push_str("\n---\n\n");

    // Détails de chaque phase
    for phase in &phases {
        let status_icon = match phase.status.as_str() {
            "done" => "✅",
            "in_progress" => "🔄",
            "blocked" => "🚫",
            _ => "⬜",
        };

        md.push_str(&format!("## Phase {} — {} {}\n\n", phase.id, phase.name, status_icon));

        if !phase.description.is_empty() {
            md.push_str(&format!("{}\n\n", phase.description));
        }

        if !phase.tasks.is_empty() {
            md.push_str("### Tâches\n\n");
            for task in &phase.tasks {
                if task.parent.is_some() {
                    continue; // Skip subtasks, they'll be shown under their parent
                }

                let task_icon = match task.status.as_str() {
                    "done" => "✅",
                    "in_progress" => "🔄",
                    "blocked" => "🚫",
                    _ => "⬜",
                };

                let optional = if task.optional { " *(optionnel)*" } else { "" };
                md.push_str(&format!("- {} **{}** — {}{}\n", task_icon, task.id, task.name, optional));

                // Sous-tâches
                for subtask in &phase.tasks {
                    if subtask.parent.as_ref() == Some(&task.id) {
                        let sub_icon = match subtask.status.as_str() {
                            "done" => "✅",
                            "in_progress" => "🔄",
                            "blocked" => "🚫",
                            _ => "⬜",
                        };
                        md.push_str(&format!("  - {} {} — {}\n", sub_icon, subtask.id, subtask.name));
                    }
                }
            }
            md.push_str("\n");
        }

        if !phase.notes.is_empty() {
            md.push_str("### Notes\n\n");
            for note in &phase.notes {
                md.push_str(&format!("- **{}** : {}\n", note.date, note.content));
            }
            md.push_str("\n");
        }

        md.push_str("---\n\n");
    }

    // Écrit le fichier
    let roadmap_path = Path::new(&config.export.roadmap_path);
    if let Err(e) = fs::write(roadmap_path, &md) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!("{} Roadmap exportée", "✓".green());
    println!("  Fichier: {}", config.export.roadmap_path.cyan());
    println!("  {} phases exportées", phases.len());
}

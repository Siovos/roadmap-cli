//! Import command - bulk task creation from YAML

use std::fs;
use std::io::Read;
use std::path::Path;
use colored::Colorize;
use crate::phase::{Phase, Task};
use crate::utils::today;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum ImportItem {
    Full {
        name: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        files: Vec<String>,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        assignee: Option<String>,
        #[serde(default)]
        due: Option<String>,
    },
    Name(String),
}

pub fn cmd_import(phase_id: String, file: String) {
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

    let yaml_content = if file == "-" {
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            println!("{} Erreur lecture stdin: {}", "Erreur:".red(), e);
            return;
        }
        buf
    } else {
        match fs::read_to_string(&file) {
            Ok(c) => c,
            Err(e) => {
                println!("{} Erreur lecture {}: {}", "Erreur:".red(), file, e);
                return;
            }
        }
    };

    let items: Vec<ImportItem> = match serde_yaml::from_str(&yaml_content) {
        Ok(items) => items,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            println!();
            println!("Format attendu (liste de noms ou objets) :");
            println!("  - \"Nom de la tâche\"");
            println!("  - name: \"Autre tâche\"");
            println!("    description: \"Description longue\"");
            println!("    optional: true");
            return;
        }
    };

    if items.is_empty() {
        println!("{} Aucune tâche à importer", "Erreur:".red());
        return;
    }

    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let mut phase: Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    let existing_count = phase.tasks.iter().filter(|t| t.parent.is_none()).count();
    let mut added = 0;

    for (i, item) in items.iter().enumerate() {
        let task_num = existing_count + i + 1;
        let task_id = format!("{}.{}", phase_id, task_num);

        let task = match item {
            ImportItem::Name(name) => Task {
                id: task_id,
                name: name.clone(),
                description: None,
                status: String::from("pending"),
                parent: None,
                workflow_stage: None,
                optional: false,
                completed_at: None,
                blocks: Vec::new(),
                blocked_by: Vec::new(),
                files: Vec::new(),
                tags: Vec::new(),
                assignee: None,
                due: None,
            },
            ImportItem::Full { name, description, optional, files, tags, assignee, due } => Task {
                id: task_id,
                name: name.clone(),
                description: description.clone(),
                status: String::from("pending"),
                parent: None,
                workflow_stage: None,
                optional: *optional,
                completed_at: None,
                blocks: Vec::new(),
                blocked_by: Vec::new(),
                files: files.clone(),
                tags: tags.clone(),
                assignee: assignee.clone(),
                due: due.clone(),
            },
        };

        phase.tasks.push(task);
        added += 1;
    }

    phase.updated_at = today();

    let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
    if let Err(e) = fs::write(&phase_file, yaml) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!(
        "{} {} tâche(s) importée(s) dans la phase {}",
        "✓".green(),
        added,
        phase_id.cyan()
    );
}

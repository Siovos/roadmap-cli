//! Add and edit phases

use std::fs;
use std::path::Path;
use colored::Colorize;
use crate::phase::Phase;
use crate::utils::{load_phases, save_phase, today};

pub fn cmd_add(id: String, name: String, parent: Option<String>, depends_on: Option<Vec<String>>) {
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

    let mut phase = Phase::new(id.clone(), name.clone());
    phase.parent = parent.clone();
    phase.depends_on = depends_on.unwrap_or_default();

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

pub fn cmd_edit(id: String, name: Option<String>, description: Option<String>, depends_on: Option<Vec<String>>) {
    let mut phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    let phase = match phases.iter_mut().find(|p| p.id == id) {
        Some(p) => p,
        None => {
            println!("{} Phase {} non trouvée", "Erreur:".red(), id.yellow());
            return;
        }
    };

    let mut modified = false;

    if let Some(new_name) = name {
        phase.name = new_name;
        modified = true;
    }

    if let Some(new_desc) = description {
        phase.description = new_desc;
        modified = true;
    }

    if let Some(deps) = depends_on {
        phase.depends_on = deps;
        modified = true;
    }

    if !modified {
        println!("{}", "Rien à modifier. Utilisez --name, --description ou --depends-on".yellow());
        return;
    }

    phase.updated_at = today();

    if let Err(e) = save_phase(phase) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!("{} Phase {} modifiée", "✓".green(), id.cyan());
}

pub fn cmd_priority(id: String, priority: u32) {
    let mut phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    let phase = match phases.iter_mut().find(|p| p.id == id) {
        Some(p) => p,
        None => {
            println!("{} Phase {} non trouvée", "Erreur:".red(), id.yellow());
            return;
        }
    };

    let old_priority = phase.priority;
    phase.priority = priority;
    phase.updated_at = today();

    if let Err(e) = save_phase(phase) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!(
        "{} Priorité de {} changée: {} → {}",
        "✓".green(),
        id.cyan(),
        old_priority,
        priority
    );
}

pub fn cmd_phase_remove(id: String, skip_confirm: bool, force: bool) {
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

    let all_phases = match load_phases() {
        Some(p) => p,
        None => return,
    };
    let sub_phases: Vec<String> = all_phases
        .iter()
        .filter(|p| p.parent.as_deref() == Some(id.as_str()))
        .map(|p| p.id.clone())
        .collect();

    if !sub_phases.is_empty() && !force {
        println!(
            "{} La phase {} a {} sous-phase(s): {}",
            "Erreur:".red(),
            id.yellow(),
            sub_phases.len(),
            sub_phases.join(", ").cyan()
        );
        println!(
            "  Utilisez {} pour forcer la suppression",
            "--force".yellow()
        );
        return;
    }

    let content = match fs::read_to_string(&phase_file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}", "Erreur:".red(), e);
            return;
        }
    };

    let phase: crate::phase::Phase = match serde_yaml::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} YAML invalide: {}", "Erreur:".red(), e);
            return;
        }
    };

    if !skip_confirm {
        let task_count = phase.tasks.len();
        if !sub_phases.is_empty() {
            println!(
                "Supprimer la phase {} ({}) avec {} tâche(s) et {} sous-phase(s) ? [y/N]",
                id.cyan(),
                phase.name,
                task_count,
                sub_phases.len()
            );
        } else if task_count > 0 {
            println!(
                "Supprimer la phase {} ({}) avec {} tâche(s) ? [y/N]",
                id.cyan(),
                phase.name,
                task_count
            );
        } else {
            println!(
                "Supprimer la phase {} ({}) ? [y/N]",
                id.cyan(),
                phase.name
            );
        }

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() || !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Annulé".dimmed());
            return;
        }
    }

    if let Err(e) = fs::remove_file(&phase_file) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    let mut sub_removed = 0;
    if force {
        for sub_id in &sub_phases {
            let sub_file = phases_dir.join(format!("phase-{}.yml", sub_id));
            if fs::remove_file(&sub_file).is_ok() {
                sub_removed += 1;
            }
        }
    }

    if sub_removed > 0 {
        println!(
            "{} Phase {} et {} sous-phase(s) supprimées",
            "✓".green(),
            id.cyan(),
            sub_removed
        );
    } else {
        println!("{} Phase {} supprimée", "✓".green(), id.cyan());
    }
}

pub fn cmd_note(id: String, content: String) {
    let mut phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    let phase = match phases.iter_mut().find(|p| p.id == id) {
        Some(p) => p,
        None => {
            println!("{} Phase {} non trouvée", "Erreur:".red(), id.yellow());
            return;
        }
    };

    phase.notes.push(crate::phase::Note {
        date: today(),
        content: content.clone(),
    });
    phase.updated_at = today();

    if let Err(e) = save_phase(phase) {
        println!("{} {}", "Erreur:".red(), e);
        return;
    }

    println!("{} Note ajoutée à {}", "✓".green(), id.cyan());
}

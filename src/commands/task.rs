//! Task management commands

use std::fs;
use std::path::Path;
use colored::Colorize;
use crate::phase::{Phase, Task};
use crate::utils::{today, resolve_phase_file};

pub fn cmd_task_add(phase_id: String, name: String, description: Option<String>, parent: Option<String>, optional: bool, files: Option<Vec<String>>, tags: Option<Vec<String>>, assignee: Option<String>, due: Option<String>) {
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

    let task_id = if let Some(ref parent_id) = parent {
        if !phase.tasks.iter().any(|t| t.id == *parent_id) {
            println!(
                "{} Tâche parente {} non trouvée",
                "Erreur:".red(),
                parent_id.yellow()
            );
            return;
        }
        let count = phase.tasks.iter().filter(|t| t.parent.as_ref() == Some(parent_id)).count();
        format!("{}.{}", parent_id, count + 1)
    } else {
        let existing_ids: Vec<&str> = phase.tasks.iter()
            .filter(|t| t.parent.is_none())
            .map(|t| t.id.as_str())
            .collect();
        let sub_phase_ids: Vec<String> = fs::read_dir(phases_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with(&format!("phase-{}", phase_id)) && name.ends_with(".yml") {
                    let id = name.trim_start_matches("phase-").trim_end_matches(".yml").to_string();
                    if id != phase_id { Some(id) } else { None }
                } else {
                    None
                }
            })
            .collect();
        let mut n = existing_ids.len() + 1;
        loop {
            let candidate = format!("{}.{}", phase_id, n);
            if !existing_ids.contains(&candidate.as_str()) && !sub_phase_ids.contains(&candidate) {
                break candidate;
            }
            n += 1;
        }
    };

    let task = Task {
        id: task_id.clone(),
        name: name.clone(),
        description,
        status: String::from("pending"),
        parent,
        workflow_stage: None,
        optional,
        completed_at: None,
        blocks: Vec::new(),
        blocked_by: Vec::new(),
        files: files.unwrap_or_default(),
        tags: tags.unwrap_or_default(),
        assignee,
        due,
    };

    phase.tasks.push(task);
    phase.updated_at = today();

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

pub fn cmd_task_done(task_ids: Vec<String>) {
    for task_id in &task_ids {
        update_task_status(task_id, "done");
    }
}

pub fn cmd_task_start(task_ids: Vec<String>) {
    for task_id in &task_ids {
        update_task_status(task_id, "in_progress");
    }
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

    let (phase_id, phase_file) = match resolve_phase_file(task_id) {
        Some(r) => r,
        None => {
            println!("{} Phase pour tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };

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

    let mut found = false;
    for task in &mut phase.tasks {
        if task.id == task_id {
            task.status = new_status.to_string();
            if new_status == "done" {
                task.completed_at = Some(today());
            }
            found = true;
            break;
        }
    }

    if !found {
        println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
        return;
    }

    phase.updated_at = today();

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

    // Auto-mark phase as done if all non-optional tasks are done
    if new_status == "done" {
        let all_done = phase.tasks.iter()
            .filter(|t| !t.optional)
            .all(|t| t.status == "done");

        if all_done && !phase.tasks.is_empty() && phase.status != "done" {
            let content = fs::read_to_string(&phase_file).expect("Erreur lecture");
            let mut phase: Phase = serde_yaml::from_str(&content).expect("YAML invalide");
            let old_status = phase.status.clone();
            phase.status = String::from("done");
            phase.updated_at = today();
            let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
            fs::write(&phase_file, yaml).expect("Erreur écriture");
            println!(
                "  {} Phase {} auto-terminée ({} → done)",
                "✅",
                phase_id.cyan(),
                old_status.dimmed()
            );
        }
    }
}

pub fn cmd_task_edit(
    task_id: String,
    name: Option<String>,
    description: Option<String>,
    optional: Option<bool>,
    files: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    assignee: Option<String>,
    due: Option<String>,
) {
    if name.is_none() && description.is_none() && optional.is_none() && files.is_none() && tags.is_none() && assignee.is_none() && due.is_none() {
        println!(
            "{} Spécifie au moins --name, --description, --optional, --files, --tag, --assignee ou --due",
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

    let (_phase_id, phase_file) = match resolve_phase_file(&task_id) {
        Some(r) => r,
        None => {
            println!("{} Phase pour tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };

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
            if let Some(ref new_files) = files {
                changes.push(format!("fichiers: {}", new_files.join(", ")));
                task.files = new_files.clone();
            }
            if let Some(ref new_tags) = tags {
                changes.push(format!("tags: {}", new_tags.join(", ")));
                task.tags = new_tags.clone();
            }
            if let Some(ref new_assignee) = assignee {
                changes.push(format!("assignee: {}", new_assignee));
                task.assignee = Some(new_assignee.clone());
            }
            if let Some(ref new_due) = due {
                changes.push(format!("due: {}", new_due));
                task.due = Some(new_due.clone());
            }
            found = true;
            break;
        }
    }

    if !found {
        println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
        return;
    }

    phase.updated_at = today();

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

pub fn cmd_task_move(task_id: String, to_phase: String) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let (_source_phase_id, source_file) = match resolve_phase_file(&task_id) {
        Some(r) => r,
        None => {
            println!("{} Phase pour tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };
    let dest_file = phases_dir.join(format!("phase-{}.yml", to_phase));

    if !dest_file.exists() {
        println!(
            "{} Phase destination {} non trouvée",
            "Erreur:".red(),
            to_phase.yellow()
        );
        return;
    }

    let source_content = fs::read_to_string(&source_file).expect("Erreur lecture");
    let mut source_phase: Phase = serde_yaml::from_str(&source_content).expect("YAML invalide");

    let task_index = source_phase.tasks.iter().position(|t| t.id == task_id);
    let task = match task_index {
        Some(i) => source_phase.tasks.remove(i),
        None => {
            println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };

    let dest_content = fs::read_to_string(&dest_file).expect("Erreur lecture");
    let mut dest_phase: Phase = serde_yaml::from_str(&dest_content).expect("YAML invalide");

    let existing_dest_ids: Vec<&str> = dest_phase.tasks.iter()
        .filter(|t| t.parent.is_none())
        .map(|t| t.id.as_str())
        .collect();
    let dest_sub_phase_ids: Vec<String> = fs::read_dir(phases_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("phase-{}", to_phase)) && name.ends_with(".yml") {
                let id = name.trim_start_matches("phase-").trim_end_matches(".yml").to_string();
                if id != to_phase { Some(id) } else { None }
            } else {
                None
            }
        })
        .collect();
    let mut n = existing_dest_ids.len() + 1;
    let new_task_id = loop {
        let candidate = format!("{}.{}", to_phase, n);
        if !existing_dest_ids.contains(&candidate.as_str()) && !dest_sub_phase_ids.contains(&candidate) {
            break candidate;
        }
        n += 1;
    };

    let new_task = Task {
        id: new_task_id.clone(),
        name: task.name,
        description: task.description,
        status: task.status,
        parent: None,
        workflow_stage: task.workflow_stage,
        optional: task.optional,
        completed_at: task.completed_at,
        blocks: task.blocks,
        blocked_by: task.blocked_by,
        files: task.files,
        tags: task.tags,
        assignee: task.assignee,
        due: task.due,
    };

    dest_phase.tasks.push(new_task);

    let now = today();
    source_phase.updated_at = now.clone();
    dest_phase.updated_at = now;

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

pub fn cmd_task_blocks(task_id: String, blocked_id: String) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let (task_phase_id, task_phase_file) = match resolve_phase_file(&task_id) {
        Some(r) => r,
        None => {
            println!("{} Phase pour tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };
    let (blocked_phase_id, blocked_phase_file) = match resolve_phase_file(&blocked_id) {
        Some(r) => r,
        None => {
            println!("{} Phase pour tâche {} non trouvée", "Erreur:".red(), blocked_id.yellow());
            return;
        }
    };

    let content = fs::read_to_string(&task_phase_file).expect("Erreur lecture");
    let mut task_phase: Phase = serde_yaml::from_str(&content).expect("YAML invalide");

    let mut task_found = false;
    for task in &mut task_phase.tasks {
        if task.id == task_id {
            if !task.blocks.contains(&blocked_id) {
                task.blocks.push(blocked_id.clone());
            }
            task_found = true;
            break;
        }
    }

    if !task_found {
        println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
        return;
    }

    task_phase.updated_at = today();
    let yaml = serde_yaml::to_string(&task_phase).expect("Erreur sérialisation");
    fs::write(&task_phase_file, yaml).expect("Erreur écriture");

    if task_phase_id == blocked_phase_id {
        let content = fs::read_to_string(&task_phase_file).expect("Erreur lecture");
        let mut phase: Phase = serde_yaml::from_str(&content).expect("YAML invalide");

        for task in &mut phase.tasks {
            if task.id == blocked_id {
                if !task.blocked_by.contains(&task_id) {
                    task.blocked_by.push(task_id.clone());
                }
                break;
            }
        }

        phase.updated_at = today();
        let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
        fs::write(&task_phase_file, yaml).expect("Erreur écriture");
    } else {
        let content = fs::read_to_string(&blocked_phase_file).expect("Erreur lecture");
        let mut blocked_phase: Phase = serde_yaml::from_str(&content).expect("YAML invalide");

        let mut blocked_found = false;
        for task in &mut blocked_phase.tasks {
            if task.id == blocked_id {
                if !task.blocked_by.contains(&task_id) {
                    task.blocked_by.push(task_id.clone());
                }
                blocked_found = true;
                break;
            }
        }

        if !blocked_found {
            println!("{} Tâche {} non trouvée", "Erreur:".red(), blocked_id.yellow());
            return;
        }

        blocked_phase.updated_at = today();
        let yaml = serde_yaml::to_string(&blocked_phase).expect("Erreur sérialisation");
        fs::write(&blocked_phase_file, yaml).expect("Erreur écriture");
    }

    println!(
        "{} {} bloque maintenant {}",
        "🔗".green(),
        task_id.cyan(),
        blocked_id.cyan()
    );
}

pub fn cmd_task_unblocks(task_id: String, blocked_id: String) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let (task_phase_id, task_phase_file) = match resolve_phase_file(&task_id) {
        Some(r) => r,
        None => {
            println!("{} Phase pour tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };
    let (blocked_phase_id, blocked_phase_file) = match resolve_phase_file(&blocked_id) {
        Some(r) => r,
        None => {
            println!("{} Phase pour tâche {} non trouvée", "Erreur:".red(), blocked_id.yellow());
            return;
        }
    };

    let content = fs::read_to_string(&task_phase_file).expect("Erreur lecture");
    let mut task_phase: Phase = serde_yaml::from_str(&content).expect("YAML invalide");

    for task in &mut task_phase.tasks {
        if task.id == task_id {
            task.blocks.retain(|id| id != &blocked_id);
            break;
        }
    }

    task_phase.updated_at = today();
    let yaml = serde_yaml::to_string(&task_phase).expect("Erreur sérialisation");
    fs::write(&task_phase_file, yaml).expect("Erreur écriture");

    if task_phase_id == blocked_phase_id {
        let content = fs::read_to_string(&task_phase_file).expect("Erreur lecture");
        let mut phase: Phase = serde_yaml::from_str(&content).expect("YAML invalide");

        for task in &mut phase.tasks {
            if task.id == blocked_id {
                task.blocked_by.retain(|id| id != &task_id);
                break;
            }
        }

        phase.updated_at = today();
        let yaml = serde_yaml::to_string(&phase).expect("Erreur sérialisation");
        fs::write(&task_phase_file, yaml).expect("Erreur écriture");
    } else {
        if blocked_phase_file.exists() {
            let content = fs::read_to_string(&blocked_phase_file).expect("Erreur lecture");
            let mut blocked_phase: Phase = serde_yaml::from_str(&content).expect("YAML invalide");

            for task in &mut blocked_phase.tasks {
                if task.id == blocked_id {
                    task.blocked_by.retain(|id| id != &task_id);
                    break;
                }
            }

            blocked_phase.updated_at = today();
            let yaml = serde_yaml::to_string(&blocked_phase).expect("Erreur sérialisation");
            fs::write(&blocked_phase_file, yaml).expect("Erreur écriture");
        }
    }

    println!(
        "{} {} ne bloque plus {}",
        "🔓",
        task_id.cyan(),
        blocked_id.cyan()
    );
}

//! Workflow management command

use std::fs;
use std::path::Path;
use colored::Colorize;
use crate::config::Config;
use crate::phase::{Phase, PhaseWorkflow, WorkflowStage};
use crate::utils::{today, resolve_phase_file};

pub fn cmd_workflow(task_id: String, advance: bool, set: Option<String>) {
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

    let (_phase_id, phase_file) = match resolve_phase_file(&task_id) {
        Some(r) => r,
        None => {
            println!("{} Phase pour tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };

    let content = fs::read_to_string(&phase_file).expect("Erreur lecture");
    let mut phase: Phase = serde_yaml::from_str(&content).expect("YAML invalide");

    let workflow = match &phase.workflow {
        Some(w) if w.enabled && !w.stages.is_empty() => w.clone(),
        _ => {
            let config_path = phases_dir.join("config.yml");
            if config_path.exists() {
                let config_content = fs::read_to_string(&config_path).expect("Erreur lecture config");
                let config: Config = serde_yaml::from_str(&config_content).unwrap_or_default();
                if config.default_workflow.enabled && !config.default_workflow.stages.is_empty() {
                    PhaseWorkflow {
                        enabled: true,
                        stages: config
                            .default_workflow
                            .stages
                            .iter()
                            .map(|s| WorkflowStage {
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

    let task = match phase.tasks.iter_mut().find(|t| t.id == task_id) {
        Some(t) => t,
        None => {
            println!("{} Tâche {} non trouvée", "Erreur:".red(), task_id.yellow());
            return;
        }
    };

    let stage_ids: Vec<&str> = workflow.stages.iter().map(|s| s.id.as_str()).collect();

    if let Some(target_stage) = set {
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

        phase.updated_at = today();
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
            None => 0,
            _ => return,
        };

        let old_stage = task.workflow_stage.clone().unwrap_or_else(|| "-".to_string());
        let new_stage = stage_ids[next_index].to_string();
        task.workflow_stage = Some(new_stage.clone());

        phase.updated_at = today();
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

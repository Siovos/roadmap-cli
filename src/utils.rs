//! Utility functions shared across commands

use std::fs;
use std::path::Path;
use colored::Colorize;
use crate::phase::Phase;

/// Load all phases from the .phases directory
/// Returns None if the roadmap is not initialized or on error
pub fn load_phases() -> Option<Vec<Phase>> {
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

    let mut phases: Vec<Phase> = Vec::new();

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

        let p: Phase = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };

        phases.push(p);
    }

    phases.sort_by(|a, b| a.priority.cmp(&b.priority));
    Some(phases)
}

/// Get the status icon for a given status
pub fn get_status_icon(status: &str) -> &'static str {
    match status {
        "done" => "✅",
        "in_progress" => "🔄",
        "blocked" => "🚫",
        _ => "⬜",
    }
}

/// Save a phase to its YAML file
pub fn save_phase(phase: &Phase) -> Result<(), String> {
    let filename = format!(".phases/phase-{}.yml", phase.id);
    let content = serde_yaml::to_string(phase)
        .map_err(|e| format!("Erreur sérialisation: {}", e))?;
    fs::write(&filename, content)
        .map_err(|e| format!("Erreur écriture {}: {}", filename, e))?;
    Ok(())
}

/// Find a phase by ID
pub fn find_phase_mut<'a>(phases: &'a mut [Phase], id: &str) -> Option<&'a mut Phase> {
    phases.iter_mut().find(|p| p.id == id)
}

/// Find a phase by ID (immutable)
pub fn find_phase<'a>(phases: &'a [Phase], id: &str) -> Option<&'a Phase> {
    phases.iter().find(|p| p.id == id)
}

/// Get today's date as a string (YYYY-MM-DD)
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Resolve the phase file for a task ID (e.g. "6.1.1" → "phase-6.1.yml" or "phase-6.yml")
/// Tries progressively shorter prefixes: 6.1 then 6, returning the first match.
/// Returns (phase_id, PathBuf) or None.
pub fn resolve_phase_file(task_id: &str) -> Option<(String, std::path::PathBuf)> {
    let phases_dir = Path::new(".phases");
    let segments: Vec<&str> = task_id.split('.').collect();
    // Try from longest prefix (all but last segment) down to first segment
    for len in (1..segments.len()).rev() {
        let candidate = segments[..len].join(".");
        let path = phases_dir.join(format!("phase-{}.yml", candidate));
        if path.exists() {
            return Some((candidate, path));
        }
    }
    None
}

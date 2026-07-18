//! Sync command - checks coherence between code and roadmap

use std::io::{BufRead, BufReader};
use std::path::Path;
use colored::Colorize;
use regex::Regex;
use crate::utils::load_phases;

pub fn cmd_sync(glob_pattern: String, fix: bool, json: bool) {
    let phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    let mut missing_files: Vec<serde_json::Value> = Vec::new();
    let mut untracked_todos: Vec<serde_json::Value> = Vec::new();
    let mut stale_tasks: Vec<serde_json::Value> = Vec::new();

    // 1. Check file references in tasks
    for phase in &phases {
        for task in &phase.tasks {
            for file in &task.files {
                if !Path::new(file).exists() {
                    missing_files.push(serde_json::json!({
                        "task_id": task.id,
                        "task_name": task.name,
                        "phase_id": phase.id,
                        "file": file,
                    }));
                }
            }

            // Tasks marked done but linked files modified recently?
            // Skip - too complex for now
        }
    }

    // 2. Check for done tasks still having TODO/FIXME in their linked files
    for phase in &phases {
        for task in &phase.tasks {
            if task.status == "done" && !task.files.is_empty() {
                for file in &task.files {
                    if let Some(todos) = find_todos_in_file(file) {
                        if !todos.is_empty() {
                            stale_tasks.push(serde_json::json!({
                                "task_id": task.id,
                                "task_name": task.name,
                                "phase_id": phase.id,
                                "file": file,
                                "todos": todos,
                            }));
                        }
                    }
                }
            }
        }
    }

    // 3. Scan code for untracked TODO/FIXME
    let pattern = glob::Pattern::new(&glob_pattern).expect("Invalid glob pattern");

    let markers = ["TODO", "FIXME", "HACK", "BUG"];
    let pattern_str = format!(r"(?:^|\s)(?://|#|/\*)\s*({})\b[:\s]*(.+?)$", markers.join("|"));
    let re = Regex::new(&pattern_str).expect("Invalid regex");

    let known: Vec<String> = phases
        .iter()
        .flat_map(|p| p.tasks.iter().map(|t| t.name.to_lowercase()))
        .collect();

    let mut files = Vec::new();
    walk_files(Path::new("."), &pattern, &mut files);

    for entry in files {
        let path_str = entry.to_string_lossy().to_string();
        let file = match std::fs::File::open(&entry) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let reader = BufReader::new(file);

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue,
            };

            if let Some(captures) = re.captures(&line) {
                let quote_count = line.matches('"').count();
                if quote_count >= 2 {
                    continue;
                }

                let marker = captures.get(1).map(|m| m.as_str().to_uppercase()).unwrap_or_default();
                let content = captures.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();

                if content.is_empty() || content.len() < 3 {
                    continue;
                }

                let content_lower = content.to_lowercase();
                let is_tracked = known.iter().any(|k| {
                    k.contains(&content_lower) || content_lower.contains(k.as_str())
                });

                if !is_tracked {
                    let short_file = path_str.strip_prefix("./").unwrap_or(&path_str);
                    untracked_todos.push(serde_json::json!({
                        "file": short_file,
                        "line": line_num + 1,
                        "marker": marker,
                        "content": content,
                    }));
                }
            }
        }
    }

    // Output
    if json {
        let report = serde_json::json!({
            "missing_files": missing_files,
            "untracked_todos": untracked_todos,
            "stale_tasks": stale_tasks,
        });
        let output = serde_json::to_string_pretty(&report).expect("Erreur JSON");
        println!("{}", output);
        return;
    }

    println!();
    println!("{}", "🔄 Sync roadmap ↔ code".bold());
    println!();

    let has_issues = !missing_files.is_empty() || !untracked_todos.is_empty() || !stale_tasks.is_empty();

    // Missing files
    if !missing_files.is_empty() {
        println!("  {} {} référence(s) de fichier cassée(s):", "📁".red(), missing_files.len());
        for item in &missing_files {
            println!(
                "    {} Tâche {} → {} n'existe plus",
                "✗".red(),
                item["task_id"].as_str().unwrap().cyan(),
                item["file"].as_str().unwrap().yellow()
            );
        }
        println!();
    }

    // Stale tasks (done but still have TODOs)
    if !stale_tasks.is_empty() {
        println!(
            "  {} {} tâche(s) terminée(s) avec TODO/FIXME restants:",
            "⚠️".yellow(),
            stale_tasks.len()
        );
        for item in &stale_tasks {
            println!(
                "    {} Tâche {} (done) — {} contient encore des TODO",
                "!".yellow(),
                item["task_id"].as_str().unwrap().cyan(),
                item["file"].as_str().unwrap()
            );
            if let Some(todos) = item["todos"].as_array() {
                for todo in todos {
                    println!("      {} {}", "·".dimmed(), todo.as_str().unwrap().dimmed());
                }
            }
        }
        println!();
    }

    // Untracked TODOs
    if !untracked_todos.is_empty() {
        println!(
            "  {} {} TODO/FIXME non trackés dans la roadmap:",
            "📋",
            untracked_todos.len()
        );
        for item in &untracked_todos {
            println!(
                "    {} [{}] {}:{}  {}",
                "?".dimmed(),
                item["marker"].as_str().unwrap(),
                item["file"].as_str().unwrap().dimmed(),
                item["line"],
                item["content"].as_str().unwrap()
            );
        }
        println!();
        if !fix {
            println!(
                "  Pour créer des tâches: {}",
                format!("roadmap scan --glob \"{}\" --create --phase <id>", glob_pattern).yellow()
            );
        }
        println!();
    }

    if !has_issues {
        println!("  {} Code et roadmap synchronisés", "✅".green());
        println!();
    }
}

fn walk_files(dir: &Path, pattern: &glob::Pattern, results: &mut Vec<std::path::PathBuf>) {
    const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "dist", "build", ".phases"];

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            if name_str.starts_with('.') || SKIP_DIRS.contains(&name_str.as_ref()) {
                continue;
            }
            walk_files(&path, pattern, results);
        } else if pattern.matches_path(path.strip_prefix("./").unwrap_or(&path)) {
            results.push(path);
        }
    }
}

fn find_todos_in_file(path: &str) -> Option<Vec<String>> {
    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let re = Regex::new(r"(?i)\b(TODO|FIXME|BUG)\b[:\s]*(.+?)$").ok()?;

    let mut todos = Vec::new();
    for line in reader.lines().filter_map(|l| l.ok()) {
        if let Some(captures) = re.captures(&line) {
            let content = captures.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            if content.len() >= 3 {
                todos.push(content);
            }
        }
    }

    Some(todos)
}

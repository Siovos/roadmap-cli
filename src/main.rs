//! Roadmap CLI - Gestionnaire de roadmaps projet

mod api;
mod auth;
mod cli;
mod credentials;
mod commands;
mod config;
mod data;
mod db;
mod phase;
mod tui;
mod update;
mod utils;
mod web;
mod ws;

use std::fs;
use std::path::Path;
use clap::Parser;
use colored::Colorize;

use cli::{Cli, Commands, TaskCommands, BugCommands, FeatureCommands};
use commands::*;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Add { id, name, parent, depends_on } => cmd_add(id, name, parent, depends_on),
        Commands::Edit { id, name, description, depends_on } => cmd_edit(id, name, description, depends_on),
        Commands::List { table, json, tag, status, assignee, overdue } => cmd_list(table, json, tag, status, assignee, overdue),
        Commands::Tree { json, not_done } => cmd_tree(json, not_done),
        Commands::Show { id, json } => cmd_show(id, json),
        Commands::Task { action } => match action {
            TaskCommands::Add { phase_id, name, description, parent, optional, files, tag, assignee, due } => {
                cmd_task_add(phase_id, name, description, parent, optional, files, tag, assignee, due)
            }
            TaskCommands::Done { task_ids } => cmd_task_done(task_ids),
            TaskCommands::Start { task_ids } => cmd_task_start(task_ids),
            TaskCommands::Edit { task_id, name, description, optional, files, tag, assignee, due } => {
                cmd_task_edit(task_id, name, description, optional, files, tag, assignee, due)
            }
            TaskCommands::Move { task_id, to_phase } => cmd_task_move(task_id, to_phase),
            TaskCommands::Blocks { task_id, blocked_id } => cmd_task_blocks(task_id, blocked_id),
            TaskCommands::Unblocks { task_id, blocked_id } => cmd_task_unblocks(task_id, blocked_id),
        },
        Commands::Bug { action } => match action {
            BugCommands::Add { title, severity, phase, description, assignee, reported_by, target } => {
                cmd_bug_add(title, severity, phase, description, assignee, reported_by, target)
            }
            BugCommands::List { severity, status, json } => cmd_bug_list(severity, status, json),
            BugCommands::Show { id } => cmd_bug_show(id),
            BugCommands::Resolve { id, description, commit } => cmd_bug_resolve(id, description, commit),
            BugCommands::Update { id, status, severity, assignee, title, description, phase } => cmd_bug_update(id, status, severity, assignee, title, description, phase),
        },
        Commands::Feature { action } => match action {
            FeatureCommands::Add { title, priority, phase, description, assignee, requested_by, target } => {
                cmd_feature_add(title, priority, phase, description, assignee, requested_by, target)
            }
            FeatureCommands::List { priority, status, json } => cmd_feature_list(priority, status, json),
            FeatureCommands::Show { id } => cmd_feature_show(id),
            FeatureCommands::Implement { id, description, commit } => cmd_feature_implement(id, description, commit),
            FeatureCommands::Update { id, status, priority, assignee, title, description, phase } => cmd_feature_update(id, status, priority, assignee, title, description, phase),
        },
        Commands::Priority { id, set } => cmd_priority(id, set),
        Commands::Status { id, set } => cmd_status(id, set),
        Commands::Note { id, content } => cmd_note(id, content),
        Commands::Export => cmd_export(),
        Commands::Report { json } => cmd_report(json),
        Commands::Workflow { task_id, advance, set } => cmd_workflow(task_id, advance, set),
        Commands::Ui => cmd_ui(),
        Commands::Serve { port, open } => cmd_serve(port, open),
        Commands::Update { check } => update::cmd_update(check),
        Commands::Next { json } => cmd_next(json),
        Commands::Context { include_done, phase } => cmd_context(include_done, phase),
        Commands::Search { query, json } => cmd_search(query, json),
        Commands::Sync { glob, fix, json } => cmd_sync(glob, fix, json),
        Commands::Scan { glob, create, phase, hidden } => cmd_scan(glob, create, phase, hidden),
        Commands::Coverage { backend, frontend, bff, prefix, backend_prefix, json } => cmd_coverage(backend, frontend, bff, prefix, backend_prefix, json),
        Commands::Hooks { install, uninstall } => cmd_hooks(install, uninstall),
        Commands::Template { name, phase_id, list } => cmd_template(name, phase_id, list),
        Commands::Login { server } => cmd_login(server),
        Commands::Logout => cmd_logout(),
        Commands::Push { project } => cmd_push(project),
        Commands::Log { limit, json } => cmd_log(limit, json),
        Commands::Doctor => cmd_doctor(),
        Commands::Changelog { limit, from, to, format } => cmd_changelog(limit, from, to, format),
        Commands::Generate { what, output } => cmd_generate(what, output),
    }

    // Silent update check (1x/day, cached, non-blocking)
    update::check_update_hint();
}

fn cmd_login(server: String) {
    let rt = tokio::runtime::Runtime::new().expect("Erreur runtime tokio");
    rt.block_on(async {
        let client = reqwest::Client::new();

        // 1. Request device code
        println!("{}", "Connexion à Roadmap...".bold());
        println!();

        let resp = match client
            .post(format!("{}/api/auth/device", server))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!("{} Impossible de joindre le serveur: {}", "Erreur:".red(), e);
                return;
            }
        };

        let data: serde_json::Value = match resp.json().await {
            Ok(d) => d,
            Err(e) => {
                println!("{} Réponse invalide: {}", "Erreur:".red(), e);
                return;
            }
        };

        let device_code = data["device_code"].as_str().unwrap().to_string();
        let user_code = data["user_code"].as_str().unwrap();
        let verification_url = data["verification_url"].as_str().unwrap().to_string();

        println!("  Code: {}", user_code.cyan().bold());
        println!("  Ouvrez cette page pour autoriser l'accès:");
        println!("  {}", verification_url.yellow());
        println!();

        // 2. Open browser
        if let Err(_) = open::that(&verification_url) {
            println!("  {} Ouvrez le lien manuellement dans votre navigateur", "⚠".yellow());
        }

        // 3. Poll for approval
        print!("  En attente d'autorisation");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            print!(".");

            let poll_resp = client
                .get(format!("{}/api/auth/device/{}", server, device_code))
                .send()
                .await;

            let poll_resp = match poll_resp {
                Ok(r) => r,
                Err(_) => continue,
            };

            let status_code = poll_resp.status();
            let poll_data: serde_json::Value = match poll_resp.json().await {
                Ok(d) => d,
                Err(_) => continue,
            };

            let status = poll_data["status"].as_str().unwrap_or("pending");

            match status {
                "approved" => {
                    println!(" {}", "✓".green());
                    println!();

                    let creds = credentials::Credentials {
                        server: server.clone(),
                        access_token: poll_data["access_token"].as_str().unwrap().to_string(),
                        refresh_token: poll_data["refresh_token"].as_str().unwrap().to_string(),
                        user_email: poll_data["user_email"].as_str().unwrap().to_string(),
                        user_name: poll_data["user_name"].as_str().unwrap().to_string(),
                    };

                    match credentials::save(&creds) {
                        Ok(()) => {
                            println!(
                                "  {} Connecté en tant que {} ({})",
                                "✓".green(),
                                creds.user_name.cyan(),
                                creds.user_email.dimmed()
                            );
                            println!("  Credentials sauvegardés dans {}", "~/.roadmap/credentials.json".dimmed());
                        }
                        Err(e) => println!("{} {}", "Erreur:".red(), e),
                    }
                    return;
                }
                "denied" => {
                    println!(" {}", "✗".red());
                    println!();
                    println!("  {} Autorisation refusée", "✗".red());
                    return;
                }
                "expired" => {
                    println!(" {}", "✗".red());
                    println!();
                    println!("  {} Code expiré. Relancez {}", "✗".red(), "roadmap login".yellow());
                    return;
                }
                _ => {
                    // Still pending, continue polling
                    if status_code.as_u16() == 410 {
                        println!(" {}", "✗".red());
                        println!();
                        println!("  {} Code expiré", "✗".red());
                        return;
                    }
                }
            }
        }
    });
}

fn cmd_logout() {
    match credentials::load() {
        Some(creds) => {
            match credentials::clear() {
                Ok(()) => {
                    println!(
                        "{} Déconnecté de {} ({})",
                        "✓".green(),
                        creds.user_name.cyan(),
                        creds.server.dimmed()
                    );
                }
                Err(e) => println!("{} {}", "Erreur:".red(), e),
            }
        }
        None => println!("{}", "Pas de session active".dimmed()),
    }
}

fn cmd_push(project_name: Option<String>) {
    let phases = match utils::load_phases() {
        Some(p) => p,
        None => return,
    };

    let project = project_name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "default".to_string())
    });

    // If we have credentials, push via API instead of direct DB
    if let Some(creds) = credentials::load() {
        cmd_push_via_api(&creds, &project, &phases);
        return;
    }

    println!(
        "{} Utilisez {} pour vous connecter, puis {} pour synchroniser via l'API",
        "Info:".cyan(),
        "roadmap login".yellow(),
        "roadmap push".yellow()
    );
}

fn cmd_push_via_api(creds: &credentials::Credentials, project: &str, phases: &[phase::Phase]) {
    let rt = tokio::runtime::Runtime::new().expect("Erreur runtime tokio");
    rt.block_on(async {
        let client = reqwest::Client::new();
        let server = &creds.server;
        let token = &creds.access_token;

        // 1. Get or create project
        let slug = project.to_lowercase().replace(' ', "-");

        let project_resp = client
            .get(format!("{}/api/v1/projects/{}", server, slug))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;

        let project_exists = matches!(&project_resp, Ok(r) if r.status().is_success());

        if !project_exists {
            let create_resp = client
                .post(format!("{}/api/v1/projects", server))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "name": project,
                    "slug": slug,
                    "description": ""
                }))
                .send()
                .await;

            match create_resp {
                Ok(r) if r.status().is_success() => {}
                Ok(r) => {
                    let body = r.text().await.unwrap_or_default();
                    println!("{} Création projet: {}", "Erreur:".red(), body);
                    return;
                }
                Err(e) => {
                    println!("{} {}", "Erreur:".red(), e);
                    return;
                }
            }
        }

        // 2. Push phases
        let mut phases_ok = 0;
        let mut tasks_ok = 0;

        for phase in phases {
            let phase_resp = client
                .post(format!("{}/api/v1/projects/{}/phases", server, slug))
                .header("Authorization", format!("Bearer {}", token))
                .json(&serde_json::json!({
                    "phase_id": phase.id,
                    "name": phase.name,
                    "description": phase.description,
                    "priority": phase.priority,
                }))
                .send()
                .await;

            // If already exists, update
            match phase_resp {
                Ok(r) if r.status().is_success() || r.status().as_u16() == 409 => {
                    // Update if conflict
                    if r.status().as_u16() == 409 {
                        let _ = client
                            .put(format!("{}/api/v1/projects/{}/phases/{}", server, slug, phase.id))
                            .header("Authorization", format!("Bearer {}", token))
                            .json(&serde_json::json!({
                                "name": phase.name,
                                "description": phase.description,
                                "priority": phase.priority,
                                "status": phase.status,
                            }))
                            .send()
                            .await;
                    }
                    phases_ok += 1;
                }
                _ => continue,
            }

            // 3. Push tasks
            for task in &phase.tasks {
                let task_resp = client
                    .post(format!("{}/api/v1/projects/{}/phases/{}/tasks", server, slug, phase.id))
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&serde_json::json!({
                        "task_id": task.id,
                        "name": task.name,
                        "description": task.description,
                        "optional": task.optional,
                    }))
                    .send()
                    .await;

                match task_resp {
                    Ok(r) if r.status().is_success() || r.status().as_u16() == 409 => {
                        // Update status if conflict
                        if r.status().as_u16() == 409 {
                            let _ = client
                                .put(format!("{}/api/v1/projects/{}/phases/{}/tasks/{}", server, slug, phase.id, task.id))
                                .header("Authorization", format!("Bearer {}", token))
                                .json(&serde_json::json!({
                                    "name": task.name,
                                    "status": task.status,
                                    "description": task.description,
                                    "optional": task.optional,
                                }))
                                .send()
                                .await;
                        }
                        tasks_ok += 1;
                    }
                    _ => {}
                }
            }
        }

        println!(
            "{} Push via API réussi: {} phases, {} tâches → {}",
            "✓".green(),
            phases_ok,
            tasks_ok,
            creds.server.dimmed()
        );
    });
}

fn cmd_serve(port: u16, open_browser: bool) {
    let phases_dir = Path::new(".phases");

    if !phases_dir.exists() {
        println!(
            "{} Roadmap non initialisée. Lance d'abord: {}",
            "Erreur:".red(),
            "roadmap init".yellow()
        );
        return;
    }

    let rt = tokio::runtime::Runtime::new().expect("Erreur création runtime tokio");
    rt.block_on(web::run_server(port, open_browser));
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

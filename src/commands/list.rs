//! List and tree view commands

use colored::Colorize;
use tabled::{Table, Tabled, settings::Style};
use crate::phase::Task;
use crate::utils::{load_phases, get_status_icon};

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

pub fn cmd_list(table: bool, json: bool, tag: Option<String>, status: Option<String>, assignee: Option<String>, overdue: bool) {
    let phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    // Filtered views
    if tag.is_some() || assignee.is_some() || overdue {
        cmd_list_filtered(&phases, tag.as_deref(), status.as_deref(), assignee.as_deref(), overdue, json);
        return;
    }

    // Filter phases by status if specified
    let phases = if let Some(ref status_filter) = status {
        phases.into_iter().filter(|p| p.status == *status_filter).collect()
    } else {
        phases
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
        let output = serde_json::to_string_pretty(&phases).expect("Erreur sérialisation JSON");
        println!("{}", output);
    } else if table {
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

pub fn cmd_tree(json: bool, hide_done: bool) {
    let phases = match load_phases() {
        Some(p) => p,
        None => return,
    };

    // Filter out done phases if --not-done
    let phases: Vec<_> = if hide_done {
        phases.into_iter().filter(|p| p.status != "done").collect()
    } else {
        phases
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

        fn build_task_tree(tasks: &[Task], parent: Option<&str>, hide_done: bool) -> Vec<TreeNode> {
            tasks
                .iter()
                .filter(|t| t.parent.as_deref() == parent)
                .filter(|t| !hide_done || t.status != "done")
                .map(|t| TreeNode {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    node_type: if t.optional { "task_optional".to_string() } else { "task".to_string() },
                    status: t.status.clone(),
                    priority: None,
                    children: build_task_tree(tasks, Some(&t.id), hide_done),
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
                    .filter(|sp| !hide_done || sp.status != "done")
                    .map(|sp| TreeNode {
                        id: sp.id.clone(),
                        name: sp.name.clone(),
                        node_type: "subphase".to_string(),
                        status: sp.status.clone(),
                        priority: Some(sp.priority),
                        children: build_task_tree(&sp.tasks, None, hide_done),
                    })
                    .collect();

                children.extend(build_task_tree(&phase.tasks, None, hide_done));

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

            let children: Vec<_> = sub_phases.iter()
                .filter(|sp| sp.parent.as_ref() == Some(&phase.id))
                .filter(|sp| !hide_done || sp.status != "done")
                .collect();
            for (j, sub) in children.iter().enumerate() {
                let sub_prefix = if is_last_phase { "    " } else { "│   " };
                let sub_branch = if j == children.len() - 1 { "└── " } else { "├── " };
                let sub_icon = get_status_icon(&sub.status);

                println!(
                    "{}{}{} Phase {} — {}",
                    sub_prefix, sub_branch, sub_icon, sub.id, sub.name
                );
            }

            let task_prefix = if is_last_phase { "    " } else { "│   " };
            print_tree_tasks(&phase.tasks, None, task_prefix, &[], hide_done);
        }
    }
}

fn cmd_list_filtered(
    phases: &[crate::phase::Phase],
    tag_filter: Option<&str>,
    status_filter: Option<&str>,
    assignee_filter: Option<&str>,
    overdue: bool,
    json: bool,
) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut matched: Vec<serde_json::Value> = Vec::new();

    for phase in phases {
        for task in &phase.tasks {
            let tag_ok = tag_filter.is_none()
                || task.tags.iter().any(|t| t == tag_filter.unwrap());
            let status_ok = status_filter.is_none()
                || task.status == status_filter.unwrap();
            let assignee_ok = assignee_filter.is_none()
                || task.assignee.as_deref() == assignee_filter;
            let overdue_ok = !overdue
                || (task.due.as_ref().is_some_and(|d| d.as_str() < today.as_str())
                    && task.status != "done");

            if tag_ok && status_ok && assignee_ok && overdue_ok {
                matched.push(serde_json::json!({
                    "id": task.id,
                    "name": task.name,
                    "status": task.status,
                    "phase_id": phase.id,
                    "phase_name": phase.name,
                    "tags": task.tags,
                    "optional": task.optional,
                    "assignee": task.assignee,
                    "due": task.due,
                }));
            }
        }
    }

    if json {
        let output = serde_json::to_string_pretty(&matched).expect("Erreur JSON");
        println!("{}", output);
        return;
    }

    if matched.is_empty() {
        let mut filters = Vec::new();
        if let Some(t) = tag_filter { filters.push(format!("tag={}", t)); }
        if let Some(s) = status_filter { filters.push(format!("status={}", s)); }
        if let Some(a) = assignee_filter { filters.push(format!("assignee={}", a)); }
        if overdue { filters.push("overdue".to_string()); }
        println!("Aucune tâche trouvée (filtres: {})", filters.join(", ").yellow());
        return;
    }

    // Build title
    let mut title_parts = Vec::new();
    if let Some(t) = tag_filter { title_parts.push(format!("tag:{}", t)); }
    if let Some(s) = status_filter { title_parts.push(format!("status:{}", s)); }
    if let Some(a) = assignee_filter { title_parts.push(format!("@{}", a)); }
    if overdue { title_parts.push("en retard".to_string()); }

    println!(
        "{} {} tâche(s) [{}]:",
        "🔍",
        matched.len().to_string().cyan(),
        title_parts.join(", ").bold()
    );
    println!();

    for item in &matched {
        let status_icon = get_status_icon(item["status"].as_str().unwrap_or("pending"));
        let optional = if item["optional"].as_bool().unwrap_or(false) { " (opt)" } else { "" };

        let mut extras = Vec::new();
        if let Some(a) = item["assignee"].as_str() {
            extras.push(format!("@{}", a));
        }
        if let Some(d) = item["due"].as_str() {
            let is_overdue = d < today.as_str() && item["status"].as_str() != Some("done");
            if is_overdue {
                extras.push(format!("⚠️ {}", d));
            } else {
                extras.push(format!("📅 {}", d));
            }
        }

        let extras_str = if extras.is_empty() {
            String::new()
        } else {
            format!(" {}", extras.join(" ").dimmed())
        };

        println!(
            "  {} {} — {}{}{}",
            status_icon,
            item["id"].as_str().unwrap().cyan(),
            item["name"].as_str().unwrap(),
            optional,
            extras_str
        );
        println!(
            "      Phase {} — {}",
            item["phase_id"].as_str().unwrap().dimmed(),
            item["phase_name"].as_str().unwrap().dimmed()
        );
    }
}

fn print_tree_tasks(tasks: &[Task], parent: Option<&str>, prefix: &str, parent_is_last: &[bool], hide_done: bool) {
    let filtered: Vec<_> = tasks.iter()
        .filter(|t| t.parent.as_deref() == parent)
        .filter(|t| !hide_done || t.status != "done")
        .collect();

    for (i, task) in filtered.iter().enumerate() {
        let is_last = i == filtered.len() - 1;
        let branch = if is_last { "└── " } else { "├── " };
        let status_icon = get_status_icon(&task.status);
        let optional = if task.optional { " (opt)" } else { "" };

        println!(
            "{}{}{} {} — {}{}",
            prefix, branch, status_icon, task.id, task.name, optional
        );

        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        let mut new_is_last = parent_is_last.to_vec();
        new_is_last.push(is_last);
        print_tree_tasks(tasks, Some(&task.id), &child_prefix, &new_is_last, hide_done);
    }
}

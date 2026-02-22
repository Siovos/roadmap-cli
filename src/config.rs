use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    pub default_workflow: WorkflowConfig,
    pub statuses: Vec<Status>,
    pub export: ExportConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub enabled: bool,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stage {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub id: String,
    pub label: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    pub roadmap_path: String,
    pub phases_docs_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            project: ProjectConfig {
                name: String::from("mon-projet"),
                description: String::from("Description du projet"),
            },
            default_workflow: WorkflowConfig {
                enabled: false,
                stages: vec![
                    Stage {
                        id: String::from("analysis"),
                        name: String::from("Analyse"),
                    },
                    Stage {
                        id: String::from("design"),
                        name: String::from("Conception"),
                    },
                    Stage {
                        id: String::from("implementation"),
                        name: String::from("Développement"),
                    },
                    Stage {
                        id: String::from("testing"),
                        name: String::from("Tests"),
                    },
                    Stage {
                        id: String::from("documentation"),
                        name: String::from("Documentation"),
                    },
                ],
            },
            statuses: vec![
                Status {
                    id: String::from("pending"),
                    label: String::from("À faire"),
                    icon: String::from("⬜"),
                },
                Status {
                    id: String::from("in_progress"),
                    label: String::from("En cours"),
                    icon: String::from("🔄"),
                },
                Status {
                    id: String::from("done"),
                    label: String::from("Terminé"),
                    icon: String::from("✅"),
                },
                Status {
                    id: String::from("blocked"),
                    label: String::from("Bloqué"),
                    icon: String::from("🚫"),
                },
            ],
            export: ExportConfig {
                roadmap_path: String::from("./ROADMAP.md"),
                phases_docs_path: String::from("./docs"),
            },
        }
    }
}

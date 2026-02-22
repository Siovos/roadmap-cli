use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Phase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: u32,
    pub status: String,
    pub parent: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub workflow: Option<PhaseWorkflow>,
    pub tasks: Vec<Task>,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseWorkflow {
    pub enabled: bool,
    pub stages: Vec<WorkflowStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStage {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub parent: Option<String>,
    pub workflow_stage: Option<String>,
    pub optional: bool,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub date: String,
    pub content: String,
}

impl Phase {
    pub fn new(id: String, name: String) -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();

        Phase {
            id,
            name,
            description: String::new(),
            priority: 10,
            status: String::from("pending"),
            parent: None,
            created_at: now.clone(),
            updated_at: now,
            workflow: None,
            tasks: Vec::new(),
            notes: Vec::new(),
        }
    }
}

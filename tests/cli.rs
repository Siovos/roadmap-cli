use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn roadmap() -> Command {
    Command::cargo_bin("roadmap-cli").unwrap()
}

fn in_tmp() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    roadmap().arg("init").current_dir(dir.path()).assert().success();
    dir
}

// ============================================================================
// init
// ============================================================================

#[test]
fn init_creates_phases_dir() {
    let dir = tempfile::tempdir().unwrap();
    roadmap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Roadmap initialisée"));
    assert!(dir.path().join(".phases").exists());
    assert!(dir.path().join(".phases/config.yml").exists());
}

#[test]
fn init_twice_fails() {
    let dir = in_tmp();
    roadmap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("existe déjà"));
}

// ============================================================================
// add / show
// ============================================================================

#[test]
fn add_creates_phase_file() {
    let dir = in_tmp();
    roadmap()
        .args(["add", "1", "MVP"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Phase 1 créée"));
    assert!(dir.path().join(".phases/phase-1.yml").exists());
}

#[test]
fn add_with_description_priority_status() {
    let dir = in_tmp();
    roadmap()
        .args(["add", "1", "Test", "--description", "Une desc", "--priority", "2", "--status", "in_progress"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"priority\": 2"))
        .stdout(predicate::str::contains("\"status\": \"in_progress\""))
        .stdout(predicate::str::contains("Une desc"));
}

#[test]
fn add_duplicate_fails() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["add", "1", "B"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("existe déjà"));
}

#[test]
fn show_json_output() {
    let dir = in_tmp();
    roadmap().args(["add", "5", "Infra"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["show", "5", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"5\""))
        .stdout(predicate::str::contains("\"name\": \"Infra\""));
}

// ============================================================================
// task add / done / start
// ============================================================================

#[test]
fn task_add_and_done() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "add", "1", "Setup CI"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tâche 1.1 ajoutée"));
    roadmap()
        .args(["task", "done", "1.1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("done"));
}

#[test]
fn task_start_multiple() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "B"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "start", "1.1", "1.2"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1.1"))
        .stdout(predicate::str::contains("1.2"));
}

// ============================================================================
// task remove
// ============================================================================

#[test]
fn task_remove_with_yes() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "B"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "remove", "1.1", "-y"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Tâche 1.1 supprimée"));
    // Task 1.2 should still exist
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1.2"))
        .stdout(predicate::str::contains("B"));
}

#[test]
fn task_remove_multiple() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "B"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "C"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "remove", "1.1", "1.3", "-y"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1.1 supprimée"))
        .stdout(predicate::str::contains("1.3 supprimée"));
}

#[test]
fn task_remove_nonexistent() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "remove", "1.99", "-y"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("non trouvée"));
}

// ============================================================================
// phase remove
// ============================================================================

#[test]
fn phase_remove_with_yes() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["remove", "1", "-y"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Phase 1 supprimée"));
    assert!(!dir.path().join(".phases/phase-1.yml").exists());
}

#[test]
fn phase_remove_blocks_with_subphases() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "1a", "Sub", "--parent", "1"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["remove", "1", "-y"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("sous-phase"))
        .stdout(predicate::str::contains("--force"));
    // File should still exist
    assert!(dir.path().join(".phases/phase-1.yml").exists());
}

#[test]
fn phase_remove_force_with_subphases() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "1a", "Sub", "--parent", "1"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["remove", "1", "-y", "--force"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("sous-phase(s) supprimées"));
    assert!(!dir.path().join(".phases/phase-1.yml").exists());
    assert!(!dir.path().join(".phases/phase-1a.yml").exists());
}

// ============================================================================
// import
// ============================================================================

#[test]
fn import_from_file() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();

    let tasks_file = dir.path().join("tasks.yml");
    fs::write(&tasks_file, "- \"Setup CI\"\n- \"Tests\"\n- name: \"Deploy\"\n  optional: true\n").unwrap();

    roadmap()
        .args(["import", "1", "tasks.yml"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 tâche(s) importée(s)"));

    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup CI"))
        .stdout(predicate::str::contains("Tests"))
        .stdout(predicate::str::contains("Deploy"));
}

#[test]
fn import_from_stdin() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();

    roadmap()
        .args(["import", "1", "-"])
        .current_dir(dir.path())
        .write_stdin("- \"Tâche stdin A\"\n- \"Tâche stdin B\"\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("2 tâche(s) importée(s)"));
}

#[test]
fn import_invalid_yaml() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();

    let bad = dir.path().join("bad.yml");
    fs::write(&bad, "not: a: list:").unwrap();

    roadmap()
        .args(["import", "1", "bad.yml"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("YAML invalide"));
}

// ============================================================================
// priority (positional + --set)
// ============================================================================

#[test]
fn priority_positional() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["priority", "1", "3"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("10 → 3"));
}

#[test]
fn priority_with_set_flag() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["priority", "1", "--set", "5"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("10 → 5"));
}

#[test]
fn priority_no_value_shows_error() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["priority", "1"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("Spécifie la priorité"));
}

// ============================================================================
// note (add / list / edit / remove)
// ============================================================================

#[test]
fn note_add_and_list() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["note", "1", "Première note"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Note ajoutée"));
    roadmap()
        .args(["note", "1", "--list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Première note"));
}

#[test]
fn note_edit() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["note", "1", "Ancienne"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["note", "1", "Corrigée", "--edit", "1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Note #1"));
    roadmap()
        .args(["note", "1", "--list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Corrigée"))
        .stdout(predicate::str::contains("Ancienne").not());
}

#[test]
fn note_remove() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["note", "1", "Note A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["note", "1", "Note B"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["note", "1", "--remove", "1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("supprimée"));
    roadmap()
        .args(["note", "1", "--list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Note B"))
        .stdout(predicate::str::contains("Note A").not());
}

#[test]
fn note_remove_invalid_index() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["note", "1", "--remove", "99"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("Index 99 invalide"));
}

// ============================================================================
// status
// ============================================================================

#[test]
fn status_change() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["status", "1", "--set", "in_progress"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"in_progress\""));
}

// ============================================================================
// report / list / tree
// ============================================================================

#[test]
fn report_json() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["report", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_phases\": 1"))
        .stdout(predicate::str::contains("\"total_tasks\": 1"));
}

#[test]
fn tree_json() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["tree", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"1\""));
}

#[test]
fn list_json() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "2", "Infra"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["list", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("MVP"))
        .stdout(predicate::str::contains("Infra"));
}

// ============================================================================
// task edit
// ============================================================================

#[test]
fn task_edit_name() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Ancien nom"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "edit", "1.1", "--name", "Nouveau nom"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("modifiée"));
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Nouveau nom"));
}

// ============================================================================
// auto-complete phase when all tasks done
// ============================================================================

#[test]
fn auto_complete_phase() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "B"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "done", "1.1"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "done", "1.2"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("auto-terminée"));
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"done\""));
}

// ============================================================================
// doctor
// ============================================================================

#[test]
fn doctor_on_clean_roadmap() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["doctor"])
        .current_dir(dir.path())
        .assert()
        .success();
}

// ============================================================================
// edge cases
// ============================================================================

#[test]
fn commands_fail_without_init() {
    let dir = tempfile::tempdir().unwrap();
    roadmap()
        .args(["add", "1", "MVP"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("non initialisée"));
}

#[test]
fn show_nonexistent_phase() {
    let dir = in_tmp();
    roadmap()
        .args(["show", "999"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("non trouvée"));
}

// ============================================================================
// phase edit
// ============================================================================

#[test]
fn phase_edit_name_and_description() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["edit", "1", "--name", "MVP v2", "--description", "Nouvelle desc"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Phase 1 modifiée"));
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("MVP v2"))
        .stdout(predicate::str::contains("Nouvelle desc"));
}

#[test]
fn phase_edit_nothing_warns() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["edit", "1"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("Rien à modifier"));
}

// ============================================================================
// task move
// ============================================================================

#[test]
fn task_move_between_phases() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "Source"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "2", "Dest"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Ma tâche"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "move", "1.1", "--to", "2"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("→"))
        .stdout(predicate::str::contains("2.1"));
    roadmap()
        .args(["show", "2", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Ma tâche"));
}

// ============================================================================
// task blocks / unblocks
// ============================================================================

#[test]
fn task_blocks_and_unblocks() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "B"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "blocks", "1.1", "1.2"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("bloque"));
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("blocked_by"));
    roadmap()
        .args(["task", "unblocks", "1.1", "1.2"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ne bloque plus"));
}

#[test]
fn task_blocks_cross_phase() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "2", "B"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "T1"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "2", "T2"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "blocks", "1.1", "2.1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("bloque"));
}

// ============================================================================
// task add with optional, tags, files, assignee, due
// ============================================================================

#[test]
fn task_add_with_metadata() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "add", "1", "Deploy", "--optional", "--tag", "infra,ci", "--files", "deploy.sh,Dockerfile", "--assignee", "alice", "--due", "2026-12-31"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"optional\": true"))
        .stdout(predicate::str::contains("infra"))
        .stdout(predicate::str::contains("deploy.sh"))
        .stdout(predicate::str::contains("alice"))
        .stdout(predicate::str::contains("2026-12-31"));
}

// ============================================================================
// task edit with multiple fields
// ============================================================================

#[test]
fn task_edit_multiple_fields() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Old"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "edit", "1.1", "--name", "New", "--description", "Desc", "--optional", "true", "--assignee", "bob"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("modifiée"));
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New"))
        .stdout(predicate::str::contains("bob"));
}

#[test]
fn task_edit_no_args_errors() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "edit", "1.1"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("Spécifie au moins"));
}

// ============================================================================
// subtasks
// ============================================================================

#[test]
fn task_add_subtask() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Parent"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["task", "add", "1", "Enfant", "--parent", "1.1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1.1.1"));
}

// ============================================================================
// workflow
// ============================================================================

fn enable_workflow(dir: &tempfile::TempDir) {
    let config_path = dir.path().join(".phases/config.yml");
    let content = fs::read_to_string(&config_path).unwrap();
    let updated = content.replace("enabled: false", "enabled: true");
    fs::write(&config_path, updated).unwrap();
}

#[test]
fn workflow_advance() {
    let dir = in_tmp();
    enable_workflow(&dir);
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["workflow", "1.1", "--advance"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("analysis"));
}

#[test]
fn workflow_set_stage() {
    let dir = in_tmp();
    enable_workflow(&dir);
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["workflow", "1.1", "--set", "testing"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("testing"));
}

#[test]
fn workflow_set_invalid_stage() {
    let dir = in_tmp();
    enable_workflow(&dir);
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["workflow", "1.1", "--set", "inexistant"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("invalide"));
}

#[test]
fn workflow_no_flag_errors() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["workflow", "1.1"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("--advance"));
}

// ============================================================================
// search
// ============================================================================

#[test]
fn search_finds_phase_and_task() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "Observabilité"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Logs Pino"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["search", "Pino", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Logs Pino"));
    roadmap()
        .args(["search", "observ", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Observabilité"));
}

#[test]
fn search_no_results() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["search", "zzzzinexistant"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("Aucun résultat"));
}

#[test]
fn search_finds_in_notes() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["note", "1", "Décision architecture"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["search", "architecture", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("note"))
        .stdout(predicate::str::contains("architecture"));
}

// ============================================================================
// next
// ============================================================================

#[test]
fn next_shows_pending_tasks() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP", "--status", "in_progress"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Première"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["next", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Première"));
}

#[test]
fn next_empty_when_all_done() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "done", "1.1"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["next", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("null"));
}

// ============================================================================
// context
// ============================================================================

#[test]
fn context_output() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP", "--status", "in_progress"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Setup"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["context"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Contexte Roadmap"))
        .stdout(predicate::str::contains("Setup"));
}

#[test]
fn context_phase_filter() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "2", "Infra"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["context", "--phase", "1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Phase 1"))
        .stdout(predicate::str::contains("MVP"));
}

// ============================================================================
// export
// ============================================================================

#[test]
fn export_generates_roadmap_md() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Setup CI"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["export"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Roadmap exportée"));
    let roadmap_md = fs::read_to_string(dir.path().join("ROADMAP.md")).unwrap();
    assert!(roadmap_md.contains("MVP"));
    assert!(roadmap_md.contains("Setup CI"));
}

// ============================================================================
// template
// ============================================================================

#[test]
fn template_list() {
    let dir = in_tmp();
    roadmap()
        .args(["template", "--list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("feature"))
        .stdout(predicate::str::contains("bug"))
        .stdout(predicate::str::contains("api"))
        .stdout(predicate::str::contains("infra"));
}

#[test]
fn template_create_feature() {
    let dir = in_tmp();
    roadmap()
        .args(["template", "feature", "10"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Phase 10 créée"))
        .stdout(predicate::str::contains("template 'feature'"));
    roadmap()
        .args(["show", "10", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Analyse et spécifications"))
        .stdout(predicate::str::contains("Tests unitaires"));
}

#[test]
fn template_unknown() {
    let dir = in_tmp();
    roadmap()
        .args(["template", "inexistant", "99"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("non trouvé"));
}

// ============================================================================
// log
// ============================================================================

#[test]
fn log_shows_completed_tasks() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Setup"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "done", "1.1"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["log", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("task_done"))
        .stdout(predicate::str::contains("Setup"));
}

#[test]
fn log_empty_roadmap() {
    let dir = in_tmp();
    roadmap()
        .args(["log"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Aucun historique"));
}

#[test]
fn log_includes_notes() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["note", "1", "Une note importante"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["log", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("note"))
        .stdout(predicate::str::contains("Une note importante"));
}

// ============================================================================
// bug (add / list / show / resolve / update)
// ============================================================================

#[test]
fn bug_add_and_list() {
    let dir = in_tmp();
    roadmap()
        .args(["bug", "add", "Crash au démarrage", "--severity", "blocking"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Bug #1 créé"));
    roadmap()
        .args(["bug", "list", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Crash au démarrage"))
        .stdout(predicate::str::contains("blocking"));
}

#[test]
fn bug_show() {
    let dir = in_tmp();
    roadmap()
        .args(["bug", "add", "Memory leak"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["bug", "show", "1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Memory leak"))
        .stdout(predicate::str::contains("Bug #1"));
}

#[test]
fn bug_resolve() {
    let dir = in_tmp();
    roadmap()
        .args(["bug", "add", "Bug fix me"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["bug", "resolve", "1", "--description", "Fixed in v2"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Bug #1 résolu"));
    roadmap()
        .args(["bug", "list", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("resolved"));
}

#[test]
fn bug_update() {
    let dir = in_tmp();
    roadmap()
        .args(["bug", "add", "Old title"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["bug", "update", "1", "--title", "New title", "--severity", "minor"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Bug #1 modifié"));
    roadmap()
        .args(["bug", "show", "1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("New title"))
        .stdout(predicate::str::contains("MINOR"));
}

#[test]
fn bug_invalid_severity() {
    let dir = in_tmp();
    roadmap()
        .args(["bug", "add", "Oops", "--severity", "catastrophic"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("Sévérité invalide"));
}

#[test]
fn bug_show_nonexistent() {
    let dir = in_tmp();
    roadmap()
        .args(["bug", "show", "999"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("non trouvé"));
}

#[test]
fn bug_list_filter_severity() {
    let dir = in_tmp();
    roadmap().args(["bug", "add", "Bug A", "--severity", "blocking"]).current_dir(dir.path()).assert().success();
    roadmap().args(["bug", "add", "Bug B", "--severity", "minor"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["bug", "list", "--severity", "blocking", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Bug A"))
        .stdout(predicate::str::contains("Bug B").not());
}

// ============================================================================
// feature (add / list / show / implement / update)
// ============================================================================

#[test]
fn feature_add_and_list() {
    let dir = in_tmp();
    roadmap()
        .args(["feature", "add", "Dark mode", "--priority", "high"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Feature #1 créée"));
    roadmap()
        .args(["feature", "list", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Dark mode"))
        .stdout(predicate::str::contains("high"));
}

#[test]
fn feature_show() {
    let dir = in_tmp();
    roadmap()
        .args(["feature", "add", "Export PDF"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["feature", "show", "1"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Export PDF"))
        .stdout(predicate::str::contains("Feature #1"));
}

#[test]
fn feature_implement() {
    let dir = in_tmp();
    roadmap()
        .args(["feature", "add", "SSO"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["feature", "implement", "1", "--description", "SAML 2.0"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Feature #1 implémentée"));
    roadmap()
        .args(["feature", "list", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("implemented"));
}

#[test]
fn feature_update() {
    let dir = in_tmp();
    roadmap()
        .args(["feature", "add", "Old feature"])
        .current_dir(dir.path())
        .assert()
        .success();
    roadmap()
        .args(["feature", "update", "1", "--title", "New feature", "--priority", "critical"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Feature #1 modifiée"));
}

#[test]
fn feature_invalid_priority() {
    let dir = in_tmp();
    roadmap()
        .args(["feature", "add", "Test", "--priority", "ultra"])
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("Priorité invalide"));
}

// ============================================================================
// list with filters
// ============================================================================

#[test]
fn list_filter_by_status() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "Done phase", "--status", "done"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "2", "WIP phase", "--status", "in_progress"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["list", "--status", "done", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Done phase"))
        .stdout(predicate::str::contains("WIP phase").not());
}

// ============================================================================
// tree --hide-done
// ============================================================================

#[test]
fn tree_hide_done() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "Done phase", "--status", "done"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "2", "WIP phase", "--status", "in_progress"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["tree", "--hide-done", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("WIP phase"))
        .stdout(predicate::str::contains("Done phase").not());
}

// ============================================================================
// doctor (detailed validation)
// ============================================================================

#[test]
fn doctor_detects_broken_blocks_ref() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "B"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "blocks", "1.1", "1.2"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "remove", "1.1", "-y"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["doctor"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Aucun problème").or(predicate::str::contains("Diagnostic")));
}

#[test]
fn doctor_reports_no_issues_on_clean() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "A"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "done", "1.1"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["doctor"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Aucun problème"));
}

// ============================================================================
// depends-on (phase dependencies)
// ============================================================================

#[test]
fn phase_depends_on() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "Base"]).current_dir(dir.path()).assert().success();
    roadmap().args(["add", "2", "Extension", "--depends-on", "1"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["show", "2", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("depends_on"));
}

// ============================================================================
// report on empty roadmap
// ============================================================================

#[test]
fn report_empty_roadmap() {
    let dir = in_tmp();
    roadmap()
        .args(["report", "--json"])
        .current_dir(dir.path())
        .assert()
        .success();
}

// ============================================================================
// multiple operations integration
// ============================================================================

#[test]
fn full_lifecycle() {
    let dir = in_tmp();
    roadmap().args(["add", "1", "MVP", "--priority", "1", "--status", "in_progress"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Design"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Implement"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "add", "1", "Test"]).current_dir(dir.path()).assert().success();
    roadmap().args(["note", "1", "Kickoff meeting"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "start", "1.1"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "done", "1.1"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "done", "1.2"]).current_dir(dir.path()).assert().success();
    roadmap().args(["task", "done", "1.3"]).current_dir(dir.path()).assert().success();
    roadmap()
        .args(["show", "1", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"done\""));
    roadmap()
        .args(["report", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"phases_done\": 1"));
}

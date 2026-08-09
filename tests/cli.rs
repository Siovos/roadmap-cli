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

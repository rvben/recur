use std::process::Command;

fn ogni() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ogni"))
}

#[test]
fn schema_outputs_valid_json() {
    let output = ogni().arg("schema").output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("schema should output valid JSON");
    assert_eq!(json["name"], "ogni");
    assert!(json["commands"].is_object());
    assert!(json["cron_reference"].is_object());
    assert!(json["exit_codes"].is_object());
}

#[test]
fn schema_contains_all_commands() {
    let output = ogni().arg("schema").output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let commands = json["commands"].as_object().unwrap();
    for cmd in [
        "list",
        "explain",
        "check",
        "timeline",
        "schema",
        "completions",
    ] {
        assert!(commands.contains_key(cmd), "schema missing command: {cmd}");
    }
}

#[test]
fn explain_json_envelope() {
    let output = ogni()
        .args(["explain", "*/5 * * * *", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["expression"], "*/5 * * * *");
    assert_eq!(json["data"]["description"], "every 5 minutes");
}

#[test]
fn explain_fields_filter() {
    let output = ogni()
        .args(["explain", "0 0 * * *", "--json", "--fields", "description"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    assert!(json["data"]["description"].is_string());
    // expression should be filtered out
    assert!(json["data"]["expression"].is_null());
}

#[test]
fn fields_flag_auto_enables_json() {
    let output = ogni()
        .args(["explain", "* * * * *", "--fields", "description"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("--fields should auto-enable JSON output");
    assert_eq!(json["ok"], true);
}

#[test]
fn check_exit_code_zero_when_clean() {
    let output = ogni().args(["check", "--json"]).output().unwrap();
    // Exit 0 = no issues (or no jobs), exit 2 = issues found
    let code = output.status.code().unwrap();
    assert!(code == 0 || code == 2);
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["data"]["issues"].is_array());
}

#[test]
fn check_json_ok_matches_exit_code() {
    let output = ogni().args(["check", "--json"]).output().unwrap();
    let code = output.status.code().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    if code == 0 {
        assert_eq!(json["ok"], true);
    } else if code == 2 {
        assert_eq!(json["ok"], false);
    }
}

#[test]
fn check_quiet_produces_no_output() {
    let output = ogni().args(["check", "--quiet"]).output().unwrap();
    assert!(
        output.stdout.is_empty(),
        "stdout should be empty with --quiet"
    );
}

#[test]
fn check_dry_run_json() {
    let output = ogni()
        .args(["check", "--dry-run", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["dry_run"], true);
    assert!(json["data"]["jobs_to_check"].is_number());
}

#[test]
fn list_json_envelope() {
    let output = ogni().args(["list", "--json"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    assert!(json["data"].is_array());
}

#[test]
fn timeline_json_envelope() {
    let output = ogni()
        .args(["timeline", "--hours", "1", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["hours"], 1);
    assert!(json["data"]["start"].is_string());
    assert!(json["data"]["end"].is_string());
    assert!(json["data"]["events"].is_array());
}

#[test]
fn completions_bash_outputs_something() {
    let output = ogni().args(["completions", "bash"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ogni"),
        "bash completions should reference 'ogni'"
    );
}

#[test]
fn completions_zsh_outputs_something() {
    let output = ogni().args(["completions", "zsh"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ogni"));
}

#[test]
fn version_flag() {
    let output = ogni().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ogni"));
}

#[test]
fn invalid_command_returns_error() {
    let output = ogni().arg("nonexistent").output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn error_json_envelope() {
    // Force a runtime error by requesting a nonexistent user's crontab
    let output = ogni()
        .args(["list", "--user", "nonexistent_user_xyz_12345", "--json"])
        .output()
        .unwrap();
    // Might succeed (empty) or fail depending on system
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // Either ok:true with empty data, or ok:false with error
    assert!(json["ok"].is_boolean());
}

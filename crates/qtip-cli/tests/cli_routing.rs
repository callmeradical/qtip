use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let counter = WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique = format!(
            "qtip-cli-routing-{}-{}-{}",
            std::process::id(),
            counter,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        fs::create_dir_all(&root).expect("create temp workspace");
        Self { root }
    }

    fn manifest_path(&self) -> PathBuf {
        self.root.join("manifest.json")
    }

    fn scenarios_dir(&self) -> PathBuf {
        self.root.join("scenarios")
    }

    fn write_manifest(&self, content: &str) {
        fs::write(self.manifest_path(), content).expect("write manifest");
    }

    fn write_scenario(&self, relative_path: &str, content: &str) {
        let path = self.scenarios_dir().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create scenario parent dirs");
        }
        fs::write(path, content).expect("write scenario");
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn qtip_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_qtip"))
}

fn run_qtip(workspace: &TestWorkspace) -> std::process::Output {
    Command::new(qtip_bin())
        .arg(workspace.manifest_path())
        .arg("--scenarios")
        .arg(workspace.scenarios_dir())
        .output()
        .expect("run qtip binary")
}

fn run_qtip_json(workspace: &TestWorkspace) -> std::process::Output {
    Command::new(qtip_bin())
        .arg(workspace.manifest_path())
        .arg("--scenarios")
        .arg(workspace.scenarios_dir())
        .arg("--output")
        .arg("json")
        .output()
        .expect("run qtip binary with json output")
}

fn manifest_fixture() -> &'static str {
    r#"{
  "projectId": "cli-routing",
  "environment": "local",
  "interfaces": ["cli"],
  "capabilities": ["test"]
}"#
}

fn stdout_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn parse_json_stdout(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

#[test]
fn evaluate_routes_single_and_workflow_scenarios() {
    let workspace = TestWorkspace::new();
    workspace.write_manifest(manifest_fixture());

    workspace.write_scenario(
        "single.yaml",
        r#"
id: TEST-SINGLE-ROUTING-001
name: Single route
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: single scenario still runs
interaction:
  type: cli
  command: echo single-pass
checks:
  - type: status_code
    expected: 0
    acceptance_criteria: AC-1
  - type: stdout
    expected: single-pass
    acceptance_criteria: AC-1
"#,
    );

    workspace.write_scenario(
        "workflow.yaml",
        r#"
id: TEST-WORKFLOW-ROUTING-001
name: Workflow route
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: workflow scenario runs through workflow executor
steps:
  - name: Create loop
    interaction:
      type: cli
      command: echo '{"loop_id":"abc123"}'
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
    outputs:
      loop_id:
        from: json
        path: $.loop_id
  - name: Use output
    interaction:
      type: cli
      command: echo loop:$loop_id
    checks:
      - type: stdout
        expected: loop:abc123
        acceptance_criteria: AC-1
"#,
    );

    let output = run_qtip(&workspace);
    let stdout = stdout_text(&output);
    let stderr = stderr_text(&output);

    assert!(
        output.status.success(),
        "expected success, got status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout,
        stderr
    );
    assert!(stdout.contains("TEST-SINGLE-ROUTING-001"));
    assert!(stdout.contains("TEST-WORKFLOW-ROUTING-001"));
    assert!(stdout.contains("2 passed, 0 failed, 2 total."));
}

#[test]
fn evaluate_preserves_single_scenario_fail_exit_behavior() {
    let workspace = TestWorkspace::new();
    workspace.write_manifest(manifest_fixture());

    workspace.write_scenario(
        "single-fail.yaml",
        r#"
id: TEST-SINGLE-FAIL-001
name: Single fail
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: failure should return non-zero
interaction:
  type: cli
  command: exit 1
checks:
  - type: status_code
    expected: 0
    acceptance_criteria: AC-1
"#,
    );

    let output = run_qtip(&workspace);
    let stdout = stdout_text(&output);
    let stderr = stderr_text(&output);

    assert!(
        !output.status.success(),
        "expected non-zero exit\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(stdout.contains("TEST-SINGLE-FAIL-001"));
    assert!(stdout.contains("FAILED"));
    assert!(stdout.contains("0 passed, 1 failed, 1 total."));
}

#[test]
fn evaluate_invalid_workflow_schema_exits_non_zero_with_parser_details() {
    let workspace = TestWorkspace::new();
    workspace.write_manifest(manifest_fixture());

    workspace.write_scenario(
        "workflow-invalid.yaml",
        r#"
id: TEST-WORKFLOW-INVALID-001
name: Invalid workflow schema
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: parsing should fail with field path
steps:
  - name: Missing interaction step
"#,
    );

    let output = run_qtip(&workspace);
    let stderr = stderr_text(&output);

    assert!(
        !output.status.success(),
        "expected non-zero exit, stderr:\n{}",
        stderr
    );
    assert!(stderr.contains("Failed to load scenarios"));
    assert!(stderr.contains("steps[0].interaction"));
}

#[test]
fn evaluate_json_report_includes_workflow_phase_and_check_details() {
    let workspace = TestWorkspace::new();
    workspace.write_manifest(manifest_fixture());

    workspace.write_scenario(
        "workflow-json.yaml",
        r#"
id: TEST-WORKFLOW-JSON-001
name: Workflow JSON details
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: workflow json includes per-step detail
setup:
  - name: Prepare workspace
    interaction:
      type: cli
      command: echo setup-ready
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
steps:
  - name: Create loop
    interaction:
      type: cli
      command: echo '{"loop_id":"abc123"}'
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
    outputs:
      loop_id:
        from: json
        path: $.loop_id
  - name: Use loop output
    interaction:
      type: cli
      command: echo loop:$loop_id
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
      - type: stdout
        expected: loop:abc123
        acceptance_criteria: AC-1
teardown:
  - name: Cleanup
    interaction:
      type: cli
      command: echo cleanup
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
"#,
    );

    let output = run_qtip_json(&workspace);
    let stdout = stdout_text(&output);
    let stderr = stderr_text(&output);

    assert!(
        output.status.success(),
        "expected success, got status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout,
        stderr
    );

    let report = parse_json_stdout(&output);
    assert_eq!(report["status"], "passed");
    assert!(report["duration_ms"].as_u64().is_some());

    let result = &report["results"][0];
    assert_eq!(result["id"], "TEST-WORKFLOW-JSON-001");
    assert_eq!(result["name"], "Workflow JSON details");
    assert_eq!(result["kind"], "workflow");
    assert_eq!(result["status"], "passed");
    assert!(result["duration_ms"].as_u64().is_some());

    assert_eq!(
        result["setup"]
            .as_array()
            .expect("setup should be an array")
            .len(),
        1
    );
    assert_eq!(
        result["steps"]
            .as_array()
            .expect("steps should be an array")
            .len(),
        2
    );
    assert_eq!(
        result["teardown"]
            .as_array()
            .expect("teardown should be an array")
            .len(),
        1
    );

    let create_loop = &result["steps"][0];
    assert_eq!(create_loop["name"], "Create loop");
    assert_eq!(create_loop["outputs_captured"]["loop_id"], "abc123");
    assert_eq!(create_loop["checks"][0]["type"], "status_code");
    assert_eq!(create_loop["checks"][0]["status"], "passed");

    let use_loop = &result["steps"][1];
    assert_eq!(use_loop["checks"][0]["status"], "passed");
    assert_eq!(use_loop["checks"][1]["type"], "stdout");
    assert_eq!(use_loop["checks"][1]["status"], "passed");
}

#[test]
fn evaluate_json_report_tags_single_without_workflow_arrays() {
    let workspace = TestWorkspace::new();
    workspace.write_manifest(manifest_fixture());

    workspace.write_scenario(
        "single-json.yaml",
        r#"
id: TEST-SINGLE-JSON-001
name: Single JSON details
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: single shape remains flat
interaction:
  type: cli
  command: echo single-pass
checks:
  - type: status_code
    expected: 0
    acceptance_criteria: AC-1
"#,
    );

    let output = run_qtip_json(&workspace);
    let stdout = stdout_text(&output);
    let stderr = stderr_text(&output);
    assert!(
        output.status.success(),
        "expected success, got status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout,
        stderr
    );

    let report = parse_json_stdout(&output);
    let result = &report["results"][0];
    assert_eq!(result["id"], "TEST-SINGLE-JSON-001");
    assert_eq!(result["name"], "Single JSON details");
    assert_eq!(result["kind"], "single");
    assert_eq!(result["status"], "passed");
    assert!(result.get("setup").is_none());
    assert!(result.get("steps").is_none());
    assert!(result.get("teardown").is_none());
}

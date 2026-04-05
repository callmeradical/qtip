use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let unique = format!(
            "qtip-cli-routing-{}-{}",
            std::process::id(),
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

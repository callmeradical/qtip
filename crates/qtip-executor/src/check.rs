use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckType {
    StatusCode,
    JsonPath,
    Stdout,
    Stderr,
    LogContains,
    LogNotContains,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Check {
    #[serde(rename = "type")]
    pub check_type: CheckType,
    pub expected: Option<Value>,
    pub path: Option<String>,
    pub exists: Option<bool>,
    pub acceptance_criteria: String,
}

#[derive(Debug, Clone)]
pub struct Evidence {
    pub status: Option<i64>,
    pub data: Option<Value>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub found: Option<bool>,
}

impl Evidence {
    pub fn api(status: i64, data: Value) -> Self {
        Self {
            status: Some(status),
            data: Some(data),
            stdout: None,
            stderr: None,
            found: None,
        }
    }

    pub fn cli(status: i64, stdout: impl Into<String>, stderr: impl Into<String>) -> Self {
        Self {
            status: Some(status),
            data: None,
            stdout: Some(stdout.into()),
            stderr: Some(stderr.into()),
            found: None,
        }
    }

    pub fn log(found: bool, _content: impl Into<String>) -> Self {
        Self {
            status: None,
            data: None,
            stdout: None,
            stderr: None,
            found: Some(found),
        }
    }
}

pub fn evaluate_checks(checks: &[Check], evidence: &Evidence) -> Vec<String> {
    let mut failures = Vec::new();

    for check in checks {
        match check.check_type {
            CheckType::StatusCode => {
                if let Some(expected) = &check.expected {
                    let expected_status = expected.as_i64();
                    if expected_status != evidence.status {
                        failures.push(format!(
                            "Check status_code failed: expected {}, got {} for AC {}",
                            expected,
                            evidence.status.map_or("none".to_string(), |s| s.to_string()),
                            check.acceptance_criteria,
                        ));
                    }
                }
            }
            CheckType::Stdout => {
                if let (Some(stdout), Some(expected)) = (&evidence.stdout, &check.expected)
                    && let Some(expected_str) = expected.as_str()
                    && !stdout.contains(expected_str)
                {
                    failures.push(format!(
                        "Check stdout failed: stdout does not contain {} for AC {}",
                        expected_str, check.acceptance_criteria,
                    ));
                }
            }
            CheckType::Stderr => {
                if let (Some(stderr), Some(expected)) = (&evidence.stderr, &check.expected)
                    && let Some(expected_str) = expected.as_str()
                    && !stderr.contains(expected_str)
                {
                    failures.push(format!(
                        "Check stderr failed: stderr does not contain {} for AC {}",
                        expected_str, check.acceptance_criteria,
                    ));
                }
            }
            CheckType::LogContains => {
                if evidence.found == Some(false) {
                    failures.push(format!(
                        "Check log_contains failed: log event not found for AC {}",
                        check.acceptance_criteria,
                    ));
                }
            }
            CheckType::LogNotContains => {
                if evidence.found == Some(true) {
                    failures.push(format!(
                        "Check log_not_contains failed: log event found but should not exist for AC {}",
                        check.acceptance_criteria,
                    ));
                }
            }
            CheckType::JsonPath => {
                evaluate_json_path_check(check, evidence, &mut failures);
            }
        }
    }

    failures
}

fn evaluate_json_path_check(check: &Check, evidence: &Evidence, failures: &mut Vec<String>) {
    use jsonpath_rust::JsonPath;

    let Some(path) = &check.path else {
        return;
    };
    let Some(data) = &evidence.data else {
        return;
    };

    let results = match data.query(path) {
        Ok(vals) => vals,
        Err(_) => {
            if check.exists == Some(true) {
                failures.push(format!(
                    "Check json_path failed: path {} not found for AC {}",
                    path, check.acceptance_criteria,
                ));
            }
            return;
        }
    };

    if results.is_empty() {
        if check.exists == Some(true) {
            failures.push(format!(
                "Check json_path failed: path {} not found for AC {}",
                path, check.acceptance_criteria,
            ));
        }
    } else if let Some(expected) = &check.expected
        && results[0] != expected
    {
        failures.push(format!(
            "Check json_path failed: expected {} at {}, got {} for AC {}",
            expected, path, results[0], check.acceptance_criteria,
        ));
    }
}

use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckType {
    StatusCode,
    JsonPath,
    LoopState,
    GithubPrExists,
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
                            evidence
                                .status
                                .map_or("none".to_string(), |s| s.to_string()),
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
            CheckType::LoopState => {
                evaluate_loop_state_check(check, evidence, &mut failures);
            }
            CheckType::GithubPrExists => {
                evaluate_github_pr_exists_check(check, evidence, &mut failures);
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

fn evaluate_loop_state_check(check: &Check, evidence: &Evidence, failures: &mut Vec<String>) {
    let Some(expected) = &check.expected else {
        failures.push(format!(
            "Check loop_state failed: missing expected state for AC {}",
            check.acceptance_criteria
        ));
        return;
    };
    let expected_state = json_value_to_string(expected);

    let payload = match loop_state_json_evidence(evidence) {
        Ok(payload) => payload,
        Err(message) => {
            failures.push(format!(
                "Check loop_state failed: {} for AC {}",
                message, check.acceptance_criteria
            ));
            return;
        }
    };

    let Some(actual) = payload
        .get("state")
        .or_else(|| payload.get("record").and_then(|record| record.get("state")))
    else {
        failures.push(format!(
            "Check loop_state failed: state not found at $.state or $.record.state for AC {}",
            check.acceptance_criteria
        ));
        return;
    };
    let actual_state = json_value_to_string(actual);

    if expected_state != actual_state {
        failures.push(format!(
            "Check loop_state failed: expected `{}`, got `{}` for AC {}",
            expected_state, actual_state, check.acceptance_criteria
        ));
    }
}

#[derive(Default)]
struct GithubPrExistsFilters {
    head_ref_pattern: Option<String>,
    base_ref: Option<String>,
    title_pattern: Option<String>,
    state: Option<String>,
}

impl GithubPrExistsFilters {
    fn from_check(check: &Check) -> Result<Self, String> {
        let Some(raw_filters) = &check.expected else {
            return Ok(Self::default());
        };
        let Some(raw_filters) = raw_filters.as_object() else {
            return Err("expected filter object in `expected`".to_string());
        };

        Ok(Self {
            head_ref_pattern: string_filter(raw_filters, "head_ref_pattern")?,
            base_ref: string_filter(raw_filters, "base_ref")?,
            title_pattern: string_filter(raw_filters, "title_pattern")?,
            state: string_filter(raw_filters, "state")?,
        })
    }

    fn describe(&self) -> String {
        let mut parts = Vec::new();

        if let Some(pattern) = &self.head_ref_pattern {
            parts.push(format!("head_ref_pattern=`{pattern}`"));
        }
        if let Some(base_ref) = &self.base_ref {
            parts.push(format!("base_ref=`{base_ref}`"));
        }
        if let Some(pattern) = &self.title_pattern {
            parts.push(format!("title_pattern=`{pattern}`"));
        }
        if let Some(state) = &self.state {
            parts.push(format!("state=`{state}`"));
        }

        if parts.is_empty() {
            "none".to_string()
        } else {
            parts.join(", ")
        }
    }
}

fn string_filter(
    filters: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, String> {
    match filters.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(format!("`{key}` must be a string")),
    }
}

fn evaluate_github_pr_exists_check(check: &Check, evidence: &Evidence, failures: &mut Vec<String>) {
    let payload = match loop_state_json_evidence(evidence) {
        Ok(payload) => payload,
        Err(message) => {
            failures.push(format!(
                "Check github_pr_exists failed: {} for AC {}",
                message, check.acceptance_criteria
            ));
            return;
        }
    };

    let Some(prs) = payload.as_array() else {
        failures.push(format!(
            "Check github_pr_exists failed: expected JSON array from gh pr list --json, got {} for AC {}",
            json_type_name(&payload),
            check.acceptance_criteria
        ));
        return;
    };

    let filters = match GithubPrExistsFilters::from_check(check) {
        Ok(filters) => filters,
        Err(message) => {
            failures.push(format!(
                "Check github_pr_exists failed: {} for AC {}",
                message, check.acceptance_criteria
            ));
            return;
        }
    };

    let head_ref_regex = match compile_filter_regex(
        filters.head_ref_pattern.as_deref(),
        "head_ref_pattern",
        &check.acceptance_criteria,
        failures,
    ) {
        Some(regex) => regex,
        None => return,
    };
    let title_regex = match compile_filter_regex(
        filters.title_pattern.as_deref(),
        "title_pattern",
        &check.acceptance_criteria,
        failures,
    ) {
        Some(regex) => regex,
        None => return,
    };

    let matched = prs.iter().any(|pr| {
        let Some(pr_object) = pr.as_object() else {
            return false;
        };

        if let Some(pattern) = &head_ref_regex {
            let Some(head_ref_name) = pr_object.get("headRefName").and_then(Value::as_str) else {
                return false;
            };
            if !pattern.is_match(head_ref_name) {
                return false;
            }
        }

        if let Some(base_ref) = &filters.base_ref {
            let Some(actual_base_ref) = pr_object.get("baseRefName").and_then(Value::as_str) else {
                return false;
            };
            if actual_base_ref != base_ref {
                return false;
            }
        }

        if let Some(pattern) = &title_regex {
            let Some(title) = pr_object.get("title").and_then(Value::as_str) else {
                return false;
            };
            if !pattern.is_match(title) {
                return false;
            }
        }

        if let Some(state) = &filters.state {
            let Some(actual_state) = pr_object.get("state").and_then(Value::as_str) else {
                return false;
            };
            if !actual_state.eq_ignore_ascii_case(state) {
                return false;
            }
        }

        true
    });

    if !matched {
        failures.push(format!(
            "Check github_pr_exists failed: no PR matched filters ({}) in {} PR(s) for AC {}",
            filters.describe(),
            prs.len(),
            check.acceptance_criteria
        ));
    }
}

fn compile_filter_regex(
    pattern: Option<&str>,
    filter_name: &str,
    acceptance_criteria: &str,
    failures: &mut Vec<String>,
) -> Option<Option<Regex>> {
    let Some(pattern) = pattern else {
        return Some(None);
    };

    match Regex::new(pattern) {
        Ok(regex) => Some(Some(regex)),
        Err(error) => {
            failures.push(format!(
                "Check github_pr_exists failed: invalid `{}` regex `{}` ({}) for AC {}",
                filter_name, pattern, error, acceptance_criteria
            ));
            None
        }
    }
}

fn loop_state_json_evidence(evidence: &Evidence) -> Result<Value, String> {
    if let Some(data) = &evidence.data {
        return Ok(data.clone());
    }

    if let Some(stdout) = &evidence.stdout
        && !stdout.trim().is_empty()
    {
        return serde_json::from_str(stdout)
            .map_err(|error| format!("failed to parse stdout as JSON: {error}"));
    }

    if let Some(stderr) = &evidence.stderr
        && !stderr.trim().is_empty()
    {
        return serde_json::from_str(stderr)
            .map_err(|error| format!("failed to parse stderr as JSON: {error}"));
    }

    Err("missing JSON evidence in stdout or stderr".to_string())
}

fn json_value_to_string(value: &Value) -> String {
    match value {
        Value::String(inner) => inner.clone(),
        _ => value.to_string(),
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

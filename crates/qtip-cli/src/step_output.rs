use std::collections::HashMap;

use qtip_executor::check::Evidence;
use regex::Regex;
use serde_json::Value;

use crate::scenario::StepOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutputExtractionError {
    pub key: String,
    pub message: String,
}

#[allow(dead_code)]
pub fn extract_step_outputs(
    outputs: &HashMap<String, StepOutput>,
    evidence: &Evidence,
) -> Result<HashMap<String, String>, Vec<StepOutputExtractionError>> {
    let mut extracted = HashMap::with_capacity(outputs.len());
    let mut failures = Vec::new();

    let mut keys: Vec<&str> = outputs.keys().map(String::as_str).collect();
    keys.sort_unstable();

    for key in keys {
        let Some(output) = outputs.get(key) else {
            continue;
        };

        let result = match output.from.as_str() {
            "json" => extract_json_output(output, evidence),
            "stdout" => extract_regex_output(output, evidence.stdout.as_deref()),
            "stderr" => extract_regex_output(output, evidence.stderr.as_deref()),
            // Parser validation rejects unsupported sources before runtime.
            _ => Err(format!("unsupported output source `{}`", output.from)),
        };

        match result {
            Ok(value) => {
                extracted.insert(key.to_string(), value);
            }
            Err(message) => failures.push(StepOutputExtractionError {
                key: key.to_string(),
                message,
            }),
        }
    }

    if failures.is_empty() {
        Ok(extracted)
    } else {
        Err(failures)
    }
}

#[allow(dead_code)]
pub fn merge_step_outputs(
    context: &mut HashMap<String, String>,
    extracted: HashMap<String, String>,
) {
    context.extend(extracted);
}

fn extract_json_output(output: &StepOutput, evidence: &Evidence) -> Result<String, String> {
    use jsonpath_rust::JsonPath;

    let Some(path) = output.path.as_deref() else {
        return Err("missing `path` for json output extraction".to_string());
    };

    let data = json_evidence_value(evidence)?;
    let matches = data
        .query(path)
        .map_err(|_| format!("json path `{path}` was not found"))?;
    let Some(value) = matches.first() else {
        return Err(format!("json path `{path}` was not found"));
    };

    Ok(json_scalar_to_string(value))
}

fn json_evidence_value(evidence: &Evidence) -> Result<Value, String> {
    if let Some(data) = &evidence.data {
        return Ok(data.clone());
    }

    if let Some(stdout) = &evidence.stdout {
        return serde_json::from_str(stdout)
            .map_err(|error| format!("failed to parse stdout as json: {error}"));
    }

    if let Some(stderr) = &evidence.stderr {
        return serde_json::from_str(stderr)
            .map_err(|error| format!("failed to parse stderr as json: {error}"));
    }

    Err("json output extraction requires json evidence".to_string())
}

fn extract_regex_output(output: &StepOutput, text: Option<&str>) -> Result<String, String> {
    let Some(pattern) = output.pattern.as_deref() else {
        return Err("missing `pattern` for regex output extraction".to_string());
    };
    let Some(text) = text else {
        return Err("missing text evidence for regex extraction".to_string());
    };

    let regex = Regex::new(pattern)
        .map_err(|error| format!("invalid regex pattern `{pattern}`: {error}"))?;
    let Some(captures) = regex.captures(text) else {
        return Err(format!(
            "regex pattern `{pattern}` did not match any capture groups"
        ));
    };
    let Some(value) = captures.get(1) else {
        return Err(format!(
            "regex pattern `{pattern}` matched but did not include capture group 1"
        ));
    };

    Ok(value.as_str().to_string())
}

fn json_scalar_to_string(value: &Value) -> String {
    match value {
        Value::String(v) => v.clone(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn json_output(path: &str) -> StepOutput {
        StepOutput {
            from: "json".to_string(),
            path: Some(path.to_string()),
            pattern: None,
        }
    }

    fn stdout_output(pattern: &str) -> StepOutput {
        StepOutput {
            from: "stdout".to_string(),
            path: None,
            pattern: Some(pattern.to_string()),
        }
    }

    fn stderr_output(pattern: &str) -> StepOutput {
        StepOutput {
            from: "stderr".to_string(),
            path: None,
            pattern: Some(pattern.to_string()),
        }
    }

    #[test]
    fn extracts_jsonpath_output_from_cli_json_stdout() {
        let outputs = HashMap::from([("loop_id".to_string(), json_output("$.loop_id"))]);
        let evidence = Evidence::cli(0, r#"{"loop_id":"smi-abc123","state":"synced"}"#, "");

        let extracted = extract_step_outputs(&outputs, &evidence).expect("extraction should pass");

        assert_eq!(
            extracted.get("loop_id").map(String::as_str),
            Some("smi-abc123")
        );
    }

    #[test]
    fn regex_output_without_capture_match_returns_failure_for_key() {
        let outputs = HashMap::from([(
            "loop_id".to_string(),
            stdout_output(r"loop_id=(smi-[a-z0-9]+)"),
        )]);
        let evidence = Evidence::cli(0, "no loop id here", "");

        let failures =
            extract_step_outputs(&outputs, &evidence).expect_err("extraction should fail");

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].key, "loop_id");
        assert!(
            failures[0]
                .message
                .contains("did not match any capture groups")
        );
    }

    #[test]
    fn regex_output_without_capture_group_returns_failure_for_key() {
        let outputs = HashMap::from([("loop_id".to_string(), stdout_output(r"loop_id=smi-\w+"))]);
        let evidence = Evidence::cli(0, "loop_id=smi-abc123", "");

        let failures =
            extract_step_outputs(&outputs, &evidence).expect_err("extraction should fail");

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].key, "loop_id");
        assert!(
            failures[0]
                .message
                .contains("did not include capture group 1")
        );
    }

    #[test]
    fn extracts_multiple_outputs_and_merges_into_context() {
        let outputs = HashMap::from([
            ("loop_id".to_string(), json_output("$.loop_id")),
            ("state".to_string(), json_output("$.record.state")),
        ]);
        let evidence = Evidence::api(
            200,
            json!({"loop_id":"smi-abc123","record":{"state":"synced"}}),
        );
        let extracted = extract_step_outputs(&outputs, &evidence).expect("extraction should pass");

        assert_eq!(
            extracted.get("loop_id").map(String::as_str),
            Some("smi-abc123")
        );
        assert_eq!(extracted.get("state").map(String::as_str), Some("synced"));

        let mut workflow_context = HashMap::from([("existing".to_string(), "value".to_string())]);
        merge_step_outputs(&mut workflow_context, extracted);

        assert_eq!(
            workflow_context.get("existing").map(String::as_str),
            Some("value")
        );
        assert_eq!(
            workflow_context.get("loop_id").map(String::as_str),
            Some("smi-abc123")
        );
        assert_eq!(
            workflow_context.get("state").map(String::as_str),
            Some("synced")
        );
    }

    #[test]
    fn extracts_capture_group_from_stderr_text() {
        let outputs = HashMap::from([("warning_id".to_string(), stderr_output(r"id=(warn-\d+)"))]);
        let evidence = Evidence::cli(1, "", "warning emitted id=warn-42");

        let extracted = extract_step_outputs(&outputs, &evidence).expect("extraction should pass");

        assert_eq!(
            extracted.get("warning_id").map(String::as_str),
            Some("warn-42")
        );
    }
}

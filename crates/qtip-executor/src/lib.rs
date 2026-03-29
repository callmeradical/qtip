#![forbid(unsafe_code)]

pub mod adapters;
pub mod check;
pub mod executor;

pub fn crate_id() -> &'static str {
    "qtip-executor"
}

#[cfg(test)]
mod tests {
    use super::check::*;
    use super::executor::*;
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn crate_id_is_stable() {
        assert_eq!(crate_id(), "qtip-executor");
    }

    // -- status_code checks --

    #[test]
    fn status_code_check_passes_when_status_matches() {
        let checks = vec![Check {
            check_type: CheckType::StatusCode,
            expected: Some(json!(200)),
            path: None,
            exists: None,
            acceptance_criteria: "AC-1".to_string(),
        }];
        let evidence = Evidence::api(200, json!({}));

        let failures = evaluate_checks(&checks, &evidence);
        assert!(failures.is_empty());
    }

    #[test]
    fn status_code_check_fails_when_status_differs() {
        let checks = vec![Check {
            check_type: CheckType::StatusCode,
            expected: Some(json!(200)),
            path: None,
            exists: None,
            acceptance_criteria: "AC-1".to_string(),
        }];
        let evidence = Evidence::api(404, json!({}));

        let failures = evaluate_checks(&checks, &evidence);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("expected 200"));
        assert!(failures[0].contains("got 404"));
        assert!(failures[0].contains("AC-1"));
    }

    // -- stdout checks --

    #[test]
    fn stdout_check_passes_when_output_contains_expected() {
        let checks = vec![Check {
            check_type: CheckType::Stdout,
            expected: Some(json!("hello world")),
            path: None,
            exists: None,
            acceptance_criteria: "AC-2".to_string(),
        }];
        let evidence = Evidence::cli(0, "output: hello world\n", "");

        let failures = evaluate_checks(&checks, &evidence);
        assert!(failures.is_empty());
    }

    #[test]
    fn stdout_check_fails_when_output_missing_expected() {
        let checks = vec![Check {
            check_type: CheckType::Stdout,
            expected: Some(json!("success")),
            path: None,
            exists: None,
            acceptance_criteria: "AC-2".to_string(),
        }];
        let evidence = Evidence::cli(0, "error occurred", "");

        let failures = evaluate_checks(&checks, &evidence);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("stdout does not contain"));
        assert!(failures[0].contains("AC-2"));
    }

    // -- stderr checks --

    #[test]
    fn stderr_check_passes_when_output_contains_expected() {
        let checks = vec![Check {
            check_type: CheckType::Stderr,
            expected: Some(json!("warning")),
            path: None,
            exists: None,
            acceptance_criteria: "AC-3".to_string(),
        }];
        let evidence = Evidence::cli(0, "", "warning: deprecated");

        let failures = evaluate_checks(&checks, &evidence);
        assert!(failures.is_empty());
    }

    #[test]
    fn stderr_check_fails_when_output_missing_expected() {
        let checks = vec![Check {
            check_type: CheckType::Stderr,
            expected: Some(json!("critical")),
            path: None,
            exists: None,
            acceptance_criteria: "AC-3".to_string(),
        }];
        let evidence = Evidence::cli(0, "", "warning: minor issue");

        let failures = evaluate_checks(&checks, &evidence);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("stderr does not contain"));
    }

    // -- log_contains checks --

    #[test]
    fn log_contains_passes_when_event_found() {
        let checks = vec![Check {
            check_type: CheckType::LogContains,
            expected: None,
            path: None,
            exists: None,
            acceptance_criteria: "AC-4".to_string(),
        }];
        let evidence = Evidence::log(true, "matched line");

        let failures = evaluate_checks(&checks, &evidence);
        assert!(failures.is_empty());
    }

    #[test]
    fn log_contains_fails_when_event_not_found() {
        let checks = vec![Check {
            check_type: CheckType::LogContains,
            expected: None,
            path: None,
            exists: None,
            acceptance_criteria: "AC-4".to_string(),
        }];
        let evidence = Evidence::log(false, "");

        let failures = evaluate_checks(&checks, &evidence);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("log event not found"));
    }

    // -- log_not_contains checks --

    #[test]
    fn log_not_contains_passes_when_event_absent() {
        let checks = vec![Check {
            check_type: CheckType::LogNotContains,
            expected: None,
            path: None,
            exists: None,
            acceptance_criteria: "AC-5".to_string(),
        }];
        let evidence = Evidence::log(false, "");

        let failures = evaluate_checks(&checks, &evidence);
        assert!(failures.is_empty());
    }

    #[test]
    fn log_not_contains_fails_when_event_present() {
        let checks = vec![Check {
            check_type: CheckType::LogNotContains,
            expected: None,
            path: None,
            exists: None,
            acceptance_criteria: "AC-5".to_string(),
        }];
        let evidence = Evidence::log(true, "bad event");

        let failures = evaluate_checks(&checks, &evidence);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("should not exist"));
    }

    // -- json_path checks --

    #[test]
    fn json_path_exists_passes_when_path_found() {
        let checks = vec![Check {
            check_type: CheckType::JsonPath,
            expected: None,
            path: Some("$.token".to_string()),
            exists: Some(true),
            acceptance_criteria: "AC-6".to_string(),
        }];
        let evidence = Evidence::api(200, json!({"token": "abc123"}));

        let failures = evaluate_checks(&checks, &evidence);
        assert!(failures.is_empty());
    }

    #[test]
    fn json_path_exists_fails_when_path_missing() {
        let checks = vec![Check {
            check_type: CheckType::JsonPath,
            expected: None,
            path: Some("$.token".to_string()),
            exists: Some(true),
            acceptance_criteria: "AC-6".to_string(),
        }];
        let evidence = Evidence::api(200, json!({"user": "demo"}));

        let failures = evaluate_checks(&checks, &evidence);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("not found"));
        assert!(failures[0].contains("$.token"));
    }

    #[test]
    fn json_path_expected_value_passes_when_matching() {
        let checks = vec![Check {
            check_type: CheckType::JsonPath,
            expected: Some(json!(200)),
            path: Some("$.status".to_string()),
            exists: None,
            acceptance_criteria: "AC-7".to_string(),
        }];
        let evidence = Evidence::api(200, json!({"status": 200}));

        let failures = evaluate_checks(&checks, &evidence);
        assert!(failures.is_empty());
    }

    #[test]
    fn json_path_expected_value_fails_when_different() {
        let checks = vec![Check {
            check_type: CheckType::JsonPath,
            expected: Some(json!(200)),
            path: Some("$.status".to_string()),
            exists: None,
            acceptance_criteria: "AC-7".to_string(),
        }];
        let evidence = Evidence::api(500, json!({"status": 500}));

        let failures = evaluate_checks(&checks, &evidence);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("expected 200"));
        assert!(failures[0].contains("got 500"));
    }

    // -- multiple checks --

    #[test]
    fn multiple_checks_collect_all_failures() {
        let checks = vec![
            Check {
                check_type: CheckType::StatusCode,
                expected: Some(json!(200)),
                path: None,
                exists: None,
                acceptance_criteria: "AC-1".to_string(),
            },
            Check {
                check_type: CheckType::JsonPath,
                expected: None,
                path: Some("$.token".to_string()),
                exists: Some(true),
                acceptance_criteria: "AC-2".to_string(),
            },
        ];
        let evidence = Evidence::api(401, json!({"error": "unauthorized"}));

        let failures = evaluate_checks(&checks, &evidence);
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn all_checks_pass_returns_empty_failures() {
        let checks = vec![
            Check {
                check_type: CheckType::StatusCode,
                expected: Some(json!(200)),
                path: None,
                exists: None,
                acceptance_criteria: "AC-1".to_string(),
            },
            Check {
                check_type: CheckType::JsonPath,
                expected: None,
                path: Some("$.token".to_string()),
                exists: Some(true),
                acceptance_criteria: "AC-2".to_string(),
            },
        ];
        let evidence = Evidence::api(200, json!({"token": "abc123"}));

        let failures = evaluate_checks(&checks, &evidence);
        assert!(failures.is_empty());
    }

    // -- executor orchestration --

    struct StubAdapter {
        interaction_type: String,
        evidence: Result<Evidence, String>,
    }

    impl Adapter for StubAdapter {
        fn interaction_type(&self) -> &str {
            &self.interaction_type
        }

        fn execute<'a>(
            &'a self,
            _interaction: &'a Interaction,
        ) -> BoxFuture<'a, Result<Evidence, String>> {
            Box::pin(async { self.evidence.clone() })
        }
    }

    fn api_scenario_with_checks(checks: Vec<Check>) -> ExecutableScenario {
        ExecutableScenario {
            id: "TEST-001".to_string(),
            name: "Test scenario".to_string(),
            interaction: Interaction {
                interaction_type: "api".to_string(),
                params: HashMap::new(),
            },
            checks,
        }
    }

    #[tokio::test]
    async fn executor_returns_passed_when_all_checks_pass() {
        let mut executor = ScenarioExecutor::new();
        executor.register_adapter(Box::new(StubAdapter {
            interaction_type: "api".to_string(),
            evidence: Ok(Evidence::api(200, json!({"token": "abc"}))),
        }));

        let scenario = api_scenario_with_checks(vec![Check {
            check_type: CheckType::StatusCode,
            expected: Some(json!(200)),
            path: None,
            exists: None,
            acceptance_criteria: "AC-1".to_string(),
        }]);

        let result = executor.execute(&scenario).await;
        assert_eq!(result.status, EvaluationStatus::Passed);
        assert!(result.failures.is_empty());
        assert_eq!(result.scenario_id, "TEST-001");
    }

    #[tokio::test]
    async fn executor_returns_failed_when_check_fails() {
        let mut executor = ScenarioExecutor::new();
        executor.register_adapter(Box::new(StubAdapter {
            interaction_type: "api".to_string(),
            evidence: Ok(Evidence::api(500, json!({}))),
        }));

        let scenario = api_scenario_with_checks(vec![Check {
            check_type: CheckType::StatusCode,
            expected: Some(json!(200)),
            path: None,
            exists: None,
            acceptance_criteria: "AC-1".to_string(),
        }]);

        let result = executor.execute(&scenario).await;
        assert_eq!(result.status, EvaluationStatus::Failed);
        assert_eq!(result.failures.len(), 1);
    }

    #[tokio::test]
    async fn executor_returns_error_when_adapter_fails() {
        let mut executor = ScenarioExecutor::new();
        executor.register_adapter(Box::new(StubAdapter {
            interaction_type: "api".to_string(),
            evidence: Err("connection refused".to_string()),
        }));

        let scenario = api_scenario_with_checks(vec![]);

        let result = executor.execute(&scenario).await;
        assert_eq!(result.status, EvaluationStatus::Error);
        assert!(result.failures[0].contains("connection refused"));
    }

    #[tokio::test]
    async fn executor_returns_error_for_unsupported_interaction_type() {
        let executor = ScenarioExecutor::new();

        let scenario = ExecutableScenario {
            id: "TEST-002".to_string(),
            name: "UI scenario".to_string(),
            interaction: Interaction {
                interaction_type: "ui".to_string(),
                params: HashMap::new(),
            },
            checks: vec![],
        };

        let result = executor.execute(&scenario).await;
        assert_eq!(result.status, EvaluationStatus::Error);
        assert!(result.failures[0].contains("Unsupported interaction type: ui"));
    }

    #[tokio::test]
    async fn executor_dispatches_to_correct_adapter() {
        let mut executor = ScenarioExecutor::new();
        executor.register_adapter(Box::new(StubAdapter {
            interaction_type: "api".to_string(),
            evidence: Ok(Evidence::api(200, json!({}))),
        }));
        executor.register_adapter(Box::new(StubAdapter {
            interaction_type: "cli".to_string(),
            evidence: Ok(Evidence::cli(0, "hello", "")),
        }));

        let cli_scenario = ExecutableScenario {
            id: "CLI-001".to_string(),
            name: "CLI test".to_string(),
            interaction: Interaction {
                interaction_type: "cli".to_string(),
                params: HashMap::new(),
            },
            checks: vec![Check {
                check_type: CheckType::Stdout,
                expected: Some(json!("hello")),
                path: None,
                exists: None,
                acceptance_criteria: "AC-1".to_string(),
            }],
        };

        let result = executor.execute(&cli_scenario).await;
        assert_eq!(result.status, EvaluationStatus::Passed);
    }

    // -- CLI adapter integration tests --

    mod cli_adapter_tests {
        use super::*;
        use crate::adapters::cli::CliAdapter;

        fn cli_interaction(command: &str) -> Interaction {
            let mut params = HashMap::new();
            params.insert(
                "command".to_string(),
                serde_json::Value::String(command.to_string()),
            );
            Interaction {
                interaction_type: "cli".to_string(),
                params,
            }
        }

        #[tokio::test]
        async fn successful_command_returns_zero_exit_and_stdout() {
            let adapter = CliAdapter::new();
            let interaction = cli_interaction("echo hello world");

            let evidence = adapter.execute(&interaction).await.unwrap();
            assert_eq!(evidence.status, Some(0));
            assert!(evidence.stdout.as_ref().unwrap().contains("hello world"));
        }

        #[tokio::test]
        async fn failing_command_returns_nonzero_exit_code() {
            let adapter = CliAdapter::new();
            let interaction = cli_interaction("sh -c 'exit 42'");

            let evidence = adapter.execute(&interaction).await.unwrap();
            assert_eq!(evidence.status, Some(42));
        }

        #[tokio::test]
        async fn stderr_is_captured() {
            let adapter = CliAdapter::new();
            let interaction = cli_interaction("echo error >&2");

            let evidence = adapter.execute(&interaction).await.unwrap();
            assert!(evidence.stderr.as_ref().unwrap().contains("error"));
        }

        #[tokio::test]
        async fn missing_command_field_returns_error() {
            let adapter = CliAdapter::new();
            let interaction = Interaction {
                interaction_type: "cli".to_string(),
                params: HashMap::new(),
            };

            let result = adapter.execute(&interaction).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("missing 'command'"));
        }

        #[tokio::test]
        async fn timeout_returns_error() {
            let adapter = CliAdapter::with_timeout(1);
            let interaction = cli_interaction("sleep 10");

            let result = adapter.execute(&interaction).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("timed out"));
        }
    }

    // -- Log adapter integration tests --

    mod log_adapter_tests {
        use super::*;
        use crate::adapters::log::LogAdapter;
        use std::io::Write;

        fn log_interaction(query: &str, log_path: &str) -> Interaction {
            let mut params = HashMap::new();
            params.insert(
                "query".to_string(),
                serde_json::Value::String(query.to_string()),
            );
            params.insert(
                "log_path".to_string(),
                serde_json::Value::String(log_path.to_string()),
            );
            Interaction {
                interaction_type: "logs".to_string(),
                params,
            }
        }

        #[tokio::test]
        async fn finds_matching_line_in_log_file() {
            let mut tmp = tempfile::NamedTempFile::new().unwrap();
            writeln!(tmp, "INFO started").unwrap();
            writeln!(tmp, "ERROR connection refused").unwrap();
            writeln!(tmp, "INFO request complete").unwrap();

            let adapter = LogAdapter::new();
            let interaction =
                log_interaction("ERROR", tmp.path().to_str().unwrap());

            let evidence = adapter.execute(&interaction).await.unwrap();
            assert_eq!(evidence.found, Some(true));
        }

        #[tokio::test]
        async fn returns_not_found_when_no_match() {
            let mut tmp = tempfile::NamedTempFile::new().unwrap();
            writeln!(tmp, "INFO all good").unwrap();

            let adapter = LogAdapter::new();
            let interaction =
                log_interaction("ERROR", tmp.path().to_str().unwrap());

            let evidence = adapter.execute(&interaction).await.unwrap();
            assert_eq!(evidence.found, Some(false));
        }

        #[tokio::test]
        async fn returns_not_found_for_missing_file() {
            let adapter = LogAdapter::new();
            let interaction = log_interaction("ERROR", "/tmp/nonexistent-qtip-test.log");

            let evidence = adapter.execute(&interaction).await.unwrap();
            assert_eq!(evidence.found, Some(false));
        }

        #[tokio::test]
        async fn missing_query_returns_error() {
            let adapter = LogAdapter::new();
            let mut params = HashMap::new();
            params.insert(
                "log_path".to_string(),
                serde_json::Value::String("/tmp/test.log".to_string()),
            );
            let interaction = Interaction {
                interaction_type: "logs".to_string(),
                params,
            };

            let result = adapter.execute(&interaction).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("missing 'query'"));
        }
    }
}

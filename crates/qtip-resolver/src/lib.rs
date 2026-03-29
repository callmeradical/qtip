#![forbid(unsafe_code)]

use serde::Deserialize;

/// Returns the crate name used by downstream integration tests and wiring checks.
pub fn crate_id() -> &'static str {
    "qtip-resolver"
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AppliesTo {
    pub capabilities: Vec<String>,
    pub interfaces: Vec<String>,
    #[serde(default)]
    pub environments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ScenarioManifest {
    pub id: String,
    pub name: String,
    pub applies_to: AppliesTo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectInterface {
    pub interface_type: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectQuery {
    pub capabilities: Vec<String>,
    pub interfaces: Vec<SubjectInterface>,
    pub environment: String,
}

pub struct ScenarioResolver {
    scenarios: Vec<ScenarioManifest>,
}

impl ScenarioResolver {
    pub fn new(scenarios: Vec<ScenarioManifest>) -> Self {
        Self { scenarios }
    }

    pub fn resolve(&self, query: &SubjectQuery) -> Vec<&ScenarioManifest> {
        self.scenarios
            .iter()
            .filter(|scenario| self.matches(scenario, query))
            .collect()
    }

    fn matches(&self, scenario: &ScenarioManifest, query: &SubjectQuery) -> bool {
        let has_capability = scenario
            .applies_to
            .capabilities
            .iter()
            .any(|cap| query.capabilities.contains(cap));
        if !has_capability {
            return false;
        }

        let query_interface_types: Vec<&str> = query
            .interfaces
            .iter()
            .map(|i| i.interface_type.as_str())
            .collect();
        let has_interface = scenario
            .applies_to
            .interfaces
            .iter()
            .any(|i| query_interface_types.contains(&i.as_str()));
        if !has_interface {
            return false;
        }

        if !scenario.applies_to.environments.is_empty()
            && !scenario
                .applies_to
                .environments
                .contains(&query.environment)
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_id_is_stable() {
        assert_eq!(crate_id(), "qtip-resolver");
    }

    fn auth_api_scenario() -> ScenarioManifest {
        ScenarioManifest {
            id: "AUTH-LOGIN-001".to_string(),
            name: "Valid login returns token".to_string(),
            applies_to: AppliesTo {
                capabilities: vec!["auth".to_string()],
                interfaces: vec!["api".to_string()],
                environments: vec![],
            },
        }
    }

    fn billing_cli_scenario() -> ScenarioManifest {
        ScenarioManifest {
            id: "BILLING-EXPORT-001".to_string(),
            name: "Export billing report".to_string(),
            applies_to: AppliesTo {
                capabilities: vec!["billing".to_string()],
                interfaces: vec!["cli".to_string()],
                environments: vec![],
            },
        }
    }

    fn staging_only_scenario() -> ScenarioManifest {
        ScenarioManifest {
            id: "AUTH-STAGING-001".to_string(),
            name: "Staging auth smoke test".to_string(),
            applies_to: AppliesTo {
                capabilities: vec!["auth".to_string()],
                interfaces: vec!["api".to_string()],
                environments: vec!["staging".to_string()],
            },
        }
    }

    fn query_with_auth_api() -> SubjectQuery {
        SubjectQuery {
            capabilities: vec!["auth".to_string()],
            interfaces: vec![SubjectInterface {
                interface_type: "api".to_string(),
                name: None,
            }],
            environment: "production".to_string(),
        }
    }

    // -- Capability matching --

    #[test]
    fn matches_scenario_with_overlapping_capability() {
        let resolver = ScenarioResolver::new(vec![auth_api_scenario()]);
        let results = resolver.resolve(&query_with_auth_api());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "AUTH-LOGIN-001");
    }

    #[test]
    fn excludes_scenario_with_no_capability_overlap() {
        let resolver = ScenarioResolver::new(vec![billing_cli_scenario()]);
        let results = resolver.resolve(&query_with_auth_api());

        assert!(results.is_empty());
    }

    // -- Interface matching --

    #[test]
    fn excludes_scenario_with_no_interface_overlap() {
        let resolver = ScenarioResolver::new(vec![billing_cli_scenario()]);
        let query = SubjectQuery {
            capabilities: vec!["billing".to_string()],
            interfaces: vec![SubjectInterface {
                interface_type: "api".to_string(),
                name: None,
            }],
            environment: "production".to_string(),
        };
        let results = resolver.resolve(&query);

        assert!(results.is_empty());
    }

    #[test]
    fn matches_scenario_with_overlapping_interface() {
        let resolver = ScenarioResolver::new(vec![billing_cli_scenario()]);
        let query = SubjectQuery {
            capabilities: vec!["billing".to_string()],
            interfaces: vec![SubjectInterface {
                interface_type: "cli".to_string(),
                name: None,
            }],
            environment: "production".to_string(),
        };
        let results = resolver.resolve(&query);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "BILLING-EXPORT-001");
    }

    // -- Environment filtering --

    #[test]
    fn scenario_with_no_environments_matches_any_environment() {
        let resolver = ScenarioResolver::new(vec![auth_api_scenario()]);
        let query = SubjectQuery {
            capabilities: vec!["auth".to_string()],
            interfaces: vec![SubjectInterface {
                interface_type: "api".to_string(),
                name: None,
            }],
            environment: "whatever".to_string(),
        };
        let results = resolver.resolve(&query);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn scenario_with_environment_constraint_excludes_non_matching_environment() {
        let resolver = ScenarioResolver::new(vec![staging_only_scenario()]);
        let query = SubjectQuery {
            capabilities: vec!["auth".to_string()],
            interfaces: vec![SubjectInterface {
                interface_type: "api".to_string(),
                name: None,
            }],
            environment: "production".to_string(),
        };
        let results = resolver.resolve(&query);

        assert!(results.is_empty());
    }

    #[test]
    fn scenario_with_environment_constraint_matches_correct_environment() {
        let resolver = ScenarioResolver::new(vec![staging_only_scenario()]);
        let query = SubjectQuery {
            capabilities: vec!["auth".to_string()],
            interfaces: vec![SubjectInterface {
                interface_type: "api".to_string(),
                name: None,
            }],
            environment: "staging".to_string(),
        };
        let results = resolver.resolve(&query);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "AUTH-STAGING-001");
    }

    // -- Multi-scenario resolution --

    #[test]
    fn resolves_subset_from_mixed_scenarios() {
        let resolver = ScenarioResolver::new(vec![
            auth_api_scenario(),
            billing_cli_scenario(),
            staging_only_scenario(),
        ]);
        let results = resolver.resolve(&query_with_auth_api());

        // auth_api matches (capability + interface, no env constraint)
        // billing_cli excluded (wrong capability)
        // staging_only excluded (environment mismatch — production != staging)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "AUTH-LOGIN-001");
    }

    #[test]
    fn empty_scenarios_returns_empty() {
        let resolver = ScenarioResolver::new(vec![]);
        let results = resolver.resolve(&query_with_auth_api());

        assert!(results.is_empty());
    }

    #[test]
    fn multiple_capabilities_match_if_any_overlap() {
        let multi_cap = ScenarioManifest {
            id: "MULTI-001".to_string(),
            name: "Multi-capability scenario".to_string(),
            applies_to: AppliesTo {
                capabilities: vec!["auth".to_string(), "billing".to_string()],
                interfaces: vec!["api".to_string()],
                environments: vec![],
            },
        };
        let resolver = ScenarioResolver::new(vec![multi_cap]);
        let results = resolver.resolve(&query_with_auth_api());

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn multiple_interfaces_in_query_match_if_any_overlap() {
        let resolver = ScenarioResolver::new(vec![billing_cli_scenario()]);
        let query = SubjectQuery {
            capabilities: vec!["billing".to_string()],
            interfaces: vec![
                SubjectInterface {
                    interface_type: "api".to_string(),
                    name: None,
                },
                SubjectInterface {
                    interface_type: "cli".to_string(),
                    name: None,
                },
            ],
            environment: "production".to_string(),
        };
        let results = resolver.resolve(&query);

        assert_eq!(results.len(), 1);
    }
}

use std::collections::{BTreeSet, HashMap};

/// Shared substitution context used by scenario preparation.
/// Environment values have higher precedence than step outputs.
#[derive(Debug, Clone)]
pub struct ResolveContext<'a> {
    env: &'a HashMap<String, String>,
    outputs: &'a HashMap<String, String>,
}

impl<'a> ResolveContext<'a> {
    pub fn new(env: &'a HashMap<String, String>, outputs: &'a HashMap<String, String>) -> Self {
        Self { env, outputs }
    }

    fn lookup(&self, key: &str) -> Option<&str> {
        self.env
            .get(key)
            .map(String::as_str)
            .or_else(|| self.outputs.get(key).map(String::as_str))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    unresolved: Vec<String>,
}

impl ResolveError {
    pub fn from_set(unresolved: BTreeSet<String>) -> Self {
        Self {
            unresolved: unresolved.into_iter().collect(),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn unresolved(&self) -> &[String] {
        &self.unresolved
    }
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unresolved variables: {}", self.unresolved.join(", "))
    }
}

impl std::error::Error for ResolveError {}

#[cfg_attr(not(test), allow(dead_code))]
pub fn resolve_template(
    template: &str,
    context: &ResolveContext<'_>,
) -> Result<String, ResolveError> {
    let mut unresolved = BTreeSet::new();
    let rendered = resolve_template_inner(template, context, &mut unresolved);

    if unresolved.is_empty() {
        Ok(rendered)
    } else {
        Err(ResolveError::from_set(unresolved))
    }
}

pub fn resolve_json_value(
    value: &serde_json::Value,
    context: &ResolveContext<'_>,
) -> Result<serde_json::Value, ResolveError> {
    let mut unresolved = BTreeSet::new();
    let resolved = resolve_json_value_inner(value, context, &mut unresolved);

    if unresolved.is_empty() {
        Ok(resolved)
    } else {
        Err(ResolveError::from_set(unresolved))
    }
}

fn resolve_json_value_inner(
    value: &serde_json::Value,
    context: &ResolveContext<'_>,
    unresolved: &mut BTreeSet<String>,
) -> serde_json::Value {
    match value {
        serde_json::Value::String(template) => {
            serde_json::Value::String(resolve_template_inner(template, context, unresolved))
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(|item| resolve_json_value_inner(item, context, unresolved))
                .collect(),
        ),
        serde_json::Value::Object(map) => {
            let mut resolved = serde_json::Map::new();
            for (key, value) in map {
                resolved.insert(
                    key.clone(),
                    resolve_json_value_inner(value, context, unresolved),
                );
            }
            serde_json::Value::Object(resolved)
        }
        _ => value.clone(),
    }
}

fn resolve_template_inner(
    template: &str,
    context: &ResolveContext<'_>,
    unresolved: &mut BTreeSet<String>,
) -> String {
    let chars: Vec<char> = template.chars().collect();
    let mut output = String::with_capacity(template.len());
    let mut cursor = 0;

    while cursor < chars.len() {
        let current = chars[cursor];
        if current != '$' {
            output.push(current);
            cursor += 1;
            continue;
        }

        if cursor + 1 < chars.len() && chars[cursor + 1] == '$' {
            output.push('$');
            cursor += 2;
            continue;
        }

        if cursor + 1 < chars.len() && is_variable_start(chars[cursor + 1]) {
            let mut end = cursor + 2;
            while end < chars.len() && is_variable_char(chars[end]) {
                end += 1;
            }

            let name: String = chars[cursor + 1..end].iter().collect();
            if let Some(value) = context.lookup(&name) {
                output.push_str(value);
            } else {
                unresolved.insert(name.clone());
                output.push('$');
                output.push_str(&name);
            }
            cursor = end;
            continue;
        }

        output.push('$');
        cursor += 1;
    }

    output
}

fn is_variable_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_variable_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{ResolveContext, resolve_json_value, resolve_template};

    #[test]
    fn resolve_template_prefers_environment_values_over_outputs() {
        let env = HashMap::from([("TEST_REPO".to_string(), "org/repo".to_string())]);
        let outputs = HashMap::from([("TEST_REPO".to_string(), "from-output".to_string())]);
        let context = ResolveContext::new(&env, &outputs);

        let rendered =
            resolve_template("smith-integration cleanup --repo $TEST_REPO", &context).unwrap();
        assert_eq!(rendered, "smith-integration cleanup --repo org/repo");
    }

    #[test]
    fn resolve_template_reports_all_unresolved_variables() {
        let env = HashMap::new();
        let outputs = HashMap::new();
        let context = ResolveContext::new(&env, &outputs);

        let error =
            resolve_template("echo $loop_id then $build_id then $loop_id", &context).unwrap_err();
        assert_eq!(
            error.unresolved(),
            &["build_id".to_string(), "loop_id".to_string()]
        );
    }

    #[test]
    fn resolve_template_treats_double_dollar_as_escape() {
        let env = HashMap::from([("HOME".to_string(), "/tmp/home".to_string())]);
        let outputs = HashMap::new();
        let context = ResolveContext::new(&env, &outputs);

        let rendered = resolve_template("echo $$HOME", &context).unwrap();
        assert_eq!(rendered, "echo $HOME");
    }

    #[test]
    fn resolve_json_value_updates_nested_string_fields() {
        let env = HashMap::from([
            ("TOKEN".to_string(), "abc123".to_string()),
            ("PATH_SUFFIX".to_string(), "/health".to_string()),
        ]);
        let outputs = HashMap::new();
        let context = ResolveContext::new(&env, &outputs);

        let input = serde_json::json!({
            "request": {
                "path": "$PATH_SUFFIX",
                "headers": {
                    "Authorization": "Bearer $TOKEN"
                }
            }
        });

        let resolved = resolve_json_value(&input, &context).unwrap();
        assert_eq!(resolved["request"]["path"], "/health");
        assert_eq!(
            resolved["request"]["headers"]["Authorization"],
            "Bearer abc123"
        );
    }
}

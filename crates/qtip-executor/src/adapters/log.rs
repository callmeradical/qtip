use crate::check::Evidence;
use crate::executor::{Adapter, BoxFuture, Interaction};

pub struct LogAdapter;

impl LogAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Adapter for LogAdapter {
    fn interaction_type(&self) -> &str {
        "logs"
    }

    fn execute<'a>(
        &'a self,
        interaction: &'a Interaction,
    ) -> BoxFuture<'a, Result<Evidence, String>> {
        Box::pin(async move {
            let query = interaction
                .params
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Log interaction missing 'query' field".to_string())?;

            let log_path = interaction
                .params
                .get("log_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Log interaction missing 'log_path' field".to_string())?;

            let path = std::path::Path::new(log_path);
            if !path.exists() {
                return Ok(Evidence::log(false, ""));
            }

            let file = tokio::fs::File::open(path)
                .await
                .map_err(|e| format!("Failed to open log file: {e}"))?;

            let reader = tokio::io::BufReader::new(file);
            let mut matched_lines = Vec::new();

            use tokio::io::AsyncBufReadExt;
            let mut lines = reader.lines();
            while let Some(line) = lines
                .next_line()
                .await
                .map_err(|e| format!("Failed to read log line: {e}"))?
            {
                if line.contains(query) {
                    matched_lines.push(line);
                }
            }

            let found = !matched_lines.is_empty();
            let content = matched_lines.join("\n");
            Ok(Evidence::log(found, content))
        })
    }
}

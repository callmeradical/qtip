use crate::check::Evidence;
use crate::executor::{Adapter, BoxFuture, Interaction};

pub struct CliAdapter {
    timeout_secs: u64,
}

impl CliAdapter {
    pub fn new() -> Self {
        Self { timeout_secs: 30 }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }
}

impl Adapter for CliAdapter {
    fn interaction_type(&self) -> &str {
        "cli"
    }

    fn execute<'a>(
        &'a self,
        interaction: &'a Interaction,
    ) -> BoxFuture<'a, Result<Evidence, String>> {
        Box::pin(async move {
            let command = interaction
                .params
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "CLI interaction missing 'command' field".to_string())?;

            let result = tokio::time::timeout(
                std::time::Duration::from_secs(self.timeout_secs),
                tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output(),
            )
            .await;

            match result {
                Ok(Ok(output)) => {
                    let status = output.status.code().unwrap_or(1) as i64;
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    Ok(Evidence::cli(status, stdout, stderr))
                }
                Ok(Err(err)) => Err(format!("Failed to execute command: {err}")),
                Err(_) => Err(format!(
                    "Command timed out after {} seconds",
                    self.timeout_secs
                )),
            }
        })
    }
}

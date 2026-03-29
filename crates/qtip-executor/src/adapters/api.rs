use crate::check::Evidence;
use crate::executor::{Adapter, BoxFuture, Interaction};

pub struct ApiAdapter {
    client: reqwest::Client,
}

impl ApiAdapter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }
}

impl Adapter for ApiAdapter {
    fn interaction_type(&self) -> &str {
        "api"
    }

    fn execute<'a>(
        &'a self,
        interaction: &'a Interaction,
    ) -> BoxFuture<'a, Result<Evidence, String>> {
        Box::pin(async move {
            let method = interaction
                .params
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET");

            let url = interaction
                .params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "API interaction missing 'url' field".to_string())?;

            let mut request = match method.to_uppercase().as_str() {
                "GET" => self.client.get(url),
                "POST" => self.client.post(url),
                "PUT" => self.client.put(url),
                "DELETE" => self.client.delete(url),
                "PATCH" => self.client.patch(url),
                other => return Err(format!("Unsupported HTTP method: {other}")),
            };

            if let Some(body) = interaction.params.get("body") {
                request = request.json(body);
            }

            if let Some(headers) = interaction.params.get("headers").and_then(|v| v.as_object()) {
                for (key, value) in headers {
                    if let Some(val) = value.as_str() {
                        request = request.header(key.as_str(), val);
                    }
                }
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("HTTP request failed: {e}"))?;

            let status = response.status().as_u16() as i64;
            let data: serde_json::Value = response
                .json()
                .await
                .unwrap_or(serde_json::Value::Null);

            Ok(Evidence::api(status, data))
        })
    }
}

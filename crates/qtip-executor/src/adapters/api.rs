use std::time::Duration;

use crate::check::Evidence;
use crate::executor::{Adapter, BoxFuture, Interaction, TIMEOUT_OVERRIDE_PARAM};

type Headers = serde_json::Map<String, serde_json::Value>;

struct RequestParts {
    method: String,
    url: String,
    body: Option<serde_json::Value>,
    headers: Option<Headers>,
}

pub struct ApiAdapter {
    client: reqwest::Client,
    max_retries: u32,
    initial_backoff: Duration,
}

impl Default for ApiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiAdapter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            max_retries: 2,
            initial_backoff: Duration::from_millis(500),
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            max_retries: 2,
            initial_backoff: Duration::from_millis(500),
        }
    }

    pub fn with_retries(mut self, max_retries: u32, initial_backoff: Duration) -> Self {
        self.max_retries = max_retries;
        self.initial_backoff = initial_backoff;
        self
    }

    fn build_request(&self, interaction: &Interaction) -> Result<RequestParts, String> {
        let method = interaction
            .params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_uppercase();

        let url = interaction
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "API interaction missing 'url' field".to_string())?
            .to_string();

        let body = interaction.params.get("body").cloned();
        let headers = interaction
            .params
            .get("headers")
            .and_then(|v| v.as_object())
            .cloned();

        match method.as_str() {
            "GET" | "POST" | "PUT" | "DELETE" | "PATCH" => {}
            other => return Err(format!("Unsupported HTTP method: {other}")),
        }

        Ok(RequestParts {
            method,
            url,
            body,
            headers,
        })
    }

    async fn send_request(
        &self,
        method: &str,
        url: &str,
        body: &Option<serde_json::Value>,
        headers: &Option<Headers>,
        timeout_secs: Option<u64>,
    ) -> Result<Evidence, reqwest::Error> {
        let mut request = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            _ => self.client.get(url),
        };
        if let Some(timeout_secs) = timeout_secs {
            request = request.timeout(Duration::from_secs(timeout_secs));
        }

        if let Some(body) = body {
            request = request.json(body);
        }

        if let Some(headers) = headers {
            for (key, value) in headers {
                if let Some(val) = value.as_str() {
                    request = request.header(key.as_str(), val);
                }
            }
        }

        let response = request.send().await?;

        let status = response.status().as_u16() as i64;
        let data: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);

        Ok(Evidence::api(status, data))
    }

    fn is_retryable(err: &reqwest::Error) -> bool {
        err.is_timeout() || err.is_connect() || err.is_request()
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
            let RequestParts {
                method,
                url,
                body,
                headers,
            } = self.build_request(interaction)?;
            let timeout_override_secs = interaction
                .params
                .get(TIMEOUT_OVERRIDE_PARAM)
                .and_then(|value| value.as_u64());

            let mut last_err = String::new();
            for attempt in 0..=self.max_retries {
                if attempt > 0 {
                    let backoff = self.initial_backoff * 2u32.pow(attempt - 1);
                    tokio::time::sleep(backoff).await;
                }

                match self
                    .send_request(&method, &url, &body, &headers, timeout_override_secs)
                    .await
                {
                    Ok(evidence) => return Ok(evidence),
                    Err(err) => {
                        last_err = format!("HTTP request failed: {err}");
                        if !Self::is_retryable(&err) {
                            return Err(last_err);
                        }
                    }
                }
            }

            Err(format!("{last_err} (after {} retries)", self.max_retries))
        })
    }
}

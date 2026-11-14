use serde_json::{Value, json};
use std::time::Duration;

pub const CATALOG_SOURCE: &str = "https://api.venice.ai/api/v1/models?type=image";
pub struct Provider {
    client: reqwest::Client,
    base: String,
    pub key: Option<String>,
}
impl Provider {
    pub fn new(key: Option<String>, fixture: Option<String>) -> Result<Self, String> {
        let base: String = if let Some(base) = fixture {
            #[cfg(not(feature = "test-support"))]
            {
                let _ = base;
                return Err("Fixture provider is unavailable in this build.".into());
            }
            #[cfg(feature = "test-support")]
            {
                let url = reqwest::Url::parse(&base).map_err(|_| "Invalid fixture address.")?;
                if url.scheme() != "http"
                    || url.host_str() != Some("127.0.0.1")
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
                    || url.path() != "/"
                {
                    return Err(
                        "Fixture provider must be a plain IPv4 loopback HTTP origin.".into(),
                    );
                }
                base.trim_end_matches('/').to_owned()
            }
        } else {
            "https://api.venice.ai/api/v1".into()
        };
        let key = if base.starts_with("http:") {
            if std::env::var("VENICE_STUDIO_TEST_MISSING_CREDENTIAL").as_deref() == Ok("1") {
                None
            } else {
                Some("offline-test".into())
            }
        } else {
            key
        };
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(300))
            .no_proxy()
            .build()
            .map_err(|_| "Cannot initialize provider transport.")?;
        Ok(Self { client, base, key })
    }
    pub async fn catalog(&self) -> Result<Value, String> {
        let response = self
            .client
            .get(format!("{}/models?type=image", self.base))
            .timeout(Duration::from_secs(20))
            .send()
            .await
            .map_err(
                |_| "Catalog refresh could not reach Venice; retained catalog is unchanged.",
            )?;
        if !response.status().is_success() {
            return Err("Venice rejected catalog refresh; retained catalog is unchanged.".into());
        }
        let bytes = bounded(response, 4 * 1024 * 1024).await?;
        let v: Value = serde_json::from_slice(&bytes).map_err(|_| "Catalog was not valid JSON.")?;
        if !v.get("data").is_some_and(Value::is_array) {
            return Err("Catalog has an unknown shape.".into());
        }
        Ok(
            json!({"fetched_at":chrono::Utc::now().to_rfc3339(),"source":CATALOG_SOURCE,"models":self.redact_metadata(&v["data"])}),
        )
    }
    pub async fn styles(&self) -> Result<Value, String> {
        let response = self
            .client
            .get(format!("{}/image/styles", self.base))
            .timeout(Duration::from_secs(20))
            .send()
            .await
            .map_err(|_| "Styles refresh could not reach Venice; cached styles are unchanged.")?;
        if !response.status().is_success() {
            return Err("Venice rejected styles refresh; cached styles are unchanged.".into());
        }
        let bytes = bounded(response, 4 * 1024 * 1024).await?;
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|_| "Styles were not valid JSON.")?;
        if !value["data"]
            .as_array()
            .is_some_and(|a| a.iter().all(Value::is_string))
        {
            return Err("Styles have an unknown shape; cached styles are unchanged.".into());
        }
        Ok(
            json!({"fetched_at":chrono::Utc::now().to_rfc3339(),"source":"https://api.venice.ai/api/v1/image/styles","data":self.redact_metadata(&value["data"])}),
        )
    }
    pub async fn generate(&self, request: &Value) -> Result<(u16, Value, Value), String> {
        let key = self.key.as_deref().ok_or(
            "Set VENICE_API_KEY in the private .env or process environment and restart Studio.",
        )?;
        let response = self.client.post(format!("{}/image/generate",self.base)).bearer_auth(key).json(request).send().await.map_err(|_| "Connection interrupted or timed out. Outcome and billing are unknown; no retry was sent.")?;
        let status = response.status().as_u16();
        let mut headers = serde_json::Map::new();
        for name in [
            "x-request-id",
            "x-venice-id",
            "x-content-policy-violation-count",
            "x-content-policy-violation",
            "x-venice-is-content-policy-violation",
            "x-venice-is-blurred",
            "x-venice-is-content-violation",
            "x-venice-content-violation-count",
            "x-venice-enhanced-prompt",
            "retry-after",
        ] {
            if let Some(value) = response.headers().get(name).and_then(|x| x.to_str().ok()) {
                headers.insert(
                    name.into(),
                    Value::String(value.chars().take(8192).collect()),
                );
                if value.chars().count() > 8192 {
                    headers.insert(format!("{name}-capture-truncated"), json!(true));
                }
            }
        }
        let body = match bounded(response, 192 * 1024 * 1024).await {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_|json!({"raw_text":String::from_utf8_lossy(&bytes[..bytes.len().min(32768)]),"capture_truncated":bytes.len()>32768})),
            Err(message) => json!({"capture_error":message,"body_incomplete":true}),
        };
        Ok((status, body, Value::Object(headers)))
    }
    #[cfg(feature = "test-support")]
    pub fn is_fixture(&self) -> bool {
        self.base.starts_with("http:")
    }
    fn redact_metadata(&self, value: &Value) -> Value {
        match value {
            Value::Array(a) => Value::Array(a.iter().map(|v| self.redact_metadata(v)).collect()),
            Value::Object(o) => Value::Object(
                o.iter()
                    .map(|(key, value)| {
                        let redacted_key = self
                            .redact(&Value::String(key.clone()))
                            .as_str()
                            .unwrap()
                            .to_owned();
                        let value = if [
                            "authorization",
                            "api_key",
                            "access_token",
                            "token",
                            "secret",
                        ]
                        .contains(&key.to_lowercase().as_str())
                        {
                            json!("[redacted]")
                        } else {
                            self.redact_metadata(value)
                        };
                        (redacted_key, value)
                    })
                    .collect(),
            ),
            v => self.redact(v),
        }
    }
    pub fn redact(&self, value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(
                self.key
                    .as_ref()
                    .filter(|s| !s.is_empty())
                    .map_or_else(|| s.clone(), |key| s.replace(key, "[redacted]")),
            ),
            Value::Array(a) => {
                let mut values: Vec<Value> = a.iter().take(128).map(|v| self.redact(v)).collect();
                if a.len() > 128 {
                    values.push(json!({"capture_truncated":true,"omitted_items":a.len()-128}));
                }
                Value::Array(values)
            }
            Value::Object(o) => Value::Object(
                o.iter()
                    .filter(|(k, _)| !matches!(k.as_str(), "images" | "image" | "b64_json"))
                    .map(|(k, v)| {
                        (
                            self.redact(&Value::String(k.clone()))
                                .as_str()
                                .unwrap()
                                .to_owned(),
                            if [
                                "authorization",
                                "api_key",
                                "access_token",
                                "token",
                                "secret",
                            ]
                            .contains(&k.to_lowercase().as_str())
                            {
                                json!("[redacted]")
                            } else {
                                self.redact(v)
                            },
                        )
                    })
                    .collect(),
            ),
            v => v.clone(),
        }
    }
}
async fn bounded(mut response: reqwest::Response, max: usize) -> Result<Vec<u8>, String> {
    if response.content_length().is_some_and(|n| n > max as u64) {
        return Err(
            "Provider body exceeds the capture limit. Outcome/billing unknown; no retry sent."
                .into(),
        );
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Provider body incomplete. Outcome/billing unknown; no retry sent.")?
    {
        if bytes.len() + chunk.len() > max {
            return Err(
                "Provider body exceeds the capture limit. Outcome/billing unknown; no retry sent."
                    .into(),
            );
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

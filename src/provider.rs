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
        let (bytes, read_error) = capture(response, 192 * 1024 * 1024).await;
        let mut body = match serde_json::from_slice::<Value>(&bytes) {
            Ok(body) => body,
            Err(error) => {
                // A native JSON response can end after complete originals but before
                // the next item. Serde's grammar determines complete values; never
                // infer missing base64 bytes or scan arbitrary strings for images.
                let mut recovered = serde_json::Map::new();
                let mut decoder = serde_json::Deserializer::from_slice(&bytes);
                let _ = serde::de::Deserializer::deserialize_map(
                    &mut decoder,
                    ImageResponsePrefix(&mut recovered),
                );
                if recovered.contains_key("images") {
                    recovered.insert("body_incomplete".into(), json!(true));
                    recovered.insert("json_error".into(), json!(error.to_string()));
                    recovered.insert("capture_note".into(), json!("Only complete JSON members retained; incomplete image payload text is omitted."));
                    Value::Object(recovered)
                } else {
                    json!({"raw_text":String::from_utf8_lossy(&bytes[..bytes.len().min(32768)]),"capture_truncated":bytes.len()>32768})
                }
            }
        };
        if let Some(error) = read_error {
            if !body.is_object() {
                body = json!({"captured_value":body});
            }
            body["capture_error"] = json!(error);
            body["body_incomplete"] = json!(true);
            body["transport_incomplete"] = json!(true);
        }
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
                        let value = if sensitive_field(key) {
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
                    .map(|(k, v)| {
                        (
                            self.redact(&Value::String(k.clone()))
                                .as_str()
                                .unwrap()
                                .to_owned(),
                            if sensitive_field(k) {
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
fn sensitive_field(key: &str) -> bool {
    [
        "authorization",
        "api_key",
        "access_token",
        "token",
        "secret",
    ]
    .contains(&key.to_lowercase().as_str())
}
// This visitor recovers only complete top-level native response members and
// complete elements of its images array. It is not a repair parser.
struct ImageResponsePrefix<'a>(&'a mut serde_json::Map<String, Value>);
impl<'de> serde::de::Visitor<'de> for ImageResponsePrefix<'_> {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a native image response object")
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        while let Some(key) = map.next_key::<String>()? {
            if key == "images" {
                let mut images = Vec::new();
                let result = map.next_value_seed(ImageArrayPrefix(&mut images));
                self.0.insert(key, Value::Array(images));
                result?;
            } else {
                self.0.insert(key, map.next_value::<Value>()?);
            }
        }
        Ok(())
    }
}
struct ImageArrayPrefix<'a>(&'a mut Vec<Value>);
impl<'de> serde::de::DeserializeSeed<'de> for ImageArrayPrefix<'_> {
    type Value = ();
    fn deserialize<D: serde::de::Deserializer<'de>>(self, decoder: D) -> Result<(), D::Error> {
        decoder.deserialize_seq(self)
    }
}
impl<'de> serde::de::Visitor<'de> for ImageArrayPrefix<'_> {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("the native images array")
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut array: A) -> Result<(), A::Error> {
        while let Some(value) = array.next_element::<Value>()? {
            self.0.push(value);
        }
        Ok(())
    }
}
async fn bounded(response: reqwest::Response, max: usize) -> Result<Vec<u8>, String> {
    let (bytes, error) = capture(response, max).await;
    error.map_or(Ok(bytes), |e| Err(e.into()))
}
async fn capture(mut response: reqwest::Response, max: usize) -> (Vec<u8>, Option<&'static str>) {
    if response.content_length().is_some_and(|n| n > max as u64) {
        return (
            Vec::new(),
            Some(
                "Provider body exceeds the capture limit. Outcome/billing unknown; no retry sent.",
            ),
        );
    }
    let mut bytes = Vec::new();
    loop {
        let chunk = match response.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => return (bytes, None),
            Err(_) => {
                return (
                    bytes,
                    Some("Provider body incomplete. Outcome/billing unknown; no retry sent."),
                );
            }
        };
        if bytes.len() + chunk.len() > max {
            bytes.extend_from_slice(&chunk[..max - bytes.len()]);
            return (
                bytes,
                Some(
                    "Provider body exceeds the capture limit. Outcome/billing unknown; no retry sent.",
                ),
            );
        }
        bytes.extend_from_slice(&chunk);
    }
}

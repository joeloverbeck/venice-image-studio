//! Interpret the native image response using structured contract evidence only.
use serde_json::{Value, json};

pub struct Outcome {
    pub kind: &'static str,
    pub summary: String,
    pub counts: Value,
    pub billing: Value,
}

pub fn interpret(
    status: u16,
    body: &Value,
    headers: &Value,
    requested: u64,
    valid: usize,
    invalid: usize,
) -> Outcome {
    let body_omitted = body["content_policy_violation_count"].as_u64();
    let header_omitted = headers["x-venice-content-violation-count"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok());
    let conflict = body_omitted
        .zip(header_omitted)
        .is_some_and(|(a, b)| a != b);
    let omitted = if conflict {
        None
    } else {
        body_omitted.or(header_omitted)
    };
    let incomplete = body["body_incomplete"] == true;
    let counts = json!({"requested":requested,"returned_items":body["images"].as_array().map(Vec::len),"retained_originals":valid,"invalid_items":invalid,"provider_omitted":omitted,"omission_count_conflict":conflict});
    let mut billing = json!({"charge":"unknown","refund":"unknown"});
    let mut assertions = serde_json::Map::new();
    for (scope, source) in [("response", body), ("error", &body["error"])] {
        for key in [
            "charge",
            "charged",
            "amount_charged",
            "credits_refunded",
            "refund",
            "refunded",
            "billing",
            "usage",
            "cost",
        ] {
            if let Some(value) = source.get(key) {
                let encoded = serde_json::to_string(value).unwrap();
                let captured = if encoded.len() > 4096 {
                    json!({"capture_truncated":true,"text":encoded.chars().take(1000).collect::<String>()})
                } else {
                    value.clone()
                };
                assertions.insert(
                    if scope == "response" {
                        key.to_owned()
                    } else {
                        format!("error.{key}")
                    },
                    captured,
                );
            }
        }
    }
    let billing_summary = if assertions.is_empty() {
        " Billing and refund are unknown."
    } else {
        " Provider supplied billing/refund assertions (unverified); inspect details. Actual billing remains unknown."
    };
    if !assertions.is_empty() {
        billing["provider_assertions"] = Value::Object(assertions);
    }
    let (kind, mut summary) = if valid > 0 {
        let kind = if invalid > 0
            || valid as u64 != requested
            || omitted.is_some_and(|n| n > 0)
            || conflict
            || incomplete
            || !(200..300).contains(&status)
        {
            "partial"
        } else {
            "received"
        };
        let mut summary = format!("Received {valid} of {requested} requested originals.");
        if invalid > 0 {
            summary.push_str(&format!(
                " {invalid} items could not be retained as valid images."
            ));
        }
        if let Some(omitted) = omitted {
            summary.push_str(&format!(" Venice reports {omitted} omitted by provider content policy. No more specific reason is inferred; inspect provider details."));
        } else if conflict {
            summary.push_str(
                " Provider omission counts conflict; inspect the separate body and header values.",
            );
        } else if valid as u64 != requested {
            summary.push_str(" No usable provider omission reason or count is available.");
        }
        if incomplete {
            summary.push_str(" The response was incomplete. Only complete recovered originals are available; the remaining outcome is unknown.");
        }
        if !(200..300).contains(&status) {
            summary.push_str(&format!(
                " Provider also returned HTTP {status}; inspect details."
            ));
        }
        (kind, summary)
    } else {
        // Native ImageContentViolationError documents Boolean retryable specifically
        // on policy 422s, absent on other 422 errors. Error prose is never classified.
        let code = body["code"]
            .as_str()
            .or_else(|| body["error"]["code"].as_str());
        let policy = status == 422
            && (body["retryable"].is_boolean() || code == Some("CONTENT_POLICY_VIOLATION"));
        let fields = body["details"]["_errors"].is_array() || code == Some("IMAGE_TOO_LARGE");
        let (kind, summary) = if incomplete {
            (
                "indeterminate",
                "Provider body incomplete or exceeded capture bounds. Remote outcome is unknown; no retry was sent.",
            )
        } else {
            match status {
                401 | 403 => (
                    "access_error",
                    "Venice rejected access. Check the private API key and account permissions, then restart Studio.",
                ),
                402 if code == Some("INSUFFICIENT_BALANCE") => (
                    "balance_error",
                    "Venice reports insufficient balance. Check account balance before making another explicit attempt.",
                ),
                402 if matches!(
                    code,
                    Some("API_KEY_DIEM_SPEND_LIMIT_EXCEEDED" | "API_KEY_USD_SPEND_LIMIT_EXCEEDED")
                ) =>
                {
                    (
                        "key_limit_error",
                        "Venice reports that this API key reached its spend limit. This does not establish that the account balance is empty.",
                    )
                }
                402 => (
                    "payment_error",
                    "Venice requires payment or account access. Inspect details for balance, key limits or an authentication challenge; an empty balance is not assumed.",
                ),
                429 => (
                    "rate_limited",
                    "Venice rate limited the request. Inspect retry timing; no retry was sent.",
                ),
                422 if policy && fields => (
                    "rejected",
                    "Venice supplied conflicting policy and settings indicators. Inspect provider details; no category is assumed.",
                ),
                422 if policy => (
                    "refused",
                    "Venice reports a provider content-policy refusal. Inspect its explanation and retry advice. Nothing was changed or retried.",
                ),
                400 | 413 | 415 => (
                    "invalid_settings",
                    "Venice rejected request settings or size. Inspect field details; no policy reason is inferred.",
                ),
                422 if fields => (
                    "invalid_settings",
                    "Venice rejected request settings. Inspect field details; no policy reason is inferred.",
                ),
                422 => (
                    "rejected",
                    "Venice rejected the request. Inspect provider details; a policy reason is not assumed.",
                ),
                500..=599 => (
                    "provider_error",
                    "Venice returned a provider failure. No retry was sent.",
                ),
                _ => (
                    "failed",
                    "No valid image was received. Inspect provider details.",
                ),
            }
        };
        (kind, summary.to_owned())
    };
    if valid == 0 && !has_explanation(body) && !incomplete {
        summary.push_str(" No specific reason was supplied.");
    }
    summary.push_str(billing_summary);
    Outcome {
        kind,
        summary,
        counts,
        billing,
    }
}

pub fn has_explanation(body: &Value) -> bool {
    if body.as_str().is_some_and(|s| !s.trim().is_empty()) {
        return true;
    }
    [
        "/reason",
        "/error",
        "/message",
        "/raw_text",
        "/error/message",
        "/error/reason",
    ]
    .iter()
    .any(|pointer| {
        body.pointer(pointer)
            .and_then(Value::as_str)
            .is_some_and(|s| !s.trim().is_empty())
    }) || ["/details", "/issues", "/error/details", "/error/issues"]
        .iter()
        .any(|pointer| body.pointer(pointer).is_some_and(nonempty_detail))
}
fn nonempty_detail(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(s) => !s.trim().is_empty(),
        Value::Array(values) => values.iter().any(nonempty_detail),
        Value::Object(fields) => fields.values().any(nonempty_detail),
        _ => true,
    }
}

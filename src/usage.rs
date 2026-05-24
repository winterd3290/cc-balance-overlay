use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UsageScript {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default, alias = "apiKey")]
    pub api_key: Option<String>,
    #[serde(default)]
    pub auto_query_interval: Option<u64>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BalanceSnapshot {
    pub remaining: Option<f64>,
    pub unit: String,
    pub valid: bool,
    pub message: Option<String>,
}

impl BalanceSnapshot {
    pub fn unknown(message: impl Into<String>) -> Self {
        Self {
            remaining: None,
            unit: String::new(),
            valid: false,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub timeout_secs: u64,
}

impl BalanceRequest {
    pub fn redacted_headers(&self) -> Vec<(String, String)> {
        self.headers
            .iter()
            .map(|(name, value)| {
                if is_sensitive_header(name) {
                    (name.clone(), format!("<redacted len={}>", value.len()))
                } else {
                    (name.clone(), value.clone())
                }
            })
            .collect()
    }
}

pub fn build_balance_request(script: &UsageScript) -> Result<BalanceRequest> {
    let base_url = script
        .base_url
        .as_deref()
        .context("usage script does not define baseUrl")?
        .trim_end_matches('/');
    let endpoint = infer_endpoint(&script.code).unwrap_or_else(|| "/v1/usage".to_string());
    let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint
    } else {
        format!("{base_url}{endpoint}")
    };

    let mut headers = Vec::new();
    headers.push(("Content-Type".to_string(), "application/json".to_string()));
    if let Some(key) = script.api_key.as_deref() {
        headers.push(("Authorization".to_string(), format!("Bearer {key}")));
    }
    if let Some(token) = script.access_token.as_deref() {
        headers.push(("Authorization".to_string(), format!("Bearer {token}")));
    }
    if let Some(user_id) = script.user_id.as_deref() {
        headers.push(("New-Api-User".to_string(), user_id.to_string()));
    }

    Ok(BalanceRequest {
        url,
        headers,
        timeout_secs: script.timeout.unwrap_or(10),
    })
}

pub fn query_balance(script: &UsageScript) -> Result<BalanceSnapshot> {
    let req = build_balance_request(script)?;
    let mut headers = HeaderMap::new();
    for (name, value) in &req.headers {
        let name = HeaderName::from_bytes(name.as_bytes())
            .with_context(|| format!("invalid header name {name}"))?;
        let value = HeaderValue::from_str(value).context("invalid header value")?;
        if name == CONTENT_TYPE && headers.contains_key(CONTENT_TYPE) {
            continue;
        }
        headers.insert(name, value);
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(req.timeout_secs))
        .default_headers(headers)
        .build()?;
    let response = client.get(&req.url).send()?;
    let status = response.status();
    let body: Value = response.json().unwrap_or(Value::Null);
    if !status.is_success() {
        return Ok(BalanceSnapshot::unknown(format!(
            "balance endpoint returned {status}"
        )));
    }
    Ok(extract_balance(&body))
}

pub fn extract_balance(value: &Value) -> BalanceSnapshot {
    let data = value.get("data").unwrap_or(value);
    let remaining = first_number(data, &["remaining", "balance", "quota_remaining"])
        .or_else(|| data.get("quota").and_then(|q| first_number(q, &["remaining", "balance"])))
        .or_else(|| first_number(data, &["quota"]).map(normalize_quota));
    let unit = first_string(data, &["unit", "currency"])
        .or_else(|| {
            data.get("quota")
                .and_then(|q| first_string(q, &["unit", "currency"]))
        })
        .unwrap_or_else(|| "USD".to_string());
    let valid = first_bool(data, &["is_active", "isValid", "valid"]).unwrap_or(true);

    BalanceSnapshot {
        remaining,
        unit,
        valid,
        message: None,
    }
}

fn normalize_quota(value: f64) -> f64 {
    if value.abs() > 100_000.0 {
        value / 500_000.0
    } else {
        value
    }
}

fn infer_endpoint(code: &str) -> Option<String> {
    let marker = "{{baseUrl}}";
    let idx = code.find(marker)?;
    let rest = &code[idx + marker.len()..];
    let quote_idx = rest.find(['"', '\'', '`']).unwrap_or(rest.len());
    let endpoint = &rest[..quote_idx];
    if endpoint.starts_with('/') {
        Some(endpoint.to_string())
    } else {
        None
    }
}

fn first_number(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .filter_map(|key| value.get(*key))
        .find_map(|v| match v {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        })
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| value.get(*key))
        .find_map(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
}

fn first_bool(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .filter_map(|key| value.get(*key))
        .find_map(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
}

fn is_sensitive_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "authorization" | "x-api-key" | "api-key" | "new-api-key"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_codex_usage_request_from_usage_script() {
        let script: UsageScript = serde_json::from_value(json!({
            "enabled": true,
            "baseUrl": "https://api.example.test",
            "apiKey": "TEST_SECRET",
            "timeout": 7,
            "code": "({ request: { url: \"{{baseUrl}}/v1/usage\", method: \"GET\" } })"
        }))
        .unwrap();

        let req = build_balance_request(&script).unwrap();

        assert_eq!(req.url, "https://api.example.test/v1/usage");
        assert_eq!(req.timeout_secs, 7);
        assert!(req
            .redacted_headers()
            .iter()
            .any(|(k, v)| k == "Authorization" && v.starts_with("<redacted")));
    }

    #[test]
    fn extracts_remaining_from_common_response_shapes() {
        let direct = extract_balance(&json!({"remaining": "12.5", "unit": "CNY"}));
        let nested = extract_balance(&json!({"quota": {"remaining": 7.0, "unit": "USD"}}));
        let normalized = extract_balance(&json!({"data": {"quota": 8_863_627, "used_quota": 77_136_373}}));

        assert_eq!(direct.remaining, Some(12.5));
        assert_eq!(direct.unit, "CNY");
        assert_eq!(nested.remaining, Some(7.0));
        assert_eq!(normalized.remaining, Some(17.727254));
    }
}

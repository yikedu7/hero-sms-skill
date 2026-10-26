//! Poll for activation OTP (default receive path — no webhook required).

use crate::{
    apply_auth, build_url, choose_base_url, execute_request, index_operations, AuthStyle,
    QueryParams,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollMode {
    /// SMS-Activate `getStatusV2` (JSON).
    SaV2,
    /// SMS-Activate `getStatus` (text `STATUS_OK:code`).
    SaV1,
    /// REST `getActivationOtpLast`.
    RestLast,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StatusParse {
    Waiting,
    Received(String),
    Cancelled,
    Failed(String),
}

#[derive(Debug, Serialize)]
pub struct WaitOtpSuccess {
    pub activation_id: String,
    pub code: String,
    pub mode: String,
    pub polls: u32,
    pub elapsed_seconds: u64,
}

/// Parse SMS-Activate text status (`getStatus`).
pub fn parse_sa_status_text(body: &str) -> StatusParse {
    let trimmed = body.trim();
    if trimmed.starts_with("STATUS_OK:") {
        let code = trimmed.strip_prefix("STATUS_OK:").unwrap_or("").trim();
        if !code.is_empty() {
            return StatusParse::Received(code.to_string());
        }
    }
    if trimmed == "STATUS_CANCEL" {
        return StatusParse::Cancelled;
    }
    if trimmed.starts_with("STATUS_WAIT") {
        return StatusParse::Waiting;
    }
    StatusParse::Failed(trimmed.to_string())
}

/// Parse `getStatusV2` JSON or string fallbacks.
pub fn parse_sa_status_v2(body: &str) -> StatusParse {
    let trimmed = body.trim();
    if trimmed == "STATUS_CANCEL" {
        return StatusParse::Cancelled;
    }
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        if let Some(code) = value
            .get("data")
            .and_then(|d| d.get("code"))
            .and_then(|c| c.as_str())
        {
            if !code.is_empty() {
                return StatusParse::Received(code.to_string());
            }
        }
        return StatusParse::Waiting;
    }
    parse_sa_status_text(body)
}

/// Parse REST `getActivationOtpLast` JSON (`data.code` or top-level patterns).
pub fn parse_rest_otp_last(body: &str) -> StatusParse {
    let value: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return StatusParse::Failed(e.to_string()),
    };
    if let Some(code) = value
        .get("data")
        .and_then(|d| d.get("code"))
        .and_then(|c| c.as_str())
    {
        if !code.is_empty() {
            return StatusParse::Received(code.to_string());
        }
    }
    StatusParse::Waiting
}

fn invoke_read(
    spec: &Value,
    operation_id: &str,
    activation_id: &str,
    api_key: &str,
    auth: AuthStyle,
) -> Result<String, String> {
    let ops = index_operations(spec);
    let op = ops
        .get(operation_id)
        .ok_or_else(|| format!("missing operation {operation_id}"))?;

    let mut path_params = std::collections::HashMap::new();
    let mut query: QueryParams = BTreeMap::new();
    if operation_id == "getActivationOtpLast" {
        path_params.insert("activationId".to_string(), activation_id.to_string());
    } else {
        query.insert("id".to_string(), vec![activation_id.to_string()]);
    }

    let mut headers = BTreeMap::new();
    headers.insert("Accept".to_string(), "application/json".to_string());
    apply_auth(op, spec, &mut headers, &mut query, api_key, auth);

    let url = build_url(&choose_base_url(op), &op.path, &path_params, &query);
    let resp = execute_request(&op.method, &url, &headers, None)?;
    if resp.status < 200 || resp.status >= 300 {
        return Err(format!("HTTP {}: {}", resp.status, resp.body));
    }
    Ok(resp.body)
}

pub fn poll_until_otp(
    spec: &Value,
    activation_id: &str,
    api_key: &str,
    auth: AuthStyle,
    mode: PollMode,
    interval: Duration,
    timeout: Duration,
) -> Result<WaitOtpSuccess, String> {
    let (op_id, mode_name) = match mode {
        PollMode::SaV2 => ("getStatusV2", "sa-v2"),
        PollMode::SaV1 => ("getStatus", "sa-v1"),
        PollMode::RestLast => ("getActivationOtpLast", "rest-last"),
    };

    let start = Instant::now();
    let mut polls = 0u32;
    while start.elapsed() < timeout {
        polls += 1;
        let body = invoke_read(spec, op_id, activation_id, api_key, auth)?;
        let parsed = match mode {
            PollMode::SaV2 => parse_sa_status_v2(&body),
            PollMode::SaV1 => parse_sa_status_text(&body),
            PollMode::RestLast => parse_rest_otp_last(&body),
        };
        match parsed {
            StatusParse::Received(code) => {
                return Ok(WaitOtpSuccess {
                    activation_id: activation_id.to_string(),
                    code,
                    mode: mode_name.to_string(),
                    polls,
                    elapsed_seconds: start.elapsed().as_secs(),
                });
            }
            StatusParse::Cancelled => {
                return Err("activation cancelled (STATUS_CANCEL)".into());
            }
            StatusParse::Failed(msg) => return Err(msg),
            StatusParse::Waiting => {
                thread::sleep(interval);
            }
        }
    }
    Err(format!(
        "timeout after {}s ({} polls); consider setStatus 8 (cancel) via `herosms call setStatus`",
        timeout.as_secs(),
        polls
    ))
}

pub fn wait_otp_dry_run_json(activation_id: &str, interval: u64, timeout: u64, mode: &str) -> Value {
    json!({
        "dryRun": true,
        "activationId": activation_id,
        "mode": mode,
        "intervalSeconds": interval,
        "timeoutSeconds": timeout,
        "pollOperations": {
            "sa-v2": "getStatusV2",
            "sa-v1": "getStatus",
            "rest-last": "getActivationOtpLast"
        },
        "successWhen": "non-empty OTP/code in status response",
        "cancelWhen": "STATUS_CANCEL or REST activation ended",
        "onTimeout": "exit error; optional setStatus status=8 to cancel per OpenAPI"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_ok() {
        assert_eq!(
            parse_sa_status_text("STATUS_OK:100001"),
            StatusParse::Received("100001".into())
        );
    }

    #[test]
    fn parses_status_cancel() {
        assert_eq!(parse_sa_status_text("STATUS_CANCEL"), StatusParse::Cancelled);
    }

    #[test]
    fn parses_v2_json_code() {
        let body = r#"{"verificationType":"sms","data":{"code":"123456","text":"x"}}"#;
        assert_eq!(
            parse_sa_status_v2(body),
            StatusParse::Received("123456".into())
        );
    }
}

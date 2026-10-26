//! HeroSMS inbound webhook payloads (`webhooks.sms-incoming` / `smsIncoming`).

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Documented HeroSMS webhook source IPs (OpenAPI `tags.Webhooks` description).
pub const DOCUMENTED_SOURCE_IPS: &[&str] = &["84.32.223.53", "185.138.88.87"];

/// Incoming SMS webhook body (`operationId`: `smsIncoming`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SmsIncomingPayload {
    /// Activation ID (`ActivationId`, integer).
    pub activation_id: i64,
    /// Message/event id (required in OpenAPI; property not expanded in schema — treat as integer).
    pub id: i64,
    /// OTP source / sender (`phoneFrom`, string).
    pub phone_from: String,
    /// Service code (`ServiceId`, 2–4 letters).
    pub service: String,
    /// Full SMS body (`SMSText`, string or null).
    pub text: Option<String>,
    /// Parsed OTP if provided (`SMSCode`, string or null).
    pub code: Option<String>,
    /// Country id (`CountryId`, integer 0–999).
    pub country: i64,
    /// Arrival time (`dateISO8601`, ISO-8601 date-time).
    pub received_at: String,
}

#[derive(Debug, PartialEq)]
pub enum WebhookParseError {
    Json(String),
    MissingField(&'static str),
    InvalidField { field: &'static str, message: String },
}

impl std::fmt::Display for WebhookParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(m) => write!(f, "invalid JSON: {m}"),
            Self::MissingField(k) => write!(f, "missing required field: {k}"),
            Self::InvalidField { field, message } => {
                write!(f, "field {field}: {message}")
            }
        }
    }
}

impl std::error::Error for WebhookParseError {}

pub fn parse_sms_incoming(body: &[u8]) -> Result<SmsIncomingPayload, WebhookParseError> {
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| WebhookParseError::Json(e.to_string()))?;

    let obj = value
        .as_object()
        .ok_or(WebhookParseError::Json("expected JSON object".into()))?;

    for key in [
        "activationId",
        "id",
        "phoneFrom",
        "service",
        "text",
        "country",
        "receivedAt",
    ] {
        if !obj.contains_key(key) {
            return Err(WebhookParseError::MissingField(key));
        }
    }

    let activation_id = parse_i64_field(obj, "activationId")?;
    let id = parse_i64_field(obj, "id")?;
    let phone_from = parse_string_field(obj, "phoneFrom")?;
    let service = parse_string_field(obj, "service")?;
    let text = parse_nullable_string(obj, "text");
    let code = parse_nullable_string(obj, "code");
    let country = parse_i64_field(obj, "country")?;
    let received_at = parse_string_field(obj, "receivedAt")?;

    if service.len() < 2 || service.len() > 4 {
        return Err(WebhookParseError::InvalidField {
            field: "service",
            message: "expected 2–4 character service code".into(),
        });
    }

    Ok(SmsIncomingPayload {
        activation_id,
        id,
        phone_from,
        service,
        text,
        code,
        country,
        received_at,
    })
}

fn parse_i64_field(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &'static str,
) -> Result<i64, WebhookParseError> {
    match obj.get(key) {
        Some(serde_json::Value::Number(n)) => n
            .as_i64()
            .ok_or_else(|| WebhookParseError::InvalidField {
                field: key,
                message: "expected integer".into(),
            }),
        Some(serde_json::Value::String(s)) => s.parse().map_err(|_| WebhookParseError::InvalidField {
            field: key,
            message: "expected integer".into(),
        }),
        _ => Err(WebhookParseError::MissingField(key)),
    }
}

fn parse_string_field(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &'static str,
) -> Result<String, WebhookParseError> {
    match obj.get(key) {
        Some(serde_json::Value::String(s)) => Ok(s.clone()),
        _ => Err(WebhookParseError::InvalidField {
            field: key,
            message: "expected string".into(),
        }),
    }
}

fn parse_nullable_string(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &'static str,
) -> Option<String> {
    match obj.get(key) {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Prefer explicit `code`; otherwise try common OTP patterns in `text`.
pub fn extract_otp(payload: &SmsIncomingPayload) -> Option<String> {
    if let Some(code) = &payload.code {
        let trimmed = code.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let text = payload.text.as_deref()?.trim();
    if text.is_empty() {
        return None;
    }
    // Common patterns: "123456 is your code", "code: 12345", standalone 4–8 digits
    if let Some(m) = regex_simple_otp(text) {
        return Some(m);
    }
    None
}

fn regex_simple_otp(text: &str) -> Option<String> {
    // Lightweight scan without regex crate: find first 4–8 digit run after "code" or standalone
    let lower = text.to_ascii_lowercase();
    if let Some(idx) = lower.find("code") {
        let tail = &text[idx..];
        if let Some(digits) = longest_digit_run(tail, 4, 8) {
            return Some(digits);
        }
    }
    longest_digit_run(text, 4, 8)
}

fn longest_digit_run(s: &str, min_len: usize, max_len: usize) -> Option<String> {
    let mut best: Option<String> = None;
    let mut current = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
            if current.len() > max_len {
                current.remove(0);
            }
        } else if !current.is_empty() {
            if current.len() >= min_len {
                best = Some(current.clone());
            }
            current.clear();
        }
    }
    if current.len() >= min_len {
        best = Some(current);
    }
    best
}

pub fn ip_allowed(client_ip: IpAddr, enforce: bool) -> bool {
    if !enforce {
        return true;
    }
    let s = client_ip.to_string();
    DOCUMENTED_SOURCE_IPS.iter().any(|ip| *ip == s)
}

#[derive(Debug, Serialize)]
pub struct SmsIncomingAck {
    pub ok: bool,
    pub activation_id: i64,
    pub message_id: i64,
    pub otp: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> Vec<u8> {
        br#"{
            "activationId": 123456,
            "id": 3416693217,
            "phoneFrom": "89854",
            "service": "tg",
            "text": "Your code is 12345",
            "code": "12345",
            "country": 2,
            "receivedAt": "2025-12-16T10:30:00.000000Z"
        }"#
        .to_vec()
    }

    #[test]
    fn parses_openapi_example_shape() {
        let p = parse_sms_incoming(&sample_json()).expect("parse");
        assert_eq!(p.activation_id, 123456);
        assert_eq!(extract_otp(&p), Some("12345".into()));
    }

    #[test]
    fn missing_field_errors() {
        let err = parse_sms_incoming(br#"{"activationId":1}"#).unwrap_err();
        assert!(matches!(err, WebhookParseError::MissingField(_)));
    }

    #[test]
    fn documented_ips_list() {
        assert_eq!(DOCUMENTED_SOURCE_IPS.len(), 2);
    }
}

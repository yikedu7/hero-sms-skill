//! OpenAPI-driven HeroSMS API client library.

pub mod webhook;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

pub const REST_BASE: &str = "https://hero-sms.com/api/v1";
pub const SA_BASE: &str = "https://hero-sms.com/stubs/handler_api.php";
pub const REDACT: &str = "***REDACTED***";

const HTTP_METHODS: &[&str] = &["get", "post", "put", "patch", "delete", "head", "options"];

static SA_WRITE_GET_OPS: &[&str] = &[
    "getNumber",
    "getNumberV2",
    "getRentNumber",
    "setStatus",
    "saCancelActivation",
    "saFinishActivation",
];

#[derive(Debug, Clone)]
pub struct Operation {
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub tags: Vec<String>,
    pub summary: String,
    pub parameters: Vec<Value>,
    pub request_body: Option<Value>,
    pub security: Vec<Value>,
    pub servers: Vec<Value>,
    pub is_inbound_webhook: bool,
}

impl Operation {
    pub fn is_write(&self) -> bool {
        if self.is_inbound_webhook {
            return false;
        }
        if matches!(
            self.method.as_str(),
            "post" | "put" | "patch" | "delete"
        ) {
            return true;
        }
        if self.path.starts_with("?action=") {
            if SA_WRITE_GET_OPS.contains(&self.operation_id.as_str()) {
                return true;
            }
            let action = self.path.split('=').nth(1).unwrap_or("");
            if matches!(action, "cancelActivation" | "finishActivation" | "setStatus") {
                return true;
            }
        }
        false
    }
}

pub fn load_spec(path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read spec: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("parse spec: {e}"))
}

pub fn resolve_ref(spec: &Value, node: &Value) -> Value {
    if let Some(obj) = node.as_object() {
        if let Some(ref_key) = obj.get("$ref").and_then(|v| v.as_str()) {
            if !ref_key.starts_with("#/") {
                panic!("Unsupported ref: {ref_key}");
            }
            let mut cur = spec;
            for part in ref_key[2..].split('/') {
                let part = part.replace("~1", "/").replace("~0", "~");
                cur = &cur[part];
            }
            return resolve_ref(spec, cur);
        }
    }
    node.clone()
}

fn resolve_parameters(spec: &Value, raw: Option<&Value>) -> Vec<Value> {
    match raw.and_then(|v| v.as_array()) {
        None => Vec::new(),
        Some(items) => items.iter().map(|i| resolve_ref(spec, i)).collect(),
    }
}

fn resolve_request_body(spec: &Value, raw: Option<&Value>) -> Option<Value> {
    raw.map(|r| resolve_ref(spec, r))
}

pub fn iter_operations(spec: &Value) -> Vec<Operation> {
    let mut out = Vec::new();

    if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
        for (path, path_item) in paths {
            if let Some(item) = path_item.as_object() {
                for method in HTTP_METHODS {
                    if let Some(op) = item.get(*method) {
                        let op_obj = op.as_object().cloned().unwrap_or_default();
                        let operation_id = op_obj
                            .get("operationId")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| format!("{}:{}", method, path));
                        let tags: Vec<String> = op_obj
                            .get("tags")
                            .and_then(|t| t.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let security = op_obj
                            .get("security")
                            .or_else(|| spec.get("security"))
                            .and_then(|s| s.as_array())
                            .map(|a| a.clone())
                            .unwrap_or_default();
                        let servers = op_obj
                            .get("servers")
                            .and_then(|s| s.as_array())
                            .map(|a| a.clone())
                            .unwrap_or_default();

                        out.push(Operation {
                            operation_id,
                            method: method.to_string(),
                            path: path.clone(),
                            tags,
                            summary: op_obj
                                .get("summary")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string(),
                            parameters: resolve_parameters(spec, op_obj.get("parameters")),
                            request_body: resolve_request_body(spec, op_obj.get("requestBody")),
                            security,
                            servers,
                            is_inbound_webhook: false,
                        });
                    }
                }
            }
        }
    }

    if let Some(webhooks) = spec.get("webhooks").and_then(|w| w.as_object()) {
        for (hook_name, hook_item) in webhooks {
            if let Some(item) = hook_item.as_object() {
                for method in HTTP_METHODS {
                    if let Some(op) = item.get(*method) {
                        let op_obj = op.as_object().cloned().unwrap_or_default();
                        let operation_id = op_obj
                            .get("operationId")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| format!("webhook:{}", method));
                        let tags: Vec<String> = op_obj
                            .get("tags")
                            .and_then(|t| t.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_else(|| vec!["Webhooks".to_string()]);

                        out.push(Operation {
                            operation_id,
                            method: method.to_string(),
                            path: format!("webhook:{}", hook_name),
                            tags,
                            summary: op_obj
                                .get("summary")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string(),
                            parameters: resolve_parameters(spec, op_obj.get("parameters")),
                            request_body: resolve_request_body(spec, op_obj.get("requestBody")),
                            security: Vec::new(),
                            servers: Vec::new(),
                            is_inbound_webhook: true,
                        });
                    }
                }
            }
        }
    }

    out
}

pub fn index_operations(spec: &Value) -> HashMap<String, Operation> {
    iter_operations(spec)
        .into_iter()
        .map(|op| (op.operation_id.clone(), op))
        .collect()
}

pub fn choose_base_url(op: &Operation) -> String {
    if let Some(server) = op.servers.first() {
        if let Some(url) = server.get("url").and_then(|u| u.as_str()) {
            return url.trim_end_matches('/').to_string();
        }
    }
    if op.path.starts_with("?action=") {
        return SA_BASE.to_string();
    }
    REST_BASE.to_string()
}

/// Query values: each key maps to one or more values (for `key[]` repeats).
pub type QueryParams = BTreeMap<String, Vec<String>>;

pub fn parse_path_pairs(pairs: &[String]) -> Result<HashMap<String, String>, String> {
    let mut path_params = HashMap::new();
    for pair in pairs {
        let Some((key, value)) = pair.split_once('=') else {
            return Err(format!("Expected key=value, got: {pair}"));
        };
        path_params.insert(key.to_string(), value.to_string());
    }
    Ok(path_params)
}

pub fn parse_query_pairs(pairs: &[String]) -> Result<QueryParams, String> {
    let mut query_params: QueryParams = BTreeMap::new();
    for pair in pairs {
        let Some((key, value)) = pair.split_once('=') else {
            return Err(format!("Expected key=value, got: {pair}"));
        };
        if key.ends_with("[]") {
            query_params.entry(key.to_string()).or_default().push(value.to_string());
        } else {
            query_params.insert(key.to_string(), vec![value.to_string()]);
        }
    }
    Ok(query_params)
}

pub fn build_url(
    base: &str,
    path: &str,
    path_params: &HashMap<String, String>,
    query: &QueryParams,
) -> String {
    let (url_path, merged_query): (String, QueryParams) = if path.starts_with("?action=") {
        let mut merged = query.clone();
        for part in path[1..].split('&') {
            if let Some((k, v)) = part.split_once('=') {
                merged.entry(k.to_string()).or_insert_with(|| vec![v.to_string()]);
            }
        }
        (String::new(), merged)
    } else {
        let mut url_path = path.to_string();
        for (key, value) in path_params {
            let encoded = utf8_percent_encode(value, NON_ALPHANUMERIC).to_string();
            url_path = url_path.replace(&format!("{{{key}}}"), &encoded);
        }
        (url_path, query.clone())
    };

    let mut url = if url_path.is_empty() {
        base.to_string()
    } else {
        format!("{}{}", base.trim_end_matches('/'), url_path)
    };

    if !merged_query.is_empty() {
        let qs = merged_query
            .iter()
            .flat_map(|(k, vals)| vals.iter().map(move |v| (k.clone(), v.clone())))
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    utf8_percent_encode(&k, NON_ALPHANUMERIC),
                    utf8_percent_encode(&v, NON_ALPHANUMERIC)
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        let sep = if url.contains('?') { "&" } else { "?" };
        url = format!("{url}{sep}{qs}");
    }
    url
}

fn pick_security(op: &Operation) -> Vec<String> {
    if op.security.is_empty() {
        return vec!["apiKeyAuth".to_string()];
    }
    op.security[0]
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

#[derive(Clone, Copy)]
pub enum AuthStyle {
    Auto,
    AuthorizationApiKey,
    AuthorizationBearer,
    XApiKey,
    Query,
}

impl AuthStyle {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(Self::Auto),
            "authorization-apikey" => Ok(Self::AuthorizationApiKey),
            "authorization-bearer" => Ok(Self::AuthorizationBearer),
            "x-api-key" => Ok(Self::XApiKey),
            "query" => Ok(Self::Query),
            other => Err(format!("Unknown auth style: {other}")),
        }
    }
}

pub fn apply_auth(
    op: &Operation,
    spec: &Value,
    headers: &mut BTreeMap<String, String>,
    query: &mut QueryParams,
    api_key: &str,
    auth_style: AuthStyle,
) {
    let sec_names = pick_security(op);
    let schemes = spec
        .get("components")
        .and_then(|c| c.get("securitySchemes"))
        .and_then(|s| s.as_object());

    let mut style = auth_style;
    if matches!(style, AuthStyle::Auto) {
        if sec_names.contains(&"apiKeyQuery".to_string())
            && !sec_names.contains(&"apiKeyAuth".to_string())
        {
            style = AuthStyle::Query;
        } else {
            style = AuthStyle::AuthorizationApiKey;
        }
    }

    match style {
        AuthStyle::Query => {
            query.entry("api_key".to_string()).or_insert_with(|| vec![api_key.to_string()]);
        }
        AuthStyle::XApiKey => {
            headers.insert("x-api-key".to_string(), api_key.to_string());
        }
        AuthStyle::AuthorizationBearer => {
            headers.insert("Authorization".to_string(), format!("Bearer {api_key}"));
        }
        AuthStyle::AuthorizationApiKey => {
            headers.insert("Authorization".to_string(), format!("ApiKey {api_key}"));
        }
        AuthStyle::Auto => {}
    }

    if matches!(auth_style, AuthStyle::Auto) {
        if let Some(schemes) = schemes {
            for name in &sec_names {
                if let Some(scheme) = schemes.get(name).and_then(|s| s.as_object()) {
                    if scheme.get("in").and_then(|v| v.as_str()) == Some("query")
                        && !query.contains_key("api_key")
                    {
                        query.insert("api_key".to_string(), vec![api_key.to_string()]);
                    }
                    if scheme.get("in").and_then(|v| v.as_str()) == Some("header")
                        && scheme.get("name").and_then(|v| v.as_str()) == Some("Authorization")
                        && !headers.contains_key("Authorization")
                    {
                        headers.insert("Authorization".to_string(), format!("ApiKey {api_key}"));
                    }
                }
            }
        }
    }
}

#[derive(Serialize)]
pub struct RedactedRequest {
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Option<String>,
}

fn redact_api_key_in_url(url: &str) -> String {
    let Some((base, query)) = url.split_once('?') else {
        return url.to_string();
    };
    let redacted = query
        .split('&')
        .map(|pair| {
            let Some((k, v)) = pair.split_once('=') else {
                return pair.to_string();
            };
            if k == "api_key" {
                format!("{k}={REDACT}")
            } else {
                format!("{k}={v}")
            }
        })
        .collect::<Vec<_>>()
        .join("&");
    format!("{base}?{redacted}")
}

pub fn redact_request(
    url: &str,
    headers: &BTreeMap<String, String>,
    body: Option<&[u8]>,
) -> RedactedRequest {
    let mut safe_headers = headers.clone();
    for h in ["Authorization", "x-api-key", "X-Api-Key"] {
        if safe_headers.contains_key(h) {
            safe_headers.insert(h.to_string(), REDACT.to_string());
        }
    }

    let safe_url = redact_api_key_in_url(url);

    RedactedRequest {
        url: safe_url,
        headers: safe_headers,
        body: body.map(|b| String::from_utf8_lossy(b).to_string()),
    }
}

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

pub fn execute_request(
    method: &str,
    url: &str,
    headers: &BTreeMap<String, String>,
    body: Option<&[u8]>,
) -> Result<HttpResponse, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(60))
        .build();

    let method = method.to_uppercase();
    let mut req = match method.as_str() {
        "GET" => agent.get(url),
        "POST" => agent.post(url),
        "PUT" => agent.put(url),
        "PATCH" => agent.patch(url),
        "DELETE" => agent.delete(url),
        "HEAD" => agent.head(url),
        other => return Err(format!("Unsupported HTTP method: {other}")),
    };

    for (k, v) in headers {
        req = req.set(k, v);
    }

    let resp = if let Some(b) = body {
        req.send_bytes(b)
    } else {
        req.call()
    };

    match resp {
        Ok(r) => {
            let status = r.status();
            let body = r.into_string().map_err(|e| e.to_string())?;
            Ok(HttpResponse { status, body })
        }
        Err(ureq::Error::Status(code, r)) => {
            let body = r.into_string().unwrap_or_default();
            Ok(HttpResponse {
                status: code,
                body,
            })
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn openapi_operation_ids(spec: &Value) -> Vec<String> {
    iter_operations(spec)
        .into_iter()
        .map(|o| o.operation_id)
        .collect()
}

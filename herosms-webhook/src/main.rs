//! **Optional reference** webhook receiver for HeroSMS `smsIncoming` (not required for agents).
//!
//! Requires **your own public HTTPS URL** registered in the HeroSMS account UI.
//! Grok Bot / cloud agents do not provide inbound webhooks — use `herosms wait-otp` instead.
//! Terminate TLS at nginx/Caddy; this process speaks plain HTTP on localhost.

use clap::Parser;
use herosms::webhook::{
    extract_otp, ip_allowed, parse_sms_incoming, SmsIncomingAck, DOCUMENTED_SOURCE_IPS,
};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tiny_http::{Header, Method, Response, Server, StatusCode};

#[derive(Parser)]
#[command(
    name = "herosms-webhook",
    about = "Optional sample receiver — requires operator-owned public HTTPS (not for Grok Bot)"
)]
struct Cli {
    /// Listen address (HTTP). Terminate TLS at your reverse proxy.
    #[arg(long, default_value = "127.0.0.1:8787", env = "HEROSMS_WEBHOOK_LISTEN")]
    listen: String,

    /// URL path to accept POSTs on (must match the path you register in HeroSMS).
    #[arg(long, default_value = "/webhook/sms-incoming", env = "HEROSMS_WEBHOOK_PATH")]
    path: String,

    /// Reject requests whose source IP is not in the OpenAPI-documented allowlist.
    #[arg(long, env = "HEROSMS_WEBHOOK_ENFORCE_IPS")]
    enforce_ips: bool,

    /// Optional shared secret: require header `X-HeroSMS-Webhook-Secret` to match (not official API auth).
    #[arg(long, env = "HEROSMS_WEBHOOK_SECRET")]
    secret: Option<String>,

    /// Header name for optional shared secret (default X-HeroSMS-Webhook-Secret).
    #[arg(long, default_value = "X-HeroSMS-Webhook-Secret", env = "HEROSMS_WEBHOOK_SECRET_HEADER")]
    secret_header: String,

    /// When set, respond 503 once (for testing retry behavior).
    #[arg(long, env = "HEROSMS_WEBHOOK_SIMULATE_BUSY")]
    simulate_busy: bool,
}

static BUSY_ONCE: AtomicBool = AtomicBool::new(true);

fn client_ip(request: &tiny_http::Request) -> Option<IpAddr> {
    if let Some(h) = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("X-Forwarded-For"))
    {
        let val = h.value.as_str();
        if let Some(first) = val.split(',').next() {
            if let Ok(ip) = IpAddr::from_str(first.trim()) {
                return Some(ip);
            }
        }
    }
    request.remote_addr().map(|addr| addr.ip())
}

fn check_secret(request: &tiny_http::Request, secret: Option<&str>, header_name: &str) -> bool {
    match secret {
        None => true,
        Some(expected) => request
            .headers()
            .iter()
            .find(|h| header_name.eq_ignore_ascii_case(&h.field.to_string()))
            .map(|h| h.value.as_str() == expected)
            .unwrap_or(false),
    }
}

fn handle_post(
    mut request: tiny_http::Request,
    path: &str,
    enforce_ips: bool,
    secret: Option<&str>,
    secret_header: &str,
    simulate_busy: bool,
) {
    if request.url() != path && !request.url().starts_with(&format!("{path}?")) {
        let _ = request.respond(Response::from_string("not found").with_status_code(404));
        return;
    }

    if request.method() != &Method::Post {
        let _ = request.respond(Response::from_string("method not allowed").with_status_code(405));
        return;
    }

    if simulate_busy && BUSY_ONCE.swap(false, Ordering::SeqCst) {
        let _ = request.respond(
            Response::from_string("temporary unavailable")
                .with_status_code(StatusCode(503)),
        );
        return;
    }

    if let Some(ip) = client_ip(&request) {
        if !ip_allowed(ip, enforce_ips) {
            eprintln!("rejected webhook from non-allowlisted IP {ip}");
            let _ = request.respond(Response::from_string("forbidden").with_status_code(403));
            return;
        }
    } else if enforce_ips {
        let _ = request.respond(Response::from_string("forbidden").with_status_code(403));
        return;
    }

    if !check_secret(&request, secret, secret_header) {
        eprintln!("rejected webhook: missing or invalid optional secret header");
        let _ = request.respond(Response::from_string("unauthorized").with_status_code(401));
        return;
    }

    let content_type_ok = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Content-Type"))
        .map(|h| h.value.as_str().starts_with("application/json"))
        .unwrap_or(false);
    if !content_type_ok {
        eprintln!("webhook rejected: expected Content-Type application/json");
        let _ = request.respond(Response::from_string("unsupported media type").with_status_code(415));
        return;
    }

    let mut body = Vec::new();
    if let Err(e) = request.as_reader().read_to_end(&mut body) {
        eprintln!("read body: {e}");
        let _ = request.respond(
            Response::from_string("bad request").with_status_code(StatusCode(400)),
        );
        return;
    }

    match parse_sms_incoming(&body) {
        Ok(payload) => {
            let otp = extract_otp(&payload);
            eprintln!(
                "smsIncoming activationId={} id={} service={} country={} from={} otp={:?}",
                payload.activation_id,
                payload.id,
                payload.service,
                payload.country,
                payload.phone_from,
                otp
            );
            let ack = SmsIncomingAck {
                ok: true,
                activation_id: payload.activation_id,
                message_id: payload.id,
                otp,
            };
            let json = serde_json::to_string(&ack).unwrap_or_else(|_| "{\"ok\":true}".into());
            let _ = request.respond(
                Response::from_string(json)
                    .with_status_code(StatusCode(200))
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()),
            );
        }
        Err(err) => {
            // Malformed payloads are unlikely to succeed on retry; acknowledge to avoid retry storms.
            eprintln!("smsIncoming parse error (returning 200 to avoid retries): {err}");
            let _ = request.respond(
                Response::from_string("{\"ok\":false,\"error\":\"invalid_payload\"}")
                    .with_status_code(StatusCode(200))
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()),
            );
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let server = Server::http(&cli.listen).expect("bind listen address");
    eprintln!(
        "herosms-webhook listening on http://{}{} (TLS: terminate at reverse proxy)",
        cli.listen,
        cli.path
    );
    eprintln!(
        "OpenAPI source IPs when --enforce-ips: {}",
        DOCUMENTED_SOURCE_IPS.join(", ")
    );
    if cli.secret.is_some() {
        eprintln!(
            "Optional local secret check enabled via header {}",
            cli.secret_header
        );
    }

    let path = Arc::new(cli.path);
    let enforce = cli.enforce_ips;
    let secret = cli.secret;
    let secret_header = cli.secret_header;
    let simulate = cli.simulate_busy;

    for request in server.incoming_requests() {
        handle_post(
            request,
            path.as_str(),
            enforce,
            secret.as_deref(),
            &secret_header,
            simulate,
        );
    }
}

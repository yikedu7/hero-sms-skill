use clap::{Parser, Subcommand};
use herosms::{
    apply_auth, build_url, choose_base_url, execute_request, index_operations, iter_operations,
    load_spec, parse_path_pairs, parse_query_pairs, redact_request, wait_otp, AuthStyle, Operation,
    PollMode,
};
use std::time::Duration;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "herosms", about = "HeroSMS OpenAPI-driven CLI")]
struct Cli {
    #[arg(
        long,
        global = true,
        default_value = "docs/openapi.json",
        env = "HEROSMS_OPENAPI_SPEC"
    )]
    spec: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List operationIds from OpenAPI
    List {
        #[arg(long)]
        tag: Option<String>,
    },
    /// Show one operation (method, path, parameters)
    Describe {
        operation_id: String,
    },
    /// Invoke an operation
    Call {
        operation_id: String,
        #[arg(long)]
        path: Vec<String>,
        #[arg(long)]
        query: Vec<String>,
        #[arg(long)]
        body_json: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        #[arg(long, env = "HEROSMS_AUTH_STYLE", default_value = "auto")]
        auth: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    /// Emit all operations as JSON (for tooling/tests)
    OperationsJson,
    /// Poll until OTP/code is received (default receive path; uses API polling, not webhooks)
    WaitOtp {
        activation_id: String,
        /// Poll interval seconds (default 5)
        #[arg(long, default_value = "5")]
        interval_secs: u64,
        /// Give up after this many seconds (default 300)
        #[arg(long, default_value = "300")]
        timeout_secs: u64,
        /// sa-v2 | sa-v1 | rest-last
        #[arg(long, default_value = "sa-v2")]
        mode: String,
        #[arg(long, env = "HEROSMS_AUTH_STYLE", default_value = "auto")]
        auth: String,
        /// Print plan only (no API key required)
        #[arg(long)]
        dry_run: bool,
    },
}

fn default_spec_path() -> PathBuf {
    PathBuf::from("docs/openapi.json")
}

fn load_spec_from(cli: &Cli) -> Result<Value, String> {
    let path = if cli.spec.as_os_str().is_empty() {
        default_spec_path()
    } else {
        cli.spec.clone()
    };
    load_spec(&path)
}

fn cmd_list(spec: &Value, tag: Option<&str>) -> ExitCode {
    let mut ops = iter_operations(spec);
    ops.sort_by(|a, b| {
        let ta = a.tags.first().map(String::as_str).unwrap_or("");
        let tb = b.tags.first().map(String::as_str).unwrap_or("");
        (ta, &a.operation_id).cmp(&(tb, &b.operation_id))
    });
    for op in ops {
        if let Some(t) = tag {
            if !op.tags.iter().any(|x| x == t) {
                continue;
            }
        }
        let tag_label = op.tags.first().map(String::as_str).unwrap_or("-");
        let kind = if op.is_inbound_webhook {
            "inbound"
        } else if op.is_write() {
            "write"
        } else {
            "read"
        };
        println!(
            "{}\t{:<6}\t{}\t{}\t{}",
            op.operation_id,
            op.method.to_uppercase(),
            tag_label,
            kind,
            op.summary
        );
    }
    ExitCode::SUCCESS
}

fn cmd_describe(spec: &Value, operation_id: &str) -> ExitCode {
    let idx = index_operations(spec);
    let Some(op) = idx.get(operation_id) else {
        eprintln!("Unknown operationId: {operation_id}");
        return ExitCode::from(1);
    };
    let payload = json!({
        "operationId": op.operation_id,
        "method": op.method.to_uppercase(),
        "path": op.path,
        "tags": op.tags,
        "summary": op.summary,
        "baseUrl": choose_base_url(op),
        "isWrite": op.is_write(),
        "isInboundWebhook": op.is_inbound_webhook,
        "parameters": op.parameters,
        "requestBody": op.request_body,
        "security": op.security,
    });
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    ExitCode::SUCCESS
}

fn cmd_operations_json(spec: &Value) -> ExitCode {
    let ops: Vec<Value> = iter_operations(spec)
        .into_iter()
        .map(|op| {
            json!({
                "operationId": op.operation_id,
                "method": op.method,
                "path": op.path,
                "tags": op.tags,
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&ops).unwrap());
    ExitCode::SUCCESS
}

fn prepare_body(
    op: &Operation,
    body_json: Option<&str>,
    body_file: Option<&PathBuf>,
) -> Result<Option<Vec<u8>>, String> {
    if op.request_body.is_none() {
        return Ok(None);
    }
    if let Some(path) = body_file {
        return Ok(Some(fs::read(path).map_err(|e| e.to_string())?));
    }
    if let Some(json) = body_json {
        return Ok(Some(json.as_bytes().to_vec()));
    }
    if matches!(op.method.as_str(), "post" | "put" | "patch") {
        return Ok(Some(b"{}".to_vec()));
    }
    Ok(None)
}

fn content_type_header(op: &Operation) -> Option<String> {
    op.request_body.as_ref().and_then(|rb| {
        rb.get("content")
            .and_then(|c| c.as_object())
            .and_then(|obj| {
                if obj.contains_key("application/json") {
                    Some("application/json".to_string())
                } else {
                    obj.keys().next().cloned()
                }
            })
    })
}

fn cmd_call(
    spec: &Value,
    operation_id: &str,
    path_flags: &[String],
    query_flags: &[String],
    body_json: Option<&str>,
    body_file: Option<&PathBuf>,
    auth: &str,
    mut dry_run: bool,
    yes: bool,
) -> ExitCode {
    let idx = index_operations(spec);
    let Some(op) = idx.get(operation_id) else {
        eprintln!("Unknown operationId: {operation_id}");
        return ExitCode::from(1);
    };

    if op.is_inbound_webhook {
        eprintln!(
            "{} is an inbound webhook (HeroSMS calls your server). See skill/SKILL.md and docs/API_COVERAGE.md.",
            op.operation_id
        );
        return ExitCode::from(1);
    }

    let api_key = std::env::var("HEROSMS_API_KEY")
        .unwrap_or_default()
        .trim()
        .to_string();

    if api_key.is_empty() && !dry_run && !(op.is_write() && !yes) {
        // writes auto dry-run; reads need key unless dry-run
        if !op.is_write() {
            eprintln!("HEROSMS_API_KEY is not set. Use --dry-run to preview the request.");
            return ExitCode::from(1);
        }
    }

    let path_params = match parse_path_pairs(path_flags) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let query_params = match parse_query_pairs(query_flags) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };

    let body_bytes = match prepare_body(op, body_json, body_file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };

    let mut headers: BTreeMap<String, String> = BTreeMap::new();
    headers.insert("Accept".to_string(), "application/json".to_string());
    if let Some(ct) = content_type_header(op) {
        headers.insert("Content-Type".to_string(), ct);
    }

    let mut query = query_params;
    let auth_style = match AuthStyle::parse(auth) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };

    if !api_key.is_empty() {
        apply_auth(op, spec, &mut headers, &mut query, &api_key, auth_style);
    }

    let base = choose_base_url(op);
    let url = build_url(&base, &op.path, &path_params, &query);

    if op.is_write() && !yes && !dry_run {
        dry_run = true;
        eprintln!("Write operation: showing dry-run preview. Pass --yes to execute.");
    }

    if dry_run {
        let preview = json!({
            "operationId": op.operation_id,
            "method": op.method.to_uppercase(),
            "request": redact_request(&url, &headers, body_bytes.as_deref()),
        });
        println!("{}", serde_json::to_string_pretty(&preview).unwrap());
        return ExitCode::SUCCESS;
    }

    if api_key.is_empty() {
        eprintln!("HEROSMS_API_KEY is not set.");
        return ExitCode::from(1);
    }

    let resp = match execute_request(
        &op.method,
        &url,
        &headers,
        body_bytes.as_deref(),
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("request failed: {e}");
            return ExitCode::from(1);
        }
    };

    let mut out = json!({
        "status": resp.status,
        "body": resp.body,
    });
    if let Ok(parsed) = serde_json::from_str::<Value>(&resp.body) {
        out["json"] = parsed;
    }
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
    if (200..300).contains(&resp.status) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn cmd_wait_otp(
    spec: &Value,
    activation_id: &str,
    interval_secs: u64,
    timeout_secs: u64,
    mode: &str,
    auth: &str,
    dry_run: bool,
) -> ExitCode {
    let poll_mode = match mode {
        "sa-v2" => PollMode::SaV2,
        "sa-v1" => PollMode::SaV1,
        "rest-last" => PollMode::RestLast,
        other => {
            eprintln!("Unknown mode {other}; use sa-v2, sa-v1, or rest-last");
            return ExitCode::from(2);
        }
    };

    if dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &wait_otp::wait_otp_dry_run_json(activation_id, interval_secs, timeout_secs, mode)
            )
            .unwrap()
        );
        return ExitCode::SUCCESS;
    }

    let api_key = std::env::var("HEROSMS_API_KEY")
        .unwrap_or_default()
        .trim()
        .to_string();
    if api_key.is_empty() {
        eprintln!("HEROSMS_API_KEY is not set. Use --dry-run to show the polling plan.");
        return ExitCode::from(1);
    }

    let auth_style = match AuthStyle::parse(auth) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    match wait_otp::poll_until_otp(
        spec,
        activation_id,
        &api_key,
        auth_style,
        poll_mode,
        Duration::from_secs(interval_secs),
        Duration::from_secs(timeout_secs),
    ) {
        Ok(success) => {
            println!("{}", serde_json::to_string_pretty(&success).unwrap());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let spec = match load_spec_from(&cli) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };

    match cli.command {
        Commands::List { tag } => cmd_list(&spec, tag.as_deref()),
        Commands::Describe { operation_id } => cmd_describe(&spec, &operation_id),
        Commands::OperationsJson => cmd_operations_json(&spec),
        Commands::Call {
            operation_id,
            path,
            query,
            body_json,
            body_file,
            auth,
            dry_run,
            yes,
        } => cmd_call(
            &spec,
            &operation_id,
            &path,
            &query,
            body_json.as_deref(),
            body_file.as_ref(),
            &auth,
            dry_run,
            yes,
        ),
        Commands::WaitOtp {
            activation_id,
            interval_secs,
            timeout_secs,
            mode,
            auth,
            dry_run,
        } => cmd_wait_otp(
            &spec,
            &activation_id,
            interval_secs,
            timeout_secs,
            &mode,
            &auth,
            dry_run,
        ),
    }
}

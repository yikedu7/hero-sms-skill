use herosms::{index_operations, iter_operations, load_spec, Operation};
use std::collections::HashSet;
use std::path::PathBuf;

fn spec_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/openapi.json")
}

fn openapi_operation_ids(spec: &serde_json::Value) -> HashSet<String> {
    iter_operations(spec)
        .into_iter()
        .map(|o| o.operation_id)
        .collect()
}

#[test]
fn cli_covers_all_openapi_operations() {
    let spec = load_spec(&spec_path()).expect("load openapi");
    let spec_ids = openapi_operation_ids(&spec);
    let cli_ids: HashSet<String> = index_operations(&spec).keys().cloned().collect();
    let missing: Vec<_> = spec_ids.difference(&cli_ids).cloned().collect();
    let extra: Vec<_> = cli_ids.difference(&spec_ids).cloned().collect();
    assert!(missing.is_empty(), "missing: {:?}", missing);
    assert!(extra.is_empty(), "extra: {:?}", extra);
}

#[test]
fn openapi_operation_count_is_51() {
    let spec = load_spec(&spec_path()).expect("load openapi");
    assert_eq!(openapi_operation_ids(&spec).len(), 51);
}

fn op(spec: &serde_json::Value, id: &str) -> Operation {
    index_operations(spec).get(id).cloned().expect("operation")
}

#[test]
fn write_classification() {
    let spec = load_spec(&spec_path()).expect("load openapi");
    assert!(!op(&spec, "getBalance").is_write());
    assert!(op(&spec, "getNumber").is_write());
    assert!(op(&spec, "buyActivationBatch").is_write());
    assert!(!op(&spec, "getActiveActivations").is_write());
}

#[test]
fn dry_run_does_not_require_api_key() {
    let bin = env!("CARGO_BIN_EXE_herosms");
    let out = std::process::Command::new(bin)
        .args(["call", "getBalance", "--auth", "query", "--dry-run"])
        .env_remove("HEROSMS_API_KEY")
        .output()
        .expect("run herosms");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let payload: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("json stdout");
    assert_eq!(payload["operationId"], "getBalance");
    assert!(payload["request"]["url"]
        .as_str()
        .unwrap()
        .contains("handler_api.php"));
}

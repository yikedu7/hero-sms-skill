//! Live API smoke tests (no-op when HEROSMS_API_KEY is unset).

#[test]
fn live_get_balance_and_list_activations() {
    if std::env::var("HEROSMS_API_KEY")
        .map(|k| k.trim().is_empty())
        .unwrap_or(true)
    {
        eprintln!("skip live smoke: HEROSMS_API_KEY not set");
        return;
    }

    let bin = env!("CARGO_BIN_EXE_herosms");
    let balance = std::process::Command::new(bin)
        .args(["call", "getBalance", "--auth", "query"])
        .output()
        .expect("getBalance");
    assert!(balance.status.success(), "{}", String::from_utf8_lossy(&balance.stderr));
    let body = String::from_utf8_lossy(&balance.stdout);
    assert!(
        body.contains("ACCESS_BALANCE") || body.to_uppercase().contains("BALANCE"),
        "unexpected balance response"
    );

    let activations = std::process::Command::new(bin)
        .args(["call", "getActiveActivations", "--query", "size=1"])
        .output()
        .expect("getActiveActivations");
    assert!(
        activations.status.success(),
        "{}",
        String::from_utf8_lossy(&activations.stderr)
    );
}

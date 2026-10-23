use herosms::webhook::parse_sms_incoming;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn sample_payload() -> String {
    r#"{
        "activationId": 123456,
        "id": 3416693217,
        "phoneFrom": "Telegram",
        "service": "tg",
        "text": "Telegram code 123456",
        "code": "123456",
        "country": 6,
        "receivedAt": "2026-02-16T12:36:59+03:00"
    }"#
    .to_string()
}

#[test]
fn webhook_receiver_returns_200_and_parses_otp() {
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.local_addr().unwrap().port()
    };

    let bin = env!("CARGO_BIN_EXE_herosms-webhook");
    let mut child: Child = Command::new(bin)
        .env("HEROSMS_WEBHOOK_LISTEN", format!("127.0.0.1:{port}"))
        .env("HEROSMS_WEBHOOK_PATH", "/webhook/sms-incoming")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn webhook");

    thread::sleep(Duration::from_millis(300));

    let payload = sample_payload();
    let response = ureq::post(&format!("http://127.0.0.1:{port}/webhook/sms-incoming"))
        .set("Content-Type", "application/json")
        .send_string(&payload)
        .expect("post webhook");

    assert_eq!(response.status(), 200);
    let body = response.into_string().expect("body");
    assert!(body.contains("\"otp\":\"123456\""));

    let _ = child.kill();
}

#[test]
fn parse_matches_openapi_required_fields() {
    let payload = sample_payload();
    let parsed = parse_sms_incoming(payload.as_bytes()).expect("parse");
    assert_eq!(parsed.activation_id, 123456);
    assert_eq!(parsed.service, "tg");
}

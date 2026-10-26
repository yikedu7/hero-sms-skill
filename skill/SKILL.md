---
name: hero-sms-api
description: >-
  Call the full official HeroSMS API (REST v1 + SMS-Activate-compatible handler)
  using the OpenAPI-driven Rust CLI. Default for receiving OTP is API polling
  (getStatus / wait-otp), not webhooks. Use for balance, buy/manage activations,
  OTP, cancel/finish, email flows, classifiers, and stats. Webhooks are optional
  documentation only unless the operator hosts public HTTPS.
---

# HeroSMS API skill

Official docs: https://hero-sms.com/api  
OpenAPI source of truth: `docs/openapi.json`

## Delivery model (read this first)

| Concern | Default | Notes |
| --- | --- | --- |
| **Outbound API** | `herosms` Rust CLI | All **50** client operations via `call` / `wait-otp`. Needs `HEROSMS_API_KEY`. |
| **Receive OTP / SMS** | **Polling** | Works in Grok Bot, Cursor Cloud Agent, CI, local — no public URL required. |
| **Webhooks** | **Optional** | HeroSMS pushes to **your** HTTPS URLs. Not provided by Grok Bot. Use only if the operator runs their own endpoint (or optional forward to a bot URL they control). |

**Environment rule:** If there is no public HTTPS receiver, **use polling only**. Do not fail or block tasks because webhooks are unavailable.

## Install

```bash
cargo build -p herosms --release
# target/release/herosms
```

Optional reference sample (not required for agents): `cargo build -p herosms-webhook --release`.

## Authentication

```bash
export HEROSMS_API_KEY="your_key_here"
```

| Style | How |
| --- | --- |
| `auto` (default) | REST: `Authorization: ApiKey <key>`; SMS-Activate: `?api_key=` |
| `authorization-apikey` / `authorization-bearer` / `x-api-key` / `query` | See `--auth` |

Never print or commit the API key.

## Outbound workflow

1. `herosms list` / `herosms describe <operationId>`
2. Read calls without `--yes`
3. Writes: dry-run by default; `--yes` after operator approval

```bash
herosms call getBalance --auth query
herosms call getActiveActivations --query size=5
```

## Receiving codes (default: polling)

After `getNumber` / `buyActivationBatch`, poll until a code appears.

### Recommended: `wait-otp`

```bash
herosms wait-otp <activationId> \
  --mode sa-v2 \
  --interval-secs 5 \
  --timeout-secs 300 \
  --auth query
```

| Flag | Default | Meaning |
| --- | --- | --- |
| `--mode` | `sa-v2` | `getStatusV2` (JSON). Alternatives: `sa-v1` (`getStatus` text), `rest-last` (`getActivationOtpLast`) |
| `--interval-secs` | `5` | Sleep between polls |
| `--timeout-secs` | `300` | Stop after 5 minutes |

**Success:** non-empty OTP/code in the status response (e.g. `STATUS_OK:<code>` or V2 `data.code`). CLI prints JSON and exits `0`.

**Still waiting:** `STATUS_WAIT_CODE`, `STATUS_WAIT_RESEND`, or empty REST OTP — keep polling until timeout.

**Cancelled / dead:** `STATUS_CANCEL` — stop; activation ended.

**After timeout:** Inform the operator; optionally cancel to recover balance:

```bash
# OpenAPI setStatus: status 8 = cancel (refund when allowed)
herosms call setStatus --auth query --query id=<activationId> --query status=8 --yes
```

**Finish when done with code (funds not refunded):**

```bash
# setStatus 6 = complete activation (OpenAPI)
herosms call setStatus --auth query --query id=<activationId> --query status=6 --yes
# or REST: herosms call finishActivation --path activationId=<id> --yes
```

Preview polling without a key:

```bash
herosms wait-otp 12345 --dry-run
```

### Manual loop (same behavior)

```bash
while true; do
  herosms call getStatusV2 --auth query --query id=<activationId>
  sleep 5
done
```

Parse for `data.code` or `STATUS_OK:…`. Use the same timeout / cancel / finish rules as above.

### REST OTP list (optional)

```bash
herosms call getActivationOtpList --path activationId=<id>
herosms call getActivationOtpLast --path activationId=<id>
```

## Webhooks (optional enhancement only)

HeroSMS can **push** incoming SMS JSON to up to **3 HTTPS URLs** configured in **account personal information**. There is **no API** to register URLs.

- **Grok Bot / typical cloud agents do not expose a fillable inbound webhook URL.** Do not depend on webhooks in those environments.
- **Optional forward path:** If the operator runs `herosms-webhook` (or nginx) on the public internet, they may forward payloads to another HTTPS URL (including a bot endpoint they configured separately). That is operator infrastructure, not this skill.

### If you host a receiver (operator-owned HTTPS)

| Topic | OpenAPI / docs |
| --- | --- |
| Event | `smsIncoming` (`webhooks.sms-incoming`) |
| Method | `POST`, `Content-Type: application/json` |
| Response | **HTTP 200** within **3s** |
| Retries | **≥7**, **20–30s** apart, **≥3 min** total if not 200 |
| Source IPs | `84.32.223.53`, `185.138.88.87` |
| Auth | **None** in OpenAPI (no HMAC). Use HTTPS + IP allowlist; optional local secret in sample only |

Payload fields: `activationId`, `id`, `phoneFrom`, `service`, `text`, `code`, `country`, `receivedAt` — see previous OpenAPI table in `docs/API_COVERAGE.md`.

Sample (local HTTP; **you** add TLS and a public URL):

```bash
cargo run -p herosms-webhook --release
```

**Not a product dependency:** receiving via webhook in the bot environment is **N/A** — use polling.

## Coverage and tests

- `docs/API_COVERAGE.md` — outbound CLI + webhook platform vs receive notes
- `cargo test --workspace` — offline; live API only if `HEROSMS_API_KEY` set

## Known gaps

- Webhook URL registration is UI-only.
- No official webhook signing in OpenAPI.

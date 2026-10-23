---
name: hero-sms-api
description: >-
  Call the full official HeroSMS API (REST v1 + SMS-Activate-compatible handler)
  using the OpenAPI-driven Rust CLI in this repo. Use when buying or managing
  phone/email activations, checking balance, prices, countries, operators, OTP,
  prolong/reactivate, classifiers, deliverability stats, or documenting
  SMS-Activate migration and webhooks.
---

# HeroSMS API skill

Official docs: https://hero-sms.com/api  
OpenAPI source of truth: `docs/openapi.json` (copy of https://hero-sms.com/docs/v1/openapi.json)

## When to use

- Read-only discovery: balance, countries, services, operators, prices, active activations, OTP lists, email inventory, stats.
- Purchases and lifecycle: buy number (REST batch or SMS-Activate `getNumber`), status, finish, cancel, prolong, reactivate, replace.
- Email activations: list, buy, batch, domains, offers, reorder, delete.
- Integration help: SMS-Activate host swap (`api.sms-activate.ae` → `hero-sms.com`), auth header variants, webhook payload shape.

Do **not** use this skill to invent endpoints; only operations in `docs/openapi.json`.

## Install (Cursor / Grok Bot)

1. Clone or vendor this repository.
2. Point the agent at this skill file: `skill/SKILL.md`.
3. Build the CLI (Rust stable):

```bash
cargo build -p herosms --release
# binary: target/release/herosms
cargo build -p herosms-webhook --release
# binary: target/release/herosms-webhook
```

Optional: add `target/release` to `PATH`, or call via `cargo run --release -- <subcommand>`.

## Authentication

Set the API key **only** via environment variable (never pass the key on the command line):

```bash
export HEROSMS_API_KEY="your_key_here"
```

Supported placements (OpenAPI + common doc variants):

| Style | How |
| --- | --- |
| `auto` (default) | REST: `Authorization: ApiKey <key>`; SMS-Activate: `?api_key=` query |
| `authorization-apikey` | `Authorization: ApiKey <key>` |
| `authorization-bearer` | `Authorization: Bearer <key>` |
| `x-api-key` | Header `x-api-key: <key>` |
| `query` | Query `api_key=<key>` |

Override style: `--auth query` or `HEROSMS_AUTH_STYLE=authorization-bearer`.

**Never** print, log, or commit `HEROSMS_API_KEY`. Dry-run output redacts secrets.

## Safe workflow for agents

1. **Discover** — `herosms list` / `herosms describe <operationId>`.
2. **Read first** — balance, catalogs, lists (no `--yes` needed).
3. **Writes** — default is dry-run preview; show the operator; add `--yes` only after explicit approval.
4. **Money** — `getNumber`, `buyActivationBatch`, email purchase, rent, prolong, etc. spend balance. Do not purchase in CI or unattended smoke tests.

### Examples (read-only)

```bash
# SMS-Activate balance
herosms call getBalance --auth query

# REST active activations (first page)
herosms call getActiveActivations --query size=5

# Countries (SMS-Activate compatible)
herosms call getCountries --auth query
```

### Examples (writes — dry-run then execute)

```bash
# Preview purchase (REST batch)
herosms call buyActivationBatch \
  --body-json '{"items":[{"service":"tg","country":6}]}' \
  --dry-run

# After operator approval
herosms call buyActivationBatch \
  --body-json '{"items":[{"service":"tg","country":6}]}' \
  --yes
```

Path and query parameters use repeated `key=value` flags:

```bash
herosms call getActivationOtpLast --path activationId=12345
herosms call getHistory --auth query --query from=2025-01-01 --query to=2025-12-31
```

## Servers

- REST: `https://hero-sms.com/api/v1` + path (e.g. `/activations`)
- SMS-Activate compatible: `https://hero-sms.com/stubs/handler_api.php` + `?action=...`

The CLI picks the server from each operation’s OpenAPI `servers` block.

## Webhooks

HeroSMS pushes events to **your** servers. Outbound `herosms call` does **not** invoke webhooks. The OpenAPI defines one inbound webhook today: **Incoming SMS** (`operationId`: `smsIncoming`, path key `webhooks.sms-incoming`).

### Register URLs (platform / account UI)

- Configure up to **3 simultaneous HTTPS webhook URLs** in HeroSMS **account personal information** (OpenAPI `tags.Webhooks` description).
- There is **no client API** in `docs/openapi.json` to register or rotate URLs — coverage marks this as **platform config** (`webhook-url-registration` in `docs/API_COVERAGE.md`).
- Each incoming SMS triggers an independent POST to every configured URL.

### Delivery semantics (from OpenAPI)

| Item | Value |
| --- | --- |
| Method | `POST` |
| Content-Type | `application/json` |
| Response timeout | **3 seconds** |
| Success | HTTP **200** |
| On non-200 / timeout | Unsuccessful; **≥7 retries**, **20–30s** between attempts, **≥3 minutes** total retry window |
| Recommendation | Return **200** even if the SMS was already processed (idempotent handling) |

### Source IP allowlist (OpenAPI)

Documented sender IPs (whitelist at your firewall or `--enforce-ips` in the sample receiver):

- `84.32.223.53`
- `185.138.88.87`

### Authentication / verification (OpenAPI)

The `smsIncoming` webhook definition includes **no** signature, HMAC, or API-key header. Treat payloads as trusted only after **network controls** (HTTPS, IP allowlist, private ingress) and application validation.

**Optional local practice (not official):** the sample `herosms-webhook` binary can require a shared secret in header `X-HeroSMS-Webhook-Secret` via `HEROSMS_WEBHOOK_SECRET` — you must configure the same header at your reverse proxy or ignore this feature.

### Payload: `smsIncoming` (Incoming SMS)

Required JSON fields per `webhooks.sms-incoming.post.requestBody`:

| Field | OpenAPI type | Description | Example |
| --- | --- | --- | --- |
| `activationId` | integer (`ActivationId`) | Activation ID | `123456` |
| `id` | required in schema list; use integer message id | SMS / event id | `3416693217` |
| `phoneFrom` | string | OTP source / sender | `"89854"` or `"Telegram"` |
| `service` | string (`ServiceId`, 2–4 letters) | Service code | `"tg"` |
| `text` | string or null (`SMSText`) | Full SMS body | `"Your code is 12345"` |
| `code` | string or null (`SMSCode`, optional property) | Parsed OTP when present | `"12345"` |
| `country` | integer (`CountryId`, 0–999) | Country id | `2` |
| `receivedAt` | string date-time (`dateISO8601`) | Arrival time | `"2025-12-16T10:30:00.000000Z"` |

Example body (aligned with OpenAPI component examples):

```json
{
  "activationId": 123456,
  "id": 3416693217,
  "phoneFrom": "Telegram",
  "service": "tg",
  "text": "Telegram code 123456",
  "code": "123456",
  "country": 6,
  "receivedAt": "2026-02-16T12:36:59+03:00"
}
```

Library helpers: `herosms::webhook::parse_sms_incoming`, `extract_otp` (prefers `code`, else heuristics on `text`).

### Runnable receiver (`herosms-webhook`)

```bash
cargo build -p herosms-webhook --release
# HTTP on localhost — put Caddy/nginx in front for public HTTPS
HEROSMS_WEBHOOK_LISTEN=127.0.0.1:8787 \
  ./target/release/herosms-webhook
```

Defaults: path `/webhook/sms-incoming`, returns **200** + JSON ack with `otp` when parsed. Use `HEROSMS_WEBHOOK_ENFORCE_IPS=1` to enforce the OpenAPI IP list. Use `HEROSMS_WEBHOOK_SIMULATE_BUSY=1` once to test retry behavior (503).

See `docs/API_COVERAGE.md` for platform vs skill-side checklist rows.

## Coverage and tests

- Checklist: `docs/API_COVERAGE.md` (51 OpenAPI operations + webhook platform/receiver rows).
- Offline: `cargo test` (no network; live test no-ops without `HEROSMS_API_KEY`).
- Live smoke (optional): `HEROSMS_API_KEY=... cargo test --test live_smoke` — balance + list activations only; **no purchases**.

## Known gaps

- Outbound API surface is fully defined by the bundled OpenAPI; there are no extra undocumented REST methods in this skill.
- Webhook registration URLs are configured in the HeroSMS web UI, not via an API operation in the spec.

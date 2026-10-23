---
name: hero-sms-api
description: >-
  Call the full official HeroSMS API (REST v1 + SMS-Activate-compatible handler)
  using the OpenAPI-driven CLI in this repo. Use when buying or managing phone/email
  activations, checking balance, prices, countries, operators, OTP, prolong/reactivate,
  classifiers, deliverability stats, or documenting SMS-Activate migration and webhooks.
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
2. Point the agent at this skill file: `skill/SKILL.md` (full path on disk, or copy into your agent skills directory).
3. Ensure Python 3.10+ is available.

CLI (all operations):

```bash
python skill/scripts/herosms.py list
python skill/scripts/herosms.py describe getBalance
```

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

1. **Discover** — `list` / `describe <operationId>` to see parameters and whether the call is a write.
2. **Read first** — balance, catalogs, lists (no `--yes` needed).
3. **Writes** — always run `--dry-run` first; show the operator the preview; run again with `--yes` only after explicit approval.
4. **Money** — `getNumber`, `buyActivationBatch`, email purchase, rent, prolong, etc. spend balance. Do not purchase in CI or unattended smoke tests.

### Examples (read-only)

```bash
# SMS-Activate balance
python skill/scripts/herosms.py call getBalance --auth query

# REST active activations (first page)
python skill/scripts/herosms.py call getActiveActivations --query size=5

# Countries (SMS-Activate compatible)
python skill/scripts/herosms.py call getCountries --auth query
```

### Examples (writes — dry-run then execute)

```bash
# Preview purchase (REST batch) — does not send without --yes
python skill/scripts/herosms.py call buyActivationBatch \
  --body-json '{"items":[{"service":"tg","country":6}]}' \
  --dry-run

# After operator approval
python skill/scripts/herosms.py call buyActivationBatch \
  --body-json '{"items":[{"service":"tg","country":6}]}' \
  --yes
```

Path and query parameters use repeated `key=value` flags:

```bash
python skill/scripts/herosms.py call getActivationOtpLast --path activationId=12345
python skill/scripts/herosms.py call getHistory --auth query --query from=2025-01-01 --query to=2025-12-31
```

## Servers

- REST: `https://hero-sms.com/api/v1` + path (e.g. `/activations`)
- SMS-Activate compatible: `https://hero-sms.com/stubs/handler_api.php` + `?action=...`

The CLI picks the server from each operation’s OpenAPI `servers` block.

## Webhooks (inbound only)

Operation `smsIncoming` is **not** called by this CLI. HeroSMS POSTs JSON to up to three HTTPS URLs configured in the account profile. Respond with HTTP 200 within 3 seconds. Whitelist source IPs documented in OpenAPI (`84.32.223.53`, `185.138.88.87`). See `docs/API_COVERAGE.md`.

## Coverage and tests

- Checklist: `docs/API_COVERAGE.md` (51 operations).
- Offline: `pytest` (no network).
- Live smoke (optional): `HEROSMS_API_KEY=... pytest tests/test_live_smoke.py` — balance + list activations only; **no purchases**.

## Known gaps

- Outbound API surface is fully defined by the bundled OpenAPI; there are no extra undocumented REST methods in this skill.
- Webhook registration URLs are configured in the HeroSMS web UI, not via an API operation in the spec.

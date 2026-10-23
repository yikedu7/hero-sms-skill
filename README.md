# hero-sms-skill

Reusable **HeroSMS official API** skill for Grok Bot and Cursor agents, with full coverage of the published OpenAPI 3.2 spec (REST + SMS-Activate-compatible handler).

- Docs: https://hero-sms.com/api  
- OpenAPI: `docs/openapi.json` (mirror of https://hero-sms.com/docs/v1/openapi.json)  
- Agent recipe: [`skill/SKILL.md`](skill/SKILL.md)

## Build & install

Requires **Rust stable** (see `rust-toolchain.toml`).

```bash
cargo build --release
```

Binary: `target/release/herosms`  
Install locally: `cargo install --path .` (installs `herosms` into `~/.cargo/bin`).

Run without installing:

```bash
cargo run --release -- list
```

## API key

Copy `.env.example` to `.env` and set your key (do not commit `.env`):

```bash
export HEROSMS_API_KEY="your_key_from_hero_sms_dashboard"
```

Optional: `HEROSMS_AUTH_STYLE=auto|authorization-apikey|authorization-bearer|x-api-key|query`

## Quickstart (read-only)

```bash
# List all 51 operations (50 outbound + 1 inbound webhook)
herosms list

# SMS-Activate balance (needs HEROSMS_API_KEY)
herosms call getBalance --auth query

# Preview balance request without a key
herosms call getBalance --auth query --dry-run

# REST: first page of active activations
herosms call getActiveActivations --query size=5
```

**Warning:** Operations such as `getNumber`, `buyActivationBatch`, email purchase, prolong, and cancel/finish **spend balance or change activations**. Writes default to **dry-run**; pass `--yes` only when you intend to execute.

```bash
herosms call getNumber --auth query --query service=tg --query country=6 --dry-run
herosms call getNumber --auth query --query service=tg --query country=6 --yes
```

## CLI reference

| Command | Purpose |
| --- | --- |
| `list [--tag TAG]` | All `operationId` values from OpenAPI |
| `describe <operationId>` | Method, path, parameters, write vs read |
| `call <operationId> [--path k=v] [--query k=v] [--body-json '{}'] [--dry-run] [--yes]` | Invoke endpoint |
| `operations-json` | Machine-readable operation list |

Global flag: `--spec docs/openapi.json` (or `HEROSMS_OPENAPI_SPEC`).

## Coverage

See [`docs/API_COVERAGE.md`](docs/API_COVERAGE.md) — every path+method (+ webhook) from the spec.

Regenerate after OpenAPI updates:

```bash
python scripts/generate_api_coverage.py
```

## Tests

```bash
cargo test
```

Offline tests always run. The `live_smoke` integration test calls the real API only when `HEROSMS_API_KEY` is set (balance + activations list; **no purchases**).

```bash
HEROSMS_API_KEY=... cargo test --test live_smoke
```

## Project layout

```
docs/openapi.json          # official spec copy
docs/API_COVERAGE.md       # coverage checklist
skill/SKILL.md             # agent instructions
src/                       # Rust CLI (herosms)
tests/                     # cargo integration tests
```

## Known gaps

- Webhook URL configuration is in the HeroSMS account UI, not an outbound API in the spec.
- This repo does not implement a webhook receiver; see `skill/SKILL.md` for payload notes.

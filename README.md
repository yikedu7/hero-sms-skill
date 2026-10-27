# hero-sms-skill

Reusable **HeroSMS official API** skill for any coding agent (Cursor, Claude Code, Codex, OpenClaw, Grok Bot, and similar): **Rust CLI** for all outbound OpenAPI operations; **polling** (`wait-otp`) as the default way to receive OTP. Webhooks are optional and require your own public HTTPS endpoint.

- Docs: https://hero-sms.com/api  
- OpenAPI: `docs/openapi.json` (mirror of https://hero-sms.com/docs/v1/openapi.json)  
- Agent recipe: [`skill/SKILL.md`](skill/SKILL.md)

## Build & install

**Recommended (any coding agent / host):** download a prebuilt `herosms` binary from [GitHub Releases](https://github.com/yikedu7/hero-sms-skill/releases). Assets are published for:

| Platform | Asset suffix |
| --- | --- |
| Linux x86_64 | `x86_64-unknown-linux-musl` (static) |
| Linux arm64 | `aarch64-unknown-linux-gnu` |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |

One-liner (installs to `~/.local/bin`; add that directory to `PATH` if needed):

```bash
curl -fsSL https://raw.githubusercontent.com/yikedu7/hero-sms-skill/main/scripts/install-herosms.sh | bash
```

Pin a version: `HEROSMS_VERSION=0.1.0 curl -fsSL .../install-herosms.sh | bash`

Verify checksums from the release’s `SHA256SUMS` file after download.

**Fallback — build from source** (requires **Rust stable**, see `rust-toolchain.toml`):

```bash
cargo build -p herosms --release
# binary: target/release/herosms
cargo install -p herosms --path herosms
```

Optional webhook sample (operator-hosted HTTPS only): `cargo build -p herosms-webhook --release`

Run without installing (source tree):

```bash
cargo run -p herosms --release -- list
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

## Receive OTP (default: polling)

```bash
herosms wait-otp <activationId> --auth query --mode sa-v2
herosms wait-otp <activationId> --dry-run   # no API key
```

See `skill/SKILL.md` for intervals, timeout, cancel (`setStatus` 8), and finish (`setStatus` 6).

## Webhooks (optional)

Documented in `skill/SKILL.md`. **Not used in Grok Bot** unless the operator exposes public HTTPS. Optional sample: `cargo run -p herosms-webhook --release`.

## Tests

```bash
cargo test --workspace
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

## Releases (maintainers)

CI workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) builds platform binaries on every PR (dry build). After merge:

1. **Tag release:** `git tag v0.1.0 && git push origin v0.1.0` (version should match `herosms/Cargo.toml`), or  
2. **Manual:** Actions → **Release** → **Run workflow** (optional `version` input; defaults to `herosms/Cargo.toml`).

Published assets: `herosms-<version>-<target-triple>` plus `SHA256SUMS`.

## Known gaps

- Webhook URL configuration is in the HeroSMS account UI, not an outbound API in the spec.
- This repo does not implement a webhook receiver; see `skill/SKILL.md` for payload notes.

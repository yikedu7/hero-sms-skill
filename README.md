# hero-sms-skill

Reusable **HeroSMS official API** skill for Grok Bot and Cursor agents, with full coverage of the published OpenAPI 3.2 spec (REST + SMS-Activate-compatible handler).

- Docs: https://hero-sms.com/api  
- OpenAPI: `docs/openapi.json` (mirror of https://hero-sms.com/docs/v1/openapi.json)  
- Agent recipe: [`skill/SKILL.md`](skill/SKILL.md)

## Install

1. Clone this repository.
2. In Cursor / Grok Bot, add the skill path to your agent configuration (point at `skill/SKILL.md` on disk).
3. Install test dependencies (optional): `pip install -r requirements-dev.txt`

## API key

Copy `.env.example` to `.env` and set your key (do not commit `.env`):

```bash
export HEROSMS_API_KEY="your_key_from_hero_sms_dashboard"
```

Optional: `HEROSMS_AUTH_STYLE=auto|authorization-apikey|authorization-bearer|x-api-key|query`

## Quickstart (read-only)

```bash
# List all 51 operations (50 outbound + 1 inbound webhook)
python skill/scripts/herosms.py list

# SMS-Activate balance
python skill/scripts/herosms.py call getBalance --auth query

# REST: first page of active activations
python skill/scripts/herosms.py call getActiveActivations --query size=5
```

**Warning:** Operations such as `getNumber`, `buyActivationBatch`, email purchase, prolong, and cancel/finish **spend balance or change activations**. Use `--dry-run` first; add `--yes` only after you intend to execute.

```bash
python skill/scripts/herosms.py call getNumber \
  --auth query --query service=tg --query country=6 --dry-run
```

## CLI reference

| Command | Purpose |
| --- | --- |
| `list [--tag TAG]` | All `operationId` values from OpenAPI |
| `describe <operationId>` | Method, path, parameters, write vs read |
| `call <operationId> [--path k=v] [--query k=v] [--body-json '{}'] [--dry-run] [--yes]` | Invoke endpoint |
| `operations-json` | Machine-readable operation list (tests) |

## Coverage

See [`docs/API_COVERAGE.md`](docs/API_COVERAGE.md) — every path+method (+ webhook) from the spec, marked covered via the generic caller.

Regenerate after OpenAPI updates:

```bash
python scripts/generate_api_coverage.py
```

## Tests

```bash
pytest          # offline: 7 tests; live tests skipped without key
HEROSMS_API_KEY=... pytest tests/test_live_smoke.py   # balance + list activations only
```

## Project layout

```
docs/openapi.json          # official spec copy
docs/API_COVERAGE.md       # coverage checklist
skill/SKILL.md             # agent instructions
skill/scripts/herosms.py     # OpenAPI-driven CLI
tests/                     # pytest
```

## Known gaps

- Webhook URL configuration is in the HeroSMS account UI, not an outbound API in the spec.
- This repo does not implement a webhook receiver; see `skill/SKILL.md` for payload notes.

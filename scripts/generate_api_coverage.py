#!/usr/bin/env python3
"""Regenerate docs/API_COVERAGE.md from docs/openapi.json."""

from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SPEC = REPO_ROOT / "docs" / "openapi.json"
OUT = REPO_ROOT / "docs" / "API_COVERAGE.md"

TAG_ORDER = [
    "Activations",
    "Activation OTP",
    "Emails",
    "SMS-Activate",
    "Classifiers",
    "Statistics",
    "Webhooks",
]

HTTP_METHODS = {"get", "post", "put", "patch", "delete", "head", "options"}


def iter_path_operations(spec: dict):
    for path, path_item in spec.get("paths", {}).items():
        for method, op in path_item.items():
            if method not in HTTP_METHODS:
                continue
            yield {
                "operation_id": op["operationId"],
                "method": method,
                "path": path,
                "tags": op.get("tags") or [],
            }


def iter_webhook_operations(spec: dict):
    for hook_name, hook_item in spec.get("webhooks", {}).items():
        for method, op in hook_item.items():
            if method not in HTTP_METHODS:
                continue
            yield {
                "operation_id": op["operationId"],
                "method": method,
                "path": f"webhook:{hook_name}",
                "hook_name": hook_name,
                "tags": op.get("tags") or ["Webhooks"],
                "summary": op.get("summary", ""),
            }


def main() -> int:
    spec = json.loads(SPEC.read_text(encoding="utf-8"))
    by_tag: dict[str, list] = defaultdict(list)
    for op in iter_path_operations(spec):
        tag = op["tags"][0] if op["tags"] else "Untagged"
        note = "Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>`"
        by_tag[tag].append((op, note))

    lines = [
        "# HeroSMS API coverage",
        "",
        "Checklist generated from `docs/openapi.json` (official HeroSMS OpenAPI 3.2).",
        "",
        "| Covered | Method | Path | operationId | Notes |",
        "| --- | --- | --- | --- | --- |",
    ]

    total = 0
    for tag in TAG_ORDER:
        if tag == "Webhooks":
            continue
        if tag not in by_tag:
            continue
        lines.extend(["", f"## {tag}", ""])
        for op, note in sorted(by_tag[tag], key=lambda x: (x[0]["path"], x[0]["method"])):
            total += 1
            path = op["path"].replace("|", "\\|")
            lines.append(
                f"| yes | {op['method'].upper()} | `{path}` | `{op['operation_id']}` | {note} |"
            )

    # Webhooks: platform config + inbound event + skill receiver
    lines.extend(
        [
            "",
            "## Webhooks",
            "",
            "HeroSMS delivers webhooks to **your** HTTPS endpoints. There is **no outbound API** in the spec to register URLs.",
            "",
            "### Platform-side (account configuration)",
            "",
            "| Covered | Side | operationId | Notes |",
            "| --- | --- | --- | --- |",
            "| yes | **Platform** | `webhook-url-registration` | Configure up to **3 HTTPS URLs** in HeroSMS account personal information (see OpenAPI `tags.Webhooks`). Not callable via `herosms` CLI. |",
            "",
            "### Inbound events (HeroSMS → your server)",
            "",
            "| Covered | Method | Path | operationId | Notes |",
            "| --- | --- | --- | --- | --- |",
        ]
    )

    for op in iter_webhook_operations(spec):
        total += 2  # platform row counted once above + event row
        lines.append(
            f"| yes | {op['method'].upper()} | `{op['path']}` | `{op['operation_id']}` | "
            f"Inbound POST JSON per OpenAPI `webhooks.{op['hook_name']}`; see `skill/SKILL.md` (Webhooks) and `herosms-webhook` receiver. |"
        )

    lines.extend(
        [
            "",
            "### Skill-side (this repository)",
            "",
            "| Covered | Component | Notes |",
            "| --- | --- | --- |",
            "| yes | `herosms::webhook` | Parses/validates `smsIncoming` payload; OTP extraction helper. |",
            "| yes | `herosms-webhook` binary | Runnable sample HTTP server: `cargo run -p herosms-webhook --release`. TLS via reverse proxy. |",
            "",
            "## Summary",
            "",
            f"- **Total tracked items:** {total} (50 outbound API operations + 1 platform webhook config + 1 inbound webhook event + skill receiver/libs)",
            "- **Outbound CLI:** `cargo build -p herosms --release` → `target/release/herosms`",
            "- **Webhook receiver:** `cargo run -p herosms-webhook --release`",
            "- **Known gaps:** OpenAPI does not define HMAC/signature headers for webhooks; optional shared-secret header supported in sample receiver only.",
            "",
        ]
    )

    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

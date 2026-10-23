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
            }


def main() -> int:
    spec = json.loads(SPEC.read_text(encoding="utf-8"))
    by_tag: dict[str, list] = defaultdict(list)
    for op in iter_path_operations(spec):
        tag = op["tags"][0] if op["tags"] else "Untagged"
        note = "Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage)"
        by_tag[tag].append((op, note))

    lines = [
        "# HeroSMS API coverage",
        "",
        "Generated from `docs/openapi.json`. **Outbound client APIs** are invoked via the Rust `herosms` CLI.",
        "**Receiving OTP** in agent environments defaults to **polling** (`wait-otp`, `getStatusV2`, etc.), not webhooks.",
        "",
        "## Outbound client API (50 operations)",
        "",
        "| Covered | Method | Path | operationId | Notes |",
        "| --- | --- | --- | --- | --- |",
    ]

    outbound = 0
    for tag in TAG_ORDER:
        if tag == "Webhooks":
            continue
        if tag not in by_tag:
            continue
        lines.extend(["", f"### {tag}", ""])
        for op, note in sorted(by_tag[tag], key=lambda x: (x[0]["path"], x[0]["method"])):
            outbound += 1
            path = op["path"].replace("|", "\\|")
            lines.append(
                f"| yes | {op['method'].upper()} | `{path}` | `{op['operation_id']}` | {note} |"
            )

    lines.extend(
        [
            "",
            "## Webhooks",
            "",
            "Webhooks are **platform → your HTTPS endpoint**. Grok Bot / typical cloud agents **do not** host inbound HTTPS; mark **receive** as **N/A** there and use polling.",
            "",
            "### Platform pushes (HeroSMS → URL you configure)",
            "",
            "| Covered | Direction | operationId | We receive in bot? | Notes |",
            "| --- | --- | --- | --- | --- |",
            "| yes | Platform → customer URL | `webhook-url-registration` | N/A | Configure up to **3 HTTPS URLs** in account personal information. No client API. |",
        ]
    )

    for op in iter_webhook_operations(spec):
        lines.append(
            f"| yes | Platform → customer URL | `{op['operation_id']}` | **N/A** | "
            f"POST JSON per `webhooks.{op['hook_name']}`; documented in `skill/SKILL.md`. "
            f"Requires operator-owned public HTTPS. |"
        )

    lines.extend(
        [
            "",
            "### Skill / operator receive (optional, not a bot dependency)",
            "",
            "| Covered | Component | Bot environment | Notes |",
            "| --- | --- | --- | --- |",
            "| partial | `herosms wait-otp` | **yes** (polling) | **Default** receive path; uses `getStatusV2` / `getStatus` / REST OTP. |",
            "| partial | `herosms-webhook` sample | **N/A** | Optional reference server if operator runs TLS + public URL; not required for skill delivery. |",
            "| partial | `herosms::webhook` parse helpers | **N/A** | For custom receivers / optional forward path only. |",
            "",
            "## Summary",
            "",
            f"- **Outbound API operations:** {outbound} / 50 via `herosms call`",
            "- **OTP receive (agents):** polling (`herosms wait-otp`) — covered",
            "- **Webhook receive (agents):** N/A — documented only",
            "- **CLI:** `cargo build -p herosms --release`",
            "",
        ]
    )

    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {OUT} ({outbound} outbound ops)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

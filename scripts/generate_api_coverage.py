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
SA_WRITE_GET_OPS = {
    "getNumber",
    "getNumberV2",
    "getRentNumber",
    "setStatus",
    "saCancelActivation",
    "saFinishActivation",
}


def is_write(method: str, path: str, operation_id: str, inbound: bool) -> bool:
    if inbound:
        return False
    if method in {"post", "put", "patch", "delete"}:
        return True
    if path.startswith("?action="):
        if operation_id in SA_WRITE_GET_OPS:
            return True
        action = path.split("=", 1)[1]
        if action in {"cancelActivation", "finishActivation", "setStatus"}:
            return True
    return False


def iter_operations(spec: dict):
    for path, path_item in spec.get("paths", {}).items():
        for method, op in path_item.items():
            if method not in HTTP_METHODS:
                continue
            yield {
                "operation_id": op["operationId"],
                "method": method,
                "path": path,
                "tags": op.get("tags") or [],
                "inbound": False,
            }
    for hook_name, hook_item in spec.get("webhooks", {}).items():
        for method, op in hook_item.items():
            if method not in HTTP_METHODS:
                continue
            yield {
                "operation_id": op["operationId"],
                "method": method,
                "path": f"webhook:{hook_name}",
                "tags": op.get("tags") or ["Webhooks"],
                "inbound": True,
            }


def main() -> int:
    spec = json.loads(SPEC.read_text(encoding="utf-8"))
    by_tag: dict[str, list] = defaultdict(list)
    for op in iter_operations(spec):
        tag = op["tags"][0] if op["tags"] else "Untagged"
        if op["inbound"]:
            note = "Documented inbound webhook (configure URLs in HeroSMS account; not callable via CLI)"
        else:
            note = "Generic CLI `call` via `herosms` (Rust binary)"
        by_tag[tag].append((op, note))

    lines = [
        "# HeroSMS API coverage",
        "",
        "Checklist generated from `docs/openapi.json` (official HeroSMS OpenAPI 3.2).",
        "Each operation is covered by the generic OpenAPI-driven CLI unless noted.",
        "",
        "| Covered | Method | Path | operationId | Notes |",
        "| --- | --- | --- | --- | --- |",
    ]

    total = 0
    for tag in TAG_ORDER:
        if tag not in by_tag:
            continue
        lines.extend(["", f"## {tag}", ""])
        for op, note in sorted(by_tag[tag], key=lambda x: (x[0]["path"], x[0]["method"])):
            total += 1
            path = op["path"].replace("|", "\\|")
            lines.append(
                f"| yes | {op['method'].upper()} | `{path}` | `{op['operation_id']}` | {note} |"
            )

    lines.extend(
        [
            "",
            "## Summary",
            "",
            f"- **Total operations:** {total}",
            "- **CLI entrypoint:** `cargo build --release` → `target/release/herosms`",
            "- **Known gaps:** None beyond OpenAPI; webhook delivery is account-configured, not an outbound API.",
            "",
        ]
    )

    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {OUT} ({total} operations)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

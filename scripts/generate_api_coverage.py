#!/usr/bin/env python3
"""Regenerate docs/API_COVERAGE.md from docs/openapi.json."""

from __future__ import annotations

import sys
from collections import defaultdict
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT / "skill" / "scripts"))

from herosms import iter_operations, load_spec  # noqa: E402

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


def main() -> int:
    spec = load_spec(SPEC)
    by_tag: dict[str, list] = defaultdict(list)
    for op in iter_operations(spec):
        tag = op.tags[0] if op.tags else "Untagged"
        if op.is_inbound_webhook:
            note = "Documented inbound webhook (configure URLs in HeroSMS account; not callable via CLI)"
        else:
            note = "Generic CLI `call` via skill/scripts/herosms.py"
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
        for op, note in sorted(by_tag[tag], key=lambda x: (x[0].path, x[0].method)):
            total += 1
            covered = "yes"
            method = op.method.upper()
            path = op.path.replace("|", "\\|")
            lines.append(f"| {covered} | {method} | `{path}` | `{op.operation_id}` | {note} |")

    for tag in sorted(by_tag):
        if tag in TAG_ORDER:
            continue
        lines.extend(["", f"## {tag}", ""])
        for op, note in sorted(by_tag[tag], key=lambda x: (x[0].path, x[0].method)):
            total += 1
            lines.append(
                f"| yes | {op.method.upper()} | `{op.path}` | `{op.operation_id}` | {note} |"
            )

    lines.extend(
        [
            "",
            "## Summary",
            "",
            f"- **Total operations:** {total}",
            "- **CLI entrypoint:** `python skill/scripts/herosms.py`",
            "- **Known gaps:** None beyond OpenAPI; webhook delivery is account-configured, not an outbound API.",
            "",
        ]
    )

    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {OUT} ({total} operations)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

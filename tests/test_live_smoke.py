"""Optional live smoke tests (skipped without HEROSMS_API_KEY)."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
CLI = REPO_ROOT / "skill" / "scripts" / "herosms.py"

pytestmark = pytest.mark.skipif(
    not os.environ.get("HEROSMS_API_KEY"),
    reason="HEROSMS_API_KEY not set",
)


def run_call(operation_id: str, *extra: str) -> dict:
    proc = subprocess.run(
        [sys.executable, str(CLI), "call", operation_id, *extra],
        check=False,
        capture_output=True,
        text=True,
        cwd=REPO_ROOT,
    )
    assert proc.returncode == 0, proc.stderr + proc.stdout
    return json.loads(proc.stdout)


def test_live_get_balance() -> None:
    out = run_call("getBalance", "--auth", "query")
    body = out.get("body", "")
    assert "ACCESS_BALANCE" in body or "BALANCE" in body.upper()


def test_live_rest_list_activations() -> None:
    out = run_call("getActiveActivations", "--query", "size=1")
    assert out["status"] == 200
    assert "json" in out or "data" in out.get("body", "")

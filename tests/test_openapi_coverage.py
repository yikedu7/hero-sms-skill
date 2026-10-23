"""Offline tests: CLI knows every OpenAPI operationId."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
SPEC = REPO_ROOT / "docs" / "openapi.json"
CLI = REPO_ROOT / "skill" / "scripts" / "herosms.py"


def openapi_operation_ids() -> set[str]:
    spec = json.loads(SPEC.read_text(encoding="utf-8"))
    ids: set[str] = set()
    http_methods = {"get", "post", "put", "patch", "delete", "head", "options"}
    for _path, path_item in spec.get("paths", {}).items():
        for method, op in path_item.items():
            if method in http_methods:
                ids.add(op["operationId"])
    for _name, hook in spec.get("webhooks", {}).items():
        for method, op in hook.items():
            if method in http_methods:
                ids.add(op["operationId"])
    return ids


def cli_operation_ids() -> set[str]:
    proc = subprocess.run(
        [sys.executable, str(CLI), "operations-json"],
        check=True,
        capture_output=True,
        text=True,
        cwd=REPO_ROOT,
    )
    ops = json.loads(proc.stdout)
    return {item["operationId"] for item in ops}


def test_cli_covers_all_openapi_operations() -> None:
    spec_ids = openapi_operation_ids()
    cli_ids = cli_operation_ids()
    missing = spec_ids - cli_ids
    extra = cli_ids - spec_ids
    assert not missing, f"CLI missing operationIds: {sorted(missing)}"
    assert not extra, f"CLI has unknown operationIds: {sorted(extra)}"


def test_openapi_operation_count_is_51() -> None:
    """Guard against spec drift (50 path ops + 1 inbound webhook)."""
    assert len(openapi_operation_ids()) == 51


@pytest.mark.parametrize(
    "operation_id,expect_write",
    [
        ("getBalance", False),
        ("getNumber", True),
        ("buyActivationBatch", True),
        ("getActiveActivations", False),
    ],
)
def test_write_classification(operation_id: str, expect_write: bool) -> None:
    sys.path.insert(0, str(REPO_ROOT / "skill" / "scripts"))
    from herosms import index_operations, load_spec  # noqa: WPS433

    op = index_operations(load_spec(SPEC))[operation_id]
    assert op.is_write is expect_write


def test_dry_run_does_not_require_api_key() -> None:
    env = {k: v for k, v in __import__("os").environ.items() if k != "HEROSMS_API_KEY"}
    proc = subprocess.run(
        [sys.executable, str(CLI), "call", "getBalance", "--dry-run"],
        check=True,
        capture_output=True,
        text=True,
        cwd=REPO_ROOT,
        env=env,
    )
    payload = json.loads(proc.stdout)
    assert payload["operationId"] == "getBalance"
    assert "handler_api.php" in payload["request"]["url"]

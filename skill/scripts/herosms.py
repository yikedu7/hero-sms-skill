#!/usr/bin/env python3
"""Generic HeroSMS API CLI driven by docs/openapi.json."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SPEC = REPO_ROOT / "docs" / "openapi.json"

REST_BASE = "https://hero-sms.com/api/v1"
SA_BASE = "https://hero-sms.com/stubs/handler_api.php"

HTTP_METHODS = frozenset({"get", "post", "put", "patch", "delete", "head", "options"})

# GET actions that spend money or mutate state (SMS-Activate compatibility).
SA_WRITE_GET_ACTIONS = frozenset(
    {
        "getNumber",
        "getNumberV2",
        "getRentNumber",
        "setStatus",
        "saCancelActivation",
        "saFinishActivation",
    }
)

REDACT = "***REDACTED***"


@dataclass(frozen=True)
class Operation:
    operation_id: str
    method: str
    path: str
    tags: Tuple[str, ...]
    summary: str
    parameters: Tuple[Dict[str, Any], ...]
    request_body: Optional[Dict[str, Any]]
    security: Tuple[Dict[str, Any], ...]
    servers: Tuple[Dict[str, Any], ...]
    source: str  # "paths" | "webhooks"
    is_inbound_webhook: bool

    @property
    def is_write(self) -> bool:
        if self.is_inbound_webhook:
            return False
        if self.method in ("post", "put", "patch", "delete"):
            return True
        if self.path.startswith("?action="):
            action = self.path.split("=", 1)[1]
            if self.operation_id in SA_WRITE_GET_ACTIONS:
                return True
            if action in ("cancelActivation", "finishActivation", "setStatus"):
                return True
        return False


def load_spec(path: Path) -> Dict[str, Any]:
    with path.open(encoding="utf-8") as fh:
        return json.load(fh)


def resolve_ref(spec: Dict[str, Any], node: Any) -> Any:
    if not isinstance(node, dict):
        return node
    if "$ref" in node:
        ref = node["$ref"]
        if not ref.startswith("#/"):
            raise ValueError(f"Unsupported ref: {ref}")
        parts = ref[2:].split("/")
        cur: Any = spec
        for part in parts:
            part = part.replace("~1", "/").replace("~0", "~")
            cur = cur[part]
        return resolve_ref(spec, cur)
    return node


def resolve_parameters(spec: Dict[str, Any], raw: Optional[List[Any]]) -> Tuple[Dict[str, Any], ...]:
    if not raw:
        return ()
    out: List[Dict[str, Any]] = []
    for item in raw:
        out.append(resolve_ref(spec, item))
    return tuple(out)


def resolve_request_body(spec: Dict[str, Any], raw: Optional[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    if not raw:
        return None
    return resolve_ref(spec, raw)


def iter_operations(spec: Dict[str, Any]) -> Iterable[Operation]:
    for path, path_item in spec.get("paths", {}).items():
        for method, op in path_item.items():
            if method not in HTTP_METHODS:
                continue
            yield Operation(
                operation_id=op.get("operationId") or f"{method}:{path}",
                method=method.lower(),
                path=path,
                tags=tuple(op.get("tags") or ()),
                summary=op.get("summary") or "",
                parameters=resolve_parameters(spec, op.get("parameters")),
                request_body=resolve_request_body(spec, op.get("requestBody")),
                security=tuple(op.get("security") or spec.get("security") or ()),
                servers=tuple(op.get("servers") or ()),
                source="paths",
                is_inbound_webhook=False,
            )

    for _hook_name, hook_item in spec.get("webhooks", {}).items():
        for method, op in hook_item.items():
            if method not in HTTP_METHODS:
                continue
            yield Operation(
                operation_id=op.get("operationId") or f"webhook:{method}",
                method=method.lower(),
                path=f"webhook:{_hook_name}",
                tags=tuple(op.get("tags") or ("Webhooks",)),
                summary=op.get("summary") or "",
                parameters=resolve_parameters(spec, op.get("parameters")),
                request_body=resolve_request_body(spec, op.get("requestBody")),
                security=(),
                servers=(),
                source="webhooks",
                is_inbound_webhook=True,
            )


def index_operations(spec: Dict[str, Any]) -> Dict[str, Operation]:
    idx: Dict[str, Operation] = {}
    for op in iter_operations(spec):
        idx[op.operation_id] = op
    return idx


def choose_base_url(op: Operation) -> str:
    if op.servers:
        return op.servers[0]["url"].rstrip("/")
    if op.path.startswith("?action="):
        return SA_BASE
    return REST_BASE


def build_url(base: str, path: str, path_params: Dict[str, Any], query: Dict[str, Any]) -> str:
    if path.startswith("?action="):
        url_path = ""
        action_query = path[1:]  # action=...
        merged = dict(query)
        for part in action_query.split("&"):
            if "=" in part:
                k, v = part.split("=", 1)
                merged.setdefault(k, v)
        query = merged
    else:
        url_path = path
        for key, value in path_params.items():
            url_path = url_path.replace("{" + key + "}", urllib.parse.quote(str(value), safe=""))

    if url_path:
        url = base.rstrip("/") + url_path
    else:
        url = base

    if query:
        # Flatten list values for array query params.
        flat: List[Tuple[str, str]] = []
        for key, value in query.items():
            if isinstance(value, list):
                for item in value:
                    flat.append((key, str(item)))
            else:
                flat.append((key, str(value)))
        qs = urllib.parse.urlencode(flat, doseq=True)
        sep = "&" if "?" in url else "?"
        url = f"{url}{sep}{qs}"
    return url


def security_schemes(spec: Dict[str, Any]) -> Dict[str, Any]:
    return spec.get("components", {}).get("securitySchemes", {})


def pick_security(op: Operation, spec: Dict[str, Any]) -> Tuple[str, ...]:
    if not op.security:
        return ("apiKeyAuth",)
    req = op.security[0]
    return tuple(req.keys())


def apply_auth(
    op: Operation,
    spec: Dict[str, Any],
    headers: Dict[str, str],
    query: Dict[str, Any],
    api_key: str,
    auth_style: str,
) -> None:
    schemes = security_schemes(spec)
    sec_names = pick_security(op, spec)

    style = auth_style
    if style == "auto":
        if "apiKeyQuery" in sec_names and "apiKeyAuth" not in sec_names:
            style = "query"
        else:
            style = "authorization-apikey"

    if style == "query":
        query.setdefault("api_key", api_key)
    elif style == "x-api-key":
        headers["x-api-key"] = api_key
    elif style == "authorization-bearer":
        headers["Authorization"] = f"Bearer {api_key}"
    elif style == "authorization-apikey":
        headers["Authorization"] = f"ApiKey {api_key}"
    else:
        raise SystemExit(f"Unknown auth style: {auth_style}")

    # Validate against OpenAPI scheme names when auto.
    if auth_style == "auto":
        for name in sec_names:
            scheme = schemes.get(name, {})
            if scheme.get("in") == "query" and "api_key" not in query:
                query["api_key"] = api_key
            if scheme.get("in") == "header" and scheme.get("name") == "Authorization":
                if "Authorization" not in headers:
                    headers["Authorization"] = f"ApiKey {api_key}"


def split_params(
    op: Operation, path_params: Dict[str, Any], query_params: Dict[str, Any]
) -> Tuple[Dict[str, Any], Dict[str, Any]]:
    return dict(path_params), dict(query_params)


def redact_request(url: str, headers: Dict[str, str], body: Optional[bytes]) -> Dict[str, Any]:
    safe_headers = dict(headers)
    for h in ("Authorization", "x-api-key", "X-Api-Key"):
        if h in safe_headers:
            safe_headers[h] = REDACT
    safe_url = url
    parsed = urllib.parse.urlsplit(url)
    q = urllib.parse.parse_qsl(parsed.query, keep_blank_values=True)
    safe_q = [(k, REDACT if k == "api_key" else v) for k, v in q]
    safe_url = urllib.parse.urlunsplit(
        (parsed.scheme, parsed.netloc, parsed.path, urllib.parse.urlencode(safe_q), parsed.fragment)
    )
    return {
        "url": safe_url,
        "headers": safe_headers,
        "body": body.decode("utf-8") if body else None,
    }


def parse_kv_pairs(pairs: List[str]) -> Dict[str, Any]:
    out: Dict[str, Any] = {}
    for pair in pairs:
        if "=" not in pair:
            raise SystemExit(f"Expected key=value, got: {pair}")
        key, value = pair.split("=", 1)
        if key.endswith("[]"):
            base = key
            out.setdefault(base, []).append(value)
        else:
            out[key] = value
    return out


def execute_request(
    method: str,
    url: str,
    headers: Dict[str, str],
    body: Optional[bytes],
    timeout: float = 60.0,
) -> Tuple[int, str, Dict[str, str]]:
    req = urllib.request.Request(url, data=body, method=method.upper())
    for k, v in headers.items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read()
            text = raw.decode("utf-8", errors="replace")
            return resp.status, text, dict(resp.headers)
    except urllib.error.HTTPError as exc:
        raw = exc.read()
        text = raw.decode("utf-8", errors="replace")
        return exc.code, text, dict(exc.headers)


def cmd_list(spec: Dict[str, Any], args: argparse.Namespace) -> int:
    ops = sorted(iter_operations(spec), key=lambda o: (o.tags[0] if o.tags else "", o.operation_id))
    for op in ops:
        if args.tag and args.tag not in op.tags:
            continue
        tag = op.tags[0] if op.tags else "-"
        kind = "inbound" if op.is_inbound_webhook else ("write" if op.is_write else "read")
        print(f"{op.operation_id}\t{op.method.upper():6}\t{tag}\t{kind}\t{op.summary}")
    return 0


def cmd_describe(spec: Dict[str, Any], args: argparse.Namespace) -> int:
    idx = index_operations(spec)
    op = idx.get(args.operation_id)
    if not op:
        raise SystemExit(f"Unknown operationId: {args.operation_id}")
    payload = {
        "operationId": op.operation_id,
        "method": op.method.upper(),
        "path": op.path,
        "tags": list(op.tags),
        "summary": op.summary,
        "baseUrl": choose_base_url(op),
        "isWrite": op.is_write,
        "isInboundWebhook": op.is_inbound_webhook,
        "parameters": op.parameters,
        "requestBody": op.request_body,
        "security": list(op.security),
    }
    print(json.dumps(payload, indent=2))
    return 0


def cmd_call(spec: Dict[str, Any], args: argparse.Namespace) -> int:
    idx = index_operations(spec)
    op = idx.get(args.operation_id)
    if not op:
        raise SystemExit(f"Unknown operationId: {args.operation_id}")

    if op.is_inbound_webhook:
        raise SystemExit(
            f"{op.operation_id} is an inbound webhook (HeroSMS calls your server). "
            "See skill/SKILL.md and docs/API_COVERAGE.md."
        )

    api_key = os.environ.get("HEROSMS_API_KEY", "").strip()
    if not api_key and not args.dry_run:
        raise SystemExit("HEROSMS_API_KEY is not set. Use --dry-run to preview the request.")

    path_params = parse_kv_pairs(args.path or [])
    query_params = parse_kv_pairs(args.query or [])

    path_params, query_params = split_params(op, path_params, query_params)

    body_bytes: Optional[bytes] = None
    headers: Dict[str, str] = {"Accept": "application/json"}
    if op.request_body:
        if args.body_file:
            body_bytes = Path(args.body_file).read_bytes()
        elif args.body_json is not None:
            body_bytes = args.body_json.encode("utf-8")
        elif op.method in ("post", "put", "patch"):
            body_bytes = b"{}"
        content_types = (
            op.request_body.get("content", {}) if op.request_body else {}
        )
        if "application/json" in content_types:
            headers["Content-Type"] = "application/json"
        elif content_types:
            headers["Content-Type"] = next(iter(content_types.keys()))

    if api_key:
        apply_auth(op, spec, headers, query_params, api_key, args.auth)

    base = choose_base_url(op)
    url = build_url(base, op.path, path_params, query_params)

    if op.is_write and not args.yes and not args.dry_run:
        args.dry_run = True
        print("Write operation: showing dry-run preview. Pass --yes to execute.", file=sys.stderr)

    if args.dry_run:
        preview = {
            "operationId": op.operation_id,
            "method": op.method.upper(),
            "request": redact_request(url, headers, body_bytes),
        }
        print(json.dumps(preview, indent=2))
        return 0

    status, text, _ = execute_request(op.method, url, headers, body_bytes)
    out = {"status": status, "body": text}
    try:
        out["json"] = json.loads(text)
    except json.JSONDecodeError:
        pass
    print(json.dumps(out, indent=2))
    return 0 if 200 <= status < 300 else 1


def cmd_operations_json(spec: Dict[str, Any], _args: argparse.Namespace) -> int:
    ops = [
        {
            "operationId": op.operation_id,
            "method": op.method,
            "path": op.path,
            "tags": list(op.tags),
        }
        for op in iter_operations(spec)
    ]
    print(json.dumps(ops, indent=2))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="HeroSMS OpenAPI-driven CLI")
    parser.add_argument(
        "--spec",
        type=Path,
        default=DEFAULT_SPEC,
        help=f"OpenAPI JSON path (default: {DEFAULT_SPEC})",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    p_list = sub.add_parser("list", help="List operationIds from OpenAPI")
    p_list.add_argument("--tag", help="Filter by OpenAPI tag")
    p_list.set_defaults(func=cmd_list)

    p_desc = sub.add_parser("describe", help="Show one operation")
    p_desc.add_argument("operation_id")
    p_desc.set_defaults(func=cmd_describe)

    p_call = sub.add_parser("call", help="Invoke an operation")
    p_call.add_argument("operation_id")
    p_call.add_argument("--path", action="append", help="Path param key=value (repeatable)")
    p_call.add_argument("--query", action="append", help="Query param key=value (repeatable)")
    p_call.add_argument("--body-json", help="JSON request body string")
    p_call.add_argument("--body-file", help="Path to JSON request body file")
    p_call.add_argument(
        "--auth",
        default=os.environ.get("HEROSMS_AUTH_STYLE", "auto"),
        choices=["auto", "authorization-apikey", "authorization-bearer", "x-api-key", "query"],
        help="API key placement (default: auto from OpenAPI security)",
    )
    p_call.add_argument("--dry-run", action="store_true", help="Print request without sending")
    p_call.add_argument("--yes", action="store_true", help="Execute write operations (spend money / mutate)")
    p_call.set_defaults(func=cmd_call)

    p_ops = sub.add_parser("operations-json", help="Emit all operations as JSON (for tests)")
    p_ops.set_defaults(func=cmd_operations_json)

    return parser


def main(argv: Optional[List[str]] = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    spec = load_spec(args.spec)
    return args.func(spec, args)


if __name__ == "__main__":
    raise SystemExit(main())

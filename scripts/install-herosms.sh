#!/usr/bin/env bash
# Install the herosms CLI from GitHub Releases (hero-sms-skill).
# Agent-friendly: no Rust toolchain required.
set -euo pipefail

REPO="${HEROSMS_INSTALL_REPO:-yikedu7/hero-sms-skill}"
VERSION="${HEROSMS_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-${HOME}/.local/bin}"

detect_asset_suffix() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "${os}-${arch}" in
    Linux-x86_64) echo "x86_64-unknown-linux-musl" ;;
    Linux-aarch64 | Linux-arm64) echo "aarch64-unknown-linux-gnu" ;;
    Darwin-arm64) echo "aarch64-apple-darwin" ;;
    Darwin-x86_64) echo "x86_64-apple-darwin" ;;
    *)
      echo "unsupported platform: ${os} ${arch}" >&2
      echo "Build from source: cargo build -p herosms --release" >&2
      exit 1
      ;;
  esac
}

resolve_version() {
  if [[ "${VERSION}" != "latest" ]]; then
    echo "${VERSION#v}"
    return
  fi
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": "v\([^"]*\)".*/\1/p' \
    | head -1
}

main() {
  local suffix ver asset url tmp
  suffix="$(detect_asset_suffix)"
  ver="$(resolve_version)"
  if [[ -z "${ver}" ]]; then
    echo "Could not resolve release version (no releases yet?)." >&2
    echo "Fallback: cargo build -p herosms --release" >&2
    exit 1
  fi
  asset="herosms-${ver}-${suffix}"
  url="https://github.com/${REPO}/releases/download/v${ver}/${asset}"

  mkdir -p "${INSTALL_DIR}"
  tmp="$(mktemp)"
  trap 'rm -f "${tmp}"' EXIT

  echo "Downloading ${url}"
  curl -fsSL -o "${tmp}" "${url}"
  install -m 755 "${tmp}" "${INSTALL_DIR}/herosms"

  if ! command -v herosms >/dev/null 2>&1; then
    echo "Installed to ${INSTALL_DIR}/herosms — add ${INSTALL_DIR} to PATH if needed."
  else
    echo "Installed: $(command -v herosms) ($(herosms --version 2>/dev/null || echo "ok"))"
  fi
}

main "$@"

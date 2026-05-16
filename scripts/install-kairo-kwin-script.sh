#!/usr/bin/env bash
set -euo pipefail

PLUGIN_ID="kairo-keep-above"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PACKAGE_DIR="${PROJECT_DIR}/packaging/kde/kwin/${PLUGIN_ID}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$1" >&2
    exit 1
  fi
}

require_command kpackagetool6
require_command kwriteconfig6
require_command qdbus-qt6

if [ ! -f "${PACKAGE_DIR}/metadata.json" ]; then
  printf 'KWin script package not found: %s\n' "${PACKAGE_DIR}" >&2
  exit 1
fi

if kpackagetool6 --type=KWin/Script --list | grep -q "${PLUGIN_ID}"; then
  kpackagetool6 --type=KWin/Script --upgrade "${PACKAGE_DIR}"
else
  kpackagetool6 --type=KWin/Script --install "${PACKAGE_DIR}"
fi

kwriteconfig6 --file kwinrc --group Plugins --key "${PLUGIN_ID}Enabled" true
qdbus-qt6 org.kde.KWin /KWin reconfigure

printf 'Installed and enabled KWin script: %s\n' "${PLUGIN_ID}"
printf 'Restart Kairo, then show the floating timer or mini timer to verify keep-above behavior.\n'

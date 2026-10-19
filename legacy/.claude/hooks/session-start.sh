#!/bin/bash
# SessionStart hook for Claude Code cloud sessions: installs the tools the commit gate needs
# (Godot 4.7 Linux editor binary and PowerShell 7) so check_project.ps1 and tools/fc.sh run.
# Local (Windows) sessions skip it. Idempotent: every step is skipped when already done, so
# when the cloud environment's setup script already installed the tools (preferred: done once
# and cached), this hook only links godot, exports $GODOT and imports a fresh checkout.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

GODOT_VERSION="${GODOT_VERSION:-4.7.2}"
GODOT_DIR="/opt/godot/${GODOT_VERSION}"
GODOT_BIN="${GODOT_DIR}/godot"
SUDO=""
[ "$(id -u)" -ne 0 ] && SUDO="sudo"

log() { echo "[session-start] $*" >&2; }

# --- PowerShell 7 (Microsoft apt repo) ---------------------------------------
if ! command -v pwsh >/dev/null 2>&1; then
  log "installing PowerShell"
  . /etc/os-release
  tmp="$(mktemp -d)"
  curl -fsSL -o "$tmp/ms.deb" "https://packages.microsoft.com/config/ubuntu/${VERSION_ID}/packages-microsoft-prod.deb"
  $SUDO dpkg -i "$tmp/ms.deb" >/dev/null
  rm -rf "$tmp"
  $SUDO apt-get update -qq >/dev/null
  $SUDO DEBIAN_FRONTEND=noninteractive apt-get install -y -qq powershell >/dev/null
fi

# --- Rendering a game window off-screen (Xvfb + Mesa software OpenGL) --------
# Lets a session run the game for real and screenshot it:
#   xvfb-run -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 ...
if ! command -v xvfb-run >/dev/null 2>&1; then
  log "installing Xvfb and Mesa"
  $SUDO apt-get update -qq >/dev/null
  $SUDO DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
    xvfb xauth libgl1-mesa-dri libglx-mesa0 libxcursor1 libxinerama1 libxi6 libxrandr2 >/dev/null
fi

# --- Godot editor (official Linux build) -------------------------------------
if [ ! -x "$GODOT_BIN" ]; then
  log "installing Godot ${GODOT_VERSION}"
  name="Godot_v${GODOT_VERSION}-stable_linux.x86_64"
  urls=(
    "https://github.com/godotengine/godot/releases/download/${GODOT_VERSION}-stable/${name}.zip"
    "https://downloads.godotengine.org/?version=${GODOT_VERSION}&flavor=stable&slug=linux.x86_64.zip&platform=linux.64"
  )
  tmp="$(mktemp -d)"
  ok=0
  for u in "${urls[@]}"; do
    if curl -fsSL --retry 3 -o "$tmp/godot.zip" "$u"; then ok=1; break; fi
  done
  if [ "$ok" -ne 1 ]; then
    rm -rf "$tmp"
    log "WARNING: could not download Godot (github.com release assets and downloads.godotengine.org"
    log "         are blocked by this environment's network policy). Allow one of those hosts in the"
    log "         cloud environment's network settings; the gate cannot run until Godot is installed."
    exit 0
  fi
  python3 -c "import sys, zipfile; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" "$tmp/godot.zip" "$tmp"
  $SUDO mkdir -p "$GODOT_DIR"
  $SUDO install -m 0755 "$tmp/${name}" "$GODOT_BIN"
  rm -rf "$tmp"
fi

# check_project.ps1 looks for `godot` on PATH; tools/fc.sh reads $GODOT.
$SUDO ln -sf "$GODOT_BIN" /usr/local/bin/godot
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  echo "export GODOT=\"${GODOT_BIN}\"" >> "$CLAUDE_ENV_FILE"
fi

# First headless editor run imports assets and builds .godot/ (class cache), which the parse
# and suite runs need. Only on a fresh checkout.
if [ ! -d "${CLAUDE_PROJECT_DIR:-.}/.godot" ]; then
  log "importing project (first run)"
  (cd "${CLAUDE_PROJECT_DIR:-.}" && timeout 300 "$GODOT_BIN" --headless --import >/dev/null 2>&1) || \
    log "WARNING: initial import exited non-zero; run check_project.ps1 -Fast to see why"
fi

log "ready: $("$GODOT_BIN" --version 2>/dev/null | head -1), $(pwsh -v)"

#!/bin/bash
# SessionStart hook for Claude Code cloud sessions: installs the tools the commit gate needs (Rust stable for the Aegis
# engine; Godot 4.7 Linux editor binary for the parity reference in legacy/; PowerShell 7 for check_project.ps1) so
# tools/fc.sh gate and check_project.ps1 run. Local (Windows) sessions skip it. Idempotent: every step is skipped when
# already done, so when the cloud environment's setup script already installed the tools (preferred: done once and
# cached), this hook only links godot, exports $GODOT and imports a fresh legacy/ checkout.
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

# --- Rust (rustup, stable): the engine gate ----------------------------------
# rust-toolchain.toml pins the desktop's Windows toolchain, and .cargo/config.toml a Windows target
# folder: on Linux both are overridden through the environment.
if ! command -v cargo >/dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo" ]; then
  log "installing Rust (stable)"
  curl -fsSL --retry 3 https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable >/dev/null 2>&1 || \
    log "WARNING: could not install Rust (sh.rustup.rs / static.rust-lang.org blocked?); the gate cannot run until cargo exists"
fi
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  {
    echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    echo "export RUSTUP_TOOLCHAIN=stable"
    echo "export CARGO_TARGET_DIR=\"${CLAUDE_PROJECT_DIR:-$PWD}/target\""
  } >> "$CLAUDE_ENV_FILE"
fi

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

# --- Rendering a game window off-screen (Xvfb + Mesa software Vulkan / OpenGL) ---
# Meant to let a session run the game GUI (emberwake) and screenshot it through its agent link (not yet tried in a cloud container;
# wgpu needs Mesa's software Vulkan or OpenGL there):
#   xvfb-run -a -s "-screen 0 1280x720x24" "$CARGO_TARGET_DIR/release/emberwake" screen=runner
if ! command -v xvfb-run >/dev/null 2>&1; then
  log "installing Xvfb and Mesa"
  $SUDO apt-get update -qq >/dev/null
  $SUDO DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
    xvfb xauth libgl1-mesa-dri libglx-mesa0 mesa-vulkan-drivers libxcursor1 libxinerama1 libxi6 libxrandr2 libxkbcommon-x11-0 >/dev/null
fi

# --- Godot editor (official Linux build): the frozen reference in legacy/ ------
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
    log "         are blocked by this environment's network policy). The engine gate still runs; the Godot parity"
    log "         reference (tools/fc.sh parity, games/emberwake/parity/*.sh) needs one of those hosts allowed."
    exit 0
  fi
  python3 -c "import sys, zipfile; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" "$tmp/godot.zip" "$tmp"
  $SUDO mkdir -p "$GODOT_DIR"
  $SUDO install -m 0755 "$tmp/${name}" "$GODOT_BIN"
  rm -rf "$tmp"
fi

# games/emberwake/parity/*.sh read $GODOT (else godot on PATH).
$SUDO ln -sf "$GODOT_BIN" /usr/local/bin/godot
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  echo "export GODOT=\"${GODOT_BIN}\"" >> "$CLAUDE_ENV_FILE"
fi

# The parity tools run the frozen Godot copy in legacy/; its first headless run builds legacy/.godot (class cache).
LEGACY="${CLAUDE_PROJECT_DIR:-.}/legacy"
if [ -f "$LEGACY/project.godot" ] && [ ! -d "$LEGACY/.godot" ]; then
  log "importing legacy/ (first run)"
  timeout 300 "$GODOT_BIN" --headless --path "$LEGACY" --import >/dev/null 2>&1 || \
    log "WARNING: initial import of legacy/ exited non-zero; run games/emberwake/parity/export.sh to see why"
fi

log "ready: $("$GODOT_BIN" --version 2>/dev/null | head -1), $(pwsh -v), $("$HOME/.cargo/bin/cargo" --version 2>/dev/null || cargo --version 2>/dev/null || echo 'no cargo')"

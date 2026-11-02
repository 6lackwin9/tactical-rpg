#!/bin/bash
# SessionStart hook for Claude Code cloud sessions: installs the tools the commit gate needs (Rust stable for the Aegis
# engine; PowerShell 7 for check_project.ps1) so
# tools/fc.sh gate and check_project.ps1 run. Local (Windows) sessions skip it. Idempotent: every step is skipped when
# already done, so when the cloud environment's setup script already installed the tools (preferred: done once and
# cached), this hook installs only what is missing (no Godot: the parity suites were retired, CLN-42).
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

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

log "ready: $(pwsh -v), $("$HOME/.cargo/bin/cargo" --version 2>/dev/null || cargo --version 2>/dev/null || echo 'no cargo')"

#!/usr/bin/env bash
# CPU1 (user 2026-11-04): sourced by pgate.sh, stage.sh and flush.sh. `heavy_start <name>` lowers this shell's priority
# (children inherit it) and takes the machine-wide queue, so one partial or full gate runs at a time. The lock frees
# itself when the holder exits; one left by a dead process is taken over. Waits up to 40 minutes, then runs anyway.
HEAVY_LOCK="${TEMP:-/tmp}/aegis-heavy.lock"
heavy_start() {
  export CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4   # branches cut before the cargo config had the caps
  [ -r /proc/$$/winpid ] && powershell -NoProfile -Command "(Get-Process -Id $(cat /proc/$$/winpid)).PriorityClass='BelowNormal'" 2>/dev/null
  local waited=0 said=
  while ! mkdir "$HEAVY_LOCK" 2>/dev/null; do
    local who; who=$(cat "$HEAVY_LOCK/who" 2>/dev/null)
    if [ -n "$who" ] && ! kill -0 "${who%% *}" 2>/dev/null; then rm -rf "$HEAVY_LOCK"; continue; fi
    # No owner name a minute on: the holder died between taking the lock and naming itself.
    if [ -z "$who" ] && [ -n "$(find "$HEAVY_LOCK" -maxdepth 0 -mmin +1 2>/dev/null)" ]; then rm -rf "$HEAVY_LOCK"; continue; fi
    [ -n "$said" ] || { echo "queued behind: ${who#* }"; said=1; }
    [ "$waited" -lt 2400 ] || { echo "queue wait over 40 min: running anyway"; return 0; }
    sleep 10; waited=$((waited + 10))
  done
  echo "$$ $1 ($(basename "$PWD"))" > "$HEAVY_LOCK/who"
  HEAVY_OWNED=1
  trap heavy_end EXIT
  [ "$waited" -eq 0 ] || echo "queue wait ${waited}s"
}
heavy_end() { [ -n "${HEAVY_OWNED:-}" ] && rm -rf "$HEAVY_LOCK"; HEAVY_OWNED=; }

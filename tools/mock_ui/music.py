"""Import a generated music track (Flow Music / Lyria) as a game track (ART_UI_STUDIO 3.9).

The engine plays `music/<key>.ogg` looped from its start with a 1 s crossfade (MUSIC1). Generated tracks are whole
songs (an intro build, a fade-out ending), so a looping track needs its steady middle cut into a seamless loop:

1. Decode with the ffmpeg that `imageio-ffmpeg` bundles (`pip install imageio-ffmpeg`), 44.1 kHz stereo.
2. Find the steady part: the longest run of 0.25 s blocks within 6 dB of the track's typical loud level, which
   drops the intro build and the fade-out.
3. Find the loop inside it: a start and an end where the music just before the end matches the music just before the
   start (spectral bands over 2 s), so jumping from the end back to the start continues naturally; among good seams
   the longest loop wins. The end is then aligned to the sample by cross-correlation and the last 0.5 s is blended
   (equal power) into the audio that leads into the start.
4. Match loudness to the game's existing tracks (their median RMS) and limit peaks to -1 dBFS.
5. Write Ogg Vorbis q5 and, with --preview, an MP3 of the seam (the loop's last seconds running into its first) to
   check by ear.

`--no-loop` keeps the whole track (credits, stingers): steps 1, 4 and 5 only.

`--whole [--overlap 8]` loops the whole song instead of a section (Tushar, 2026-11-11: section cuts of a 3-minute
menu theme sounded abrupt and lost its shape): trailing silence trimmed, the fade-out laid over the intro, so a restart
from the file's start runs out of the fade into the opening. The engine always restarts a track at its start, so an
intro that plays once and a loop body after it needs loop points in the engine (requested in ART_UI_STUDIO 3.9).

    python -m tools.mock_ui.music IN.mp3 --out games/emberwake/assets/audio/music/menu_theme.ogg --preview SEAM.mp3
"""
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

import numpy as np

SR = 44100
ROOT = Path(__file__).resolve().parents[2]
REFERENCE_TRACKS = [ROOT / "kits/rpg/assets/audio/music", ROOT / "games/emberwake/assets/audio/music"]


def ffmpeg() -> str:
    try:
        import imageio_ffmpeg
    except ImportError as e:  # pragma: no cover - environment
        raise SystemExit("needs imageio-ffmpeg: python -m pip install imageio-ffmpeg") from e
    return imageio_ffmpeg.get_ffmpeg_exe()


def decode(path: Path) -> np.ndarray:
    raw = subprocess.run([ffmpeg(), "-v", "error", "-i", str(path), "-ac", "2", "-ar", str(SR), "-f", "f32le", "-"],
                         capture_output=True, check=True).stdout
    return np.frombuffer(raw, np.float32).reshape(-1, 2).astype(np.float64)


def encode(x: np.ndarray, path: Path, codec: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    args = ["-c:a", "libvorbis", "-q:a", "5"] if codec == "ogg" else ["-c:a", "libmp3lame", "-q:a", "2"]
    subprocess.run([ffmpeg(), "-v", "error", "-y", "-f", "f32le", "-ar", str(SR), "-ac", "2", "-i", "-", *args,
                    str(path)], input=x.astype(np.float32).tobytes(), check=True)


def db(v: float) -> float:
    return 20 * np.log10(max(float(v), 1e-9))


def rms(x: np.ndarray) -> float:
    return float(np.sqrt(np.mean(x ** 2)))


def block_db(mono: np.ndarray, block: int) -> np.ndarray:
    n = len(mono) // block
    return 20 * np.log10(np.maximum(np.sqrt((mono[: n * block].reshape(n, block) ** 2).mean(1)), 1e-9))


def steady_part(mono: np.ndarray, block_s: float = 0.25, drop_db: float = 6.0, dip_s: float = 2.0):
    """(start, end) samples of the longest steady stretch: blocks within drop_db of the typical loud level, short
    dips (pauses, a quiet bar) allowed up to dip_s."""
    block = int(SR * block_s)
    lv = block_db(mono, block)
    loud = np.sort(lv)[len(lv) // 2:]            # the louder half of the track
    level = float(np.median(loud)) - drop_db
    ok = lv >= level
    best, cur_start, gap = (0, 0), None, 0
    max_gap = int(dip_s / block_s)
    for i, good in enumerate(np.append(ok, False)):
        if good:
            if cur_start is None:
                cur_start = i
            gap = 0
        elif cur_start is not None:
            gap += 1
            if gap > max_gap or i == len(ok):
                end = i - gap + 1
                if end - cur_start > best[1] - best[0]:
                    best = (cur_start, end)
                cur_start, gap = None, 0
    return best[0] * block, best[1] * block, level


def band_features(mono: np.ndarray, hop: int, n_fft: int = 4096, bands: int = 32) -> np.ndarray:
    """Per-hop log energy in log-spaced bands, each frame unit-normalised (a timbre and harmony fingerprint)."""
    frames = np.lib.stride_tricks.sliding_window_view(mono, n_fft)[::hop] * np.hanning(n_fft)
    mag = np.abs(np.fft.rfft(frames, axis=1))
    freqs = np.fft.rfftfreq(n_fft, 1 / SR)
    edges = np.geomspace(60, 12000, bands + 1)
    feat = np.stack([np.log1p((mag[:, (freqs >= a) & (freqs < b)] ** 2).sum(1)) for a, b in zip(edges, edges[1:])], 1)
    feat -= feat.mean(1, keepdims=True)
    return feat / np.maximum(np.linalg.norm(feat, axis=1, keepdims=True), 1e-9)


def find_loop(mono: np.ndarray, lo: int, hi: int, min_len_s: float, window_s: float = 2.0, hop_s: float = 0.05,
              bar_s: float = 0.0):
    """Best (start, end) samples inside [lo, hi): what precedes the end matches what precedes the start. With bar_s,
    only loops a whole number of bars long count, so the beat stays in time across the wrap (beat-driven tracks
    without an exact repeat)."""
    hop = int(SR * hop_s)
    feat = band_features(mono, hop)
    w = int(window_s / hop_s)
    # window-averaged fingerprint ending at frame i (the audio just before i)
    c = np.cumsum(np.vstack([np.zeros(feat.shape[1]), feat]), 0)
    win = np.zeros_like(feat)
    win[w:] = (c[w + 1:] - c[1:-w]) / w
    win /= np.maximum(np.linalg.norm(win, axis=1, keepdims=True), 1e-9)
    f_lo = max(lo // hop, w + 1)
    f_hi = min(hi // hop, len(feat) - 1)
    min_len = int(min_len_s / hop_s)
    starts = np.arange(f_lo, f_hi - min_len)
    if not len(starts):
        raise SystemExit(f"steady part {(hi - lo) / SR:.1f} s is shorter than the minimum loop {min_len_s:.0f} s")
    sim = win[starts] @ win[f_lo:f_hi].T                       # starts x ends
    ends = np.arange(f_lo, f_hi)
    span = ends[None, :] - starts[:, None]
    valid = span >= min_len
    if bar_s:
        bars = span * hop_s / bar_s
        valid &= np.abs(bars - np.round(bars)) * bar_s <= hop_s / 2
    dist = np.where(valid, 1 - sim, np.inf)
    best = float(dist.min())
    # generated songs repeat sections almost exactly (1B: a section every 87.3 s at distance 0.0004), so only seams
    # nearly as good as the best compete; a longer loop with a rougher seam loses
    good = dist <= best + 0.0005
    length = np.where(good, ends[None, :] - starts[:, None], -1)
    si, ei = np.unravel_index(np.argmax(length), length.shape)
    return int(starts[si] * hop), int(ends[ei] * hop), float(dist[si, ei]), best


def align_end(mono: np.ndarray, start: int, end: int, search_s: float = 0.05, ref_s: float = 0.2):
    """Move the end by up to search_s so the waveform just before it lines up with the one just before the start.
    Returns (end, correlation of the two waveforms; 1 = identical)."""
    ref = mono[start - int(SR * ref_s): start]
    s = int(SR * search_s)
    best, best_off = -np.inf, 0
    for off in range(-s, s + 1, 8):
        seg = mono[end + off - len(ref): end + off]
        if len(seg) != len(ref):
            continue
        v = float(np.dot(seg, ref) / (np.linalg.norm(seg) * np.linalg.norm(ref) + 1e-9))
        if v > best:
            best, best_off = v, off
    for off in range(best_off - 8, best_off + 9):              # refine to the sample
        seg = mono[end + off - len(ref): end + off]
        if len(seg) == len(ref):
            v = float(np.dot(seg, ref) / (np.linalg.norm(seg) * np.linalg.norm(ref) + 1e-9))
            if v > best:
                best, best_off = v, off
    return end + best_off, best


def make_loop(x: np.ndarray, start: int, end: int, xf_s: float = 0.5) -> np.ndarray:
    """x[start:end] with its last xf_s blended (equal power) into the audio leading into start."""
    xf = int(SR * xf_s)
    loop = x[start:end].copy()
    t = np.linspace(0, np.pi / 2, xf)[:, None]
    loop[-xf:] = loop[-xf:] * np.cos(t) + x[start - xf:start] * np.sin(t)
    return loop


def whole_loop(x: np.ndarray, overlap_s: float, floor_db: float = -50.0):
    """The whole song as a loop: trailing near-silence trimmed, then the song's last overlap_s laid over its first
    overlap_s, so playback that restarts at the file's start runs from the fade-out straight into the intro. The head
    keeps its own level (the intro plays as written on the first play too); the tail gets a cosine fade so it reaches
    silence inside the overlap even when the song ends loud. Returns (loop, trimmed seconds, the tail's level in the
    overlap relative to the head's)."""
    mono = x.mean(1)
    lv = block_db(mono, int(SR * 0.05))
    loud = np.nonzero(lv > floor_db)[0]
    n = (int(loud[-1]) + 1) * int(SR * 0.05) if len(loud) else len(x)
    trimmed = (len(x) - n) / SR
    L = int(SR * overlap_s)
    if n < 3 * L:
        raise SystemExit(f"track {n / SR:.1f} s is too short for a {overlap_s:.0f} s overlap")
    head, tail = x[:L], x[n - L:n] * np.cos(np.linspace(0, np.pi / 2, L))[:, None]
    loop = x[:n - L].copy()
    loop[:L] = head + tail
    return loop, trimmed, db(rms(tail)) - db(rms(head))


def gap_loop(x: np.ndarray, gap_s: float, floor_db: float = -50.0):
    """The whole song with silence for the loop: leading and trailing near-silence trimmed, then exactly gap_s of
    silence after the fade-out, so a restart is fade-out, a pause, then the intro building in (no overlap, no echo).
    Returns (loop, leading trimmed s, trailing trimmed s)."""
    block = int(SR * 0.05)
    lv = block_db(x.mean(1), block)
    loud = np.nonzero(lv > floor_db)[0]
    if not len(loud):
        raise SystemExit("track is silent")
    a, b = int(loud[0]) * block, (int(loud[-1]) + 1) * block
    body = x[a:b].copy()
    edge = int(SR * 0.01)                                      # 10 ms ramps so the trims never click
    ramp = np.linspace(0, 1, edge)[:, None]
    body[:edge] *= ramp
    body[-edge:] *= ramp[::-1]
    return np.vstack([body, np.zeros((int(SR * gap_s), 2))]), a / SR, (len(x) - b) / SR


def reference_rms_db() -> float:
    levels = []
    for d in REFERENCE_TRACKS:
        for f in sorted(d.glob("*.ogg")):
            try:
                levels.append(db(rms(decode(f))))
            except subprocess.CalledProcessError:
                continue
    return float(np.median(levels)) if levels else -18.0


def loudness(x: np.ndarray, target_db: float, ceiling_db: float = -1.0, max_limit_db: float = 3.0) -> np.ndarray:
    """Gain toward target_db RMS, but never push peaks more than max_limit_db into the limiter: a quiet, dynamic
    track (a lullaby) stays quieter rather than being squashed. Peaks are then soft-limited to ceiling_db."""
    gain_db = min(target_db - db(rms(x)), ceiling_db + max_limit_db - db(np.abs(x).max()))
    y = x * 10 ** (gain_db / 20)
    c = 10 ** (ceiling_db / 20)
    a = np.abs(y)
    over = a > c * 0.9                                          # soft knee from 0.9 of the ceiling
    k = c * 0.9
    y[over] = np.sign(y[over]) * (k + (c - k) * np.tanh((a[over] - k) / (c - k)))
    return y


def main() -> None:
    ap = argparse.ArgumentParser(description="Import a generated music track as a game track.")
    ap.add_argument("src", type=Path, nargs="+",
                    help="The track, or several stems of one track (Flow Music's Split stems) to mix together.")
    ap.add_argument("--out", type=Path, required=True, help="Ogg file to write (e.g. assets/audio/music/<key>.ogg).")
    ap.add_argument("--no-loop", action="store_true", help="Keep the whole track (credits, stingers).")
    ap.add_argument("--whole", action="store_true",
                    help="Loop the whole song: its fade-out laid over its intro (menus, songs with a shape).")
    ap.add_argument("--overlap", type=float, default=8.0, help="With --whole: seconds of overlap (default 8).")
    ap.add_argument("--bpm", type=float, default=None,
                    help="Section loop only: keep the loop a whole number of 4-beat bars at this tempo.")
    ap.add_argument("--xfade", type=float, default=0.5, help="Section loop crossfade in seconds (default 0.5).")
    ap.add_argument("--gap", type=float, default=None,
                    help="Loop the whole song with this many seconds of silence after its fade-out (no overlap).")
    ap.add_argument("--min-loop", type=float, default=60.0, help="Shortest loop to accept, in seconds (default 60).")
    ap.add_argument("--target-db", type=float, default=None, help="RMS level; default: the game tracks' median.")
    ap.add_argument("--preview", type=Path, default=None, help="Also write an MP3 of the seam to check by ear.")
    a = ap.parse_args()
    parts = [decode(p) for p in a.src]
    n = min(len(p) for p in parts)
    x = sum(p[:n] for p in parts)                              # stems of one track share its timeline
    mono = x.mean(1)
    target = a.target_db if a.target_db is not None else reference_rms_db()
    name = a.src[0].name if len(a.src) == 1 else " + ".join(p.stem for p in a.src)
    print(f"{name}: {len(x) / SR:.1f} s, RMS {db(rms(x)):.1f} dBFS, peak {db(np.abs(x).max()):.1f} dBFS; "
          f"target RMS {target:.1f} dBFS")
    if a.no_loop:
        out = loudness(x, target)
    elif a.gap is not None:
        body, lead, trail = gap_loop(x, a.gap)
        print(f"  whole-song loop with a pause: {lead:.1f} s leading and {trail:.1f} s trailing silence trimmed, "
              f"{a.gap:.1f} s of silence before the restart")
        out = np.vstack([loudness(body[: len(body) - int(SR * a.gap)], target), body[len(body) - int(SR * a.gap):]])
        if a.preview:
            k = int(SR * 12)
            encode(np.vstack([out[-k:], out[:k]]), a.preview, "mp3")
            print(f"  wrap preview (12 s before the restart, 12 s after) -> {a.preview}")
    elif a.whole:
        loop, trimmed, rel = whole_loop(x, a.overlap)
        print(f"  whole-song loop: {trimmed:.1f} s of trailing silence trimmed, last {a.overlap:.0f} s laid over the "
              f"intro (tail {rel:+.1f} dB against the intro there)")
        out = loudness(loop, target)
        if a.preview:
            k = int(SR * 12)
            encode(np.vstack([out[-k:], out[:k]]), a.preview, "mp3")
            print(f"  wrap preview (12 s before the restart, 12 s after) -> {a.preview}")
    else:
        lo, hi, level = steady_part(mono)
        print(f"  steady part {lo / SR:.1f}..{hi / SR:.1f} s (blocks above {level:.1f} dB)")
        bar_s = 4 * 60 / a.bpm if a.bpm else 0.0
        start, end, dist, best = find_loop(mono, lo, hi, a.min_loop, bar_s=bar_s)
        if bar_s:
            print(f"  bar-locked at {a.bpm:g} BPM: {(end - start) / SR / bar_s:.0f} bars of {bar_s:.2f} s")
        end, corr = align_end(mono, start, end)
        print(f"  loop {start / SR:.2f}..{end / SR:.2f} s ({(end - start) / SR:.1f} s), seam distance {dist:.4f} "
              f"(best possible {best:.4f}; 0 = identical), waveform match at the seam {corr:.2f} (1 = identical)")
        out = loudness(make_loop(x, start, end, a.xfade), target)
        if a.preview:
            k = int(SR * 8)
            encode(np.vstack([out[-k:], out[:k]]), a.preview, "mp3")
            print(f"  seam preview (8 s before the wrap, 8 s after) -> {a.preview}")
    encode(out, a.out, "ogg")
    print(f"  wrote {a.out} ({len(out) / SR:.1f} s, RMS {db(rms(out)):.1f} dBFS, peak {db(np.abs(out).max()):.1f} dBFS)")


if __name__ == "__main__":
    main()

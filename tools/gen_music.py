#!/usr/bin/env python3
# Writes the generated music loops (MUSIC1-A): original tracks synthesized from notes, no samples.
#   python tools/gen_music.py [track ...] [--layer kit|game ...] [--root DIR] [--list]   (default: every track)
# Each track lands at its ASSETORG2 layer's audio/music/<key>.ogg: the mood loops (combat, boss, town, shop, danger,
# travel) in the RPG kit (kits/rpg/assets/), the world tracks in the game (games/emberwake/assets/). DIR is a
# workspace root (default: this repository); --list prints the paths without rendering (no ffmpeg needed).
# No menu_theme key (AI-02, 2026-11-12): the shipped menu_theme.ogg is a Flow Music track (tools/mock_ui/music.py,
# art_src/music/menu_theme.json) that a run with no arguments must never overwrite.
# Needs numpy and ffmpeg (libvorbis). Each track renders as one circular buffer, so notes, reverb and
# noise beds that run past the end wrap onto the start and the loop has no seam. Prints "§ wrote <path> <s>s".
# AI-30: the ambience beds (AMBIENCE) land in the game's audio/ambience/<key>.ogg, levelled low (RMS -26 dB) for the
# ambience bus (PR7-34); their keys follow the ones PR7-34 lists.
import argparse
import os
import shutil
import subprocess
import sys
import tempfile
import wave

import numpy as np

SR = 44100
NOTE_NAMES = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}


def midi(name: str) -> int:
    # "A3", "C#4", "Bb2" -> MIDI number.
    pc = NOTE_NAMES[name[0]]
    rest = name[1:]
    while rest and rest[0] in "#b":
        pc += 1 if rest[0] == "#" else -1
        rest = rest[1:]
    return pc + 12 * (int(rest) + 1)


def hz(m: float) -> float:
    return 440.0 * 2.0 ** ((m - 69) / 12.0)


class Track:
    # A circular stereo buffer: anything written past the end wraps onto the start.
    def __init__(self, bpm: float, beats_per_bar: int, bars: int, seed: int) -> None:
        self.beat = 60.0 / bpm
        self.bpb = beats_per_bar
        self.bars = bars
        self.n = int(round(bars * beats_per_bar * self.beat * SR))
        self.dry = np.zeros((2, self.n))
        self.wet = np.zeros((2, self.n))
        self.rng = np.random.default_rng(seed)

    def at(self, bar: float, beat: float = 0.0) -> float:
        return (bar * self.bpb + beat) * self.beat

    def add(self, sig: np.ndarray, t: float, pan: float = 0.0, gain: float = 1.0, send: float = 0.3) -> None:
        start = int(round(t * SR)) % self.n
        left = np.cos((pan + 1.0) * np.pi / 4.0) * gain
        right = np.sin((pan + 1.0) * np.pi / 4.0) * gain
        idx = (np.arange(len(sig)) + start) % self.n
        for ch, g in ((0, left), (1, right)):
            np.add.at(self.dry[ch], idx, sig * g * (1.0 - send * 0.5))
            np.add.at(self.wet[ch], idx, sig * g * send)

    def add_bed(self, sig2: np.ndarray, gain: float = 1.0, send: float = 0.2) -> None:
        # A full-length stereo bed (drone, noise wash); already loop-length.
        self.dry += sig2 * gain * (1.0 - send * 0.5)
        self.wet += sig2 * gain * send

    def lfo(self, cycles: int, phase: float = 0.0) -> np.ndarray:
        # 0..1 sine with a whole number of cycles over the loop, so it is seamless.
        t = np.arange(self.n) / self.n
        return 0.5 + 0.5 * np.sin(2 * np.pi * cycles * t + phase)

    def render(self, reverb_s: float, peak_db: float = -1.5) -> np.ndarray:
        out = self.dry + circular_reverb(self.wet, reverb_s, self.rng)
        out = np.tanh(out * 1.2) / 1.2
        peak = np.max(np.abs(out))
        return out * (10 ** (peak_db / 20.0) / max(peak, 1e-9))


def circular_reverb(x: np.ndarray, seconds: float, rng: np.random.Generator) -> np.ndarray:
    n = x.shape[1]
    m = int(seconds * SR)
    t = np.arange(m) / SR
    out = np.zeros_like(x)
    for ch in range(2):
        ir = rng.standard_normal(m) * np.exp(-6.9 * t / seconds)
        ir[: int(0.012 * SR)] *= np.linspace(0, 1, int(0.012 * SR))
        ir = lowpass(ir, 5000.0)
        ir /= np.sqrt(np.sum(ir ** 2))
        pad = np.zeros(n)
        pad[: min(m, n)] = ir[: min(m, n)]
        out[ch] = np.fft.irfft(np.fft.rfft(x[ch]) * np.fft.rfft(pad), n)
    return out * 0.9


def lowpass(x: np.ndarray, cutoff: float, high: float = 0.0) -> np.ndarray:
    # Circular FFT band filter with soft edges (cutoff = top, high = bottom; 0 = none).
    f = np.fft.rfftfreq(len(x), 1.0 / SR)
    spec = np.fft.rfft(x)
    gain = 1.0 / (1.0 + (f / cutoff) ** 4)
    if high > 0:
        gain *= 1.0 - 1.0 / (1.0 + (f / high) ** 4)
    return np.fft.irfft(spec * gain, len(x))


def env(n: int, attack: float, release: float) -> np.ndarray:
    e = np.ones(n)
    a = min(n, max(1, int(attack * SR)))
    r = min(n - a, max(1, int(release * SR)))
    e[:a] = np.linspace(0, 1, a)
    if r > 0:
        e[n - r:] = np.linspace(1, 0, r) ** 2
    return e


# --- instruments (mono, return a buffer that may outlast the note) ---------------------------------

def pluck(m: float, dur: float, bright: float = 1.0, decay: float = 1.6) -> np.ndarray:
    # Harp-like: harmonics that die faster the higher they are.
    length = dur + 2.5
    t = np.arange(int(length * SR)) / SR
    f = hz(m)
    sig = np.zeros_like(t)
    for h in range(1, 11):
        if f * h > SR / 2.2:
            break
        sig += (1.0 / h ** (1.6 - 0.4 * bright)) * np.sin(2 * np.pi * f * h * t + h) * np.exp(-t * decay * (1 + 0.55 * h))
    sig *= np.minimum(1.0, t / 0.004)
    return sig * 0.3


def pad(notes: list, dur: float, attack: float = 1.2, release: float = 1.5, harmonics: int = 7, detune: float = 0.12) -> np.ndarray:
    length = dur + release
    n = int(length * SR)
    t = np.arange(n) / SR
    sig = np.zeros(n)
    for m in notes:
        for d in (-detune, 0.0, detune):
            f = hz(m + d)
            for h in range(1, harmonics + 1):
                if f * h > SR / 2.5:
                    break
                sig += np.sin(2 * np.pi * f * h * t + 1.7 * h + d) / (h ** 1.4)
    e = env(n, attack, release)
    return sig * e * (0.05 / max(1, len(notes)) ** 0.5)


def horn(m: float, dur: float) -> np.ndarray:
    n = int((dur + 0.8) * SR)
    t = np.arange(n) / SR
    f = hz(m)
    vib = 1 + 0.004 * np.sin(2 * np.pi * 5.0 * t) * np.minimum(1, t / 0.6)
    phase = 2 * np.pi * f * np.cumsum(vib) / SR
    sig = sum(np.sin(h * phase) * (0.9 ** h) / h ** 0.6 for h in range(1, 9))
    return sig * env(n, 0.25, 0.8) * 0.07


def whistle(m: float, dur: float, rng: np.random.Generator) -> np.ndarray:
    n = int((dur + 0.3) * SR)
    t = np.arange(n) / SR
    f = hz(m)
    vib = 1 + 0.006 * np.sin(2 * np.pi * 5.5 * t) * np.minimum(1, t / 0.35)
    phase = 2 * np.pi * f * np.cumsum(vib) / SR
    sig = np.sin(phase) + 0.18 * np.sin(2 * phase) + 0.06 * np.sin(3 * phase)
    breath = lowpass(rng.standard_normal(n), f * 3, f * 0.8) * 0.12
    return (sig + breath) * env(n, 0.05, 0.3) * 0.12


def bass(m: float, dur: float, grit: float = 0.3) -> np.ndarray:
    n = int((dur + 0.15) * SR)
    t = np.arange(n) / SR
    f = hz(m)
    sig = np.sin(2 * np.pi * f * t) + grit * np.sin(4 * np.pi * f * t) + grit * 0.4 * np.sin(6 * np.pi * f * t)
    return sig * env(n, 0.006, 0.12) * np.exp(-t * 1.2) * 0.32


def drum(pitch: float, decay: float, noise: float, rng: np.random.Generator, drop: float = 2.2) -> np.ndarray:
    # Frame drum / kick / tom: pitched sine with a falling pitch plus a noise slap.
    n = int((decay * 4 + 0.05) * SR)
    t = np.arange(n) / SR
    f = pitch * (1 + drop * np.exp(-t * 40))
    body = np.sin(2 * np.pi * np.cumsum(f) / SR) * np.exp(-t / decay)
    slap = lowpass(rng.standard_normal(n), 3500, 200) * np.exp(-t * 60) * noise
    return (body + slap) * 0.5


def snare(rng: np.random.Generator) -> np.ndarray:
    n = int(0.35 * SR)
    t = np.arange(n) / SR
    tone = np.sin(2 * np.pi * 190 * t) * np.exp(-t * 30)
    rattle = lowpass(rng.standard_normal(n), 7000, 900) * np.exp(-t * 14)
    return (0.5 * tone + rattle) * 0.32


def hat(rng: np.random.Generator, open_: bool = False) -> np.ndarray:
    n = int((0.3 if open_ else 0.08) * SR)
    t = np.arange(n) / SR
    sig = lowpass(rng.standard_normal(n), 14000, 7000) * np.exp(-t * (9 if open_ else 55))
    return sig * 0.16


def anvil(m: float, rng: np.random.Generator) -> np.ndarray:
    # Struck metal: inharmonic partials with long ring.
    n = int(2.2 * SR)
    t = np.arange(n) / SR
    f = hz(m)
    sig = np.zeros(n)
    for ratio, amp, dec in ((1.0, 1.0, 1.4), (2.76, 0.6, 2.2), (5.4, 0.35, 3.5), (8.93, 0.2, 5.0)):
        sig += amp * np.sin(2 * np.pi * f * ratio * t) * np.exp(-t * dec)
    click = lowpass(rng.standard_normal(n), 9000, 2000) * np.exp(-t * 120)
    return (sig + click * 0.6) * 0.14


def drone(tr: Track, notes: list, gain: float, cycles: int = 3) -> None:
    t = np.arange(tr.n) / SR
    bed = np.zeros((2, tr.n))
    for i, m in enumerate(notes):
        # Snap each partial to a whole number of cycles over the loop so the bed is seamless.
        f = round(hz(m) * tr.n / SR) * SR / tr.n
        tone = np.sin(2 * np.pi * f * t) + 0.25 * np.sin(4 * np.pi * f * t)
        bed[i % 2] += tone * (0.7 + 0.3 * tr.lfo(cycles + i, i))
        bed[(i + 1) % 2] += tone * 0.6 * (0.7 + 0.3 * tr.lfo(cycles + i + 1, i + 2))
    tr.add_bed(bed / len(notes), gain, 0.3)


def wash(tr: Track, low: float, high: float, cycles: int, gain: float, depth: float = 0.8) -> None:
    # Wind or sea: band-limited noise that swells a whole number of times per loop.
    bed = np.zeros((2, tr.n))
    for ch in range(2):
        bed[ch] = lowpass(tr.rng.standard_normal(tr.n), high, low)
        bed[ch] /= np.max(np.abs(bed[ch]))
        bed[ch] *= (1 - depth) + depth * tr.lfo(cycles, ch * 0.9) ** 2
    tr.add_bed(bed, gain, 0.4)


def melody(tr: Track, notes: list, start_bar: int, inst, pan: float = 0.1, gain: float = 1.0, send: float = 0.35) -> None:
    # notes: (beat offset from start_bar, midi, length in beats); midi None = rest.
    for beat, m, length in notes:
        if m is None:
            continue
        tr.add(inst(m, length * tr.beat), tr.at(start_bar, beat), pan, gain, send)


def chord_tones(root: int, quality: str) -> list:
    steps = {"m": (0, 3, 7), "M": (0, 4, 7), "7": (0, 4, 7, 10), "m7": (0, 3, 7, 10), "sus": (0, 5, 7)}[quality]
    return [root + s for s in steps]


# --- tracks ------------------------------------------------------------------------------------------

def ashlands() -> np.ndarray:
    # 64 BPM, 4/4, 20 bars (75 s). D Phrygian dominant flavour, wind bed, sparse hand drum.
    tr = Track(64, 4, 20, seed=23)
    prog = ["D2", "Eb2", "D2", "C2", "Bb1", "A1", "D2", "Eb2", "G1", "A1"]
    quals = ["M", "M", "M", "m", "M", "M", "M", "M", "m", "M"]
    scale = [midi(n) for n in ("D4", "Eb4", "F#4", "G4", "A4", "Bb4", "C5", "D5", "Eb5", "F#5")]
    for i, (root, q) in enumerate(zip(prog, quals)):
        r = midi(root)
        tones = chord_tones(r + 24, q)
        tr.add(pad(tones, 8 * tr.beat, 2.0, 2.0, 5, 0.08), tr.at(2 * i), 0.0, 0.8, 0.5)
        tr.add(bass(r + 12, 6 * tr.beat, 0.05), tr.at(2 * i), 0.0, 0.4, 0.2)
        # Melody: a slow walk on the scale, strong beats land on chord tones.
        pos = 4
        for bar in (0, 1):
            pattern = [(0, 1.5), (1.5, 0.5), (2, 2)] if (i + bar) % 2 == 0 else [(0, 1), (1, 1), (2, 1), (3, 1)]
            if (i + bar) % 4 == 3:
                pattern = [(0, 3)]
            for beat, length in pattern:
                pos = int(np.clip(pos + tr.rng.integers(-2, 3), 0, len(scale) - 1))
                m = scale[pos]
                if beat == 0:
                    pcs = [t % 12 for t in tones]
                    while m % 12 not in pcs:
                        m -= 1
                tr.add(pluck(m, length * tr.beat, 0.8, 1.1), tr.at(2 * i + bar, beat), 0.25, 0.8, 0.45)
        for bar in (0, 1):
            tr.add(drum(110, 0.12, 0.8, tr.rng), tr.at(2 * i + bar, 0), -0.2, 0.3, 0.25)
            tr.add(drum(160, 0.07, 0.9, tr.rng), tr.at(2 * i + bar, 1.5), 0.25, 0.15, 0.25)
            tr.add(drum(140, 0.08, 0.9, tr.rng), tr.at(2 * i + bar, 2.5), 0.2, 0.18, 0.25)
    drone(tr, [midi("D1"), midi("A1"), midi("D2")], 0.16, 2)
    wash(tr, 300, 2200, 5, 0.06)
    return tr.render(3.6)


def iron_reaches() -> np.ndarray:
    # 80 BPM, 4/4, 24 bars (72 s). E minor, anvil strikes on 2 and 4, low toms on 1 and 3, horn pads.
    tr = Track(80, 4, 24, seed=37)
    prog = [("E2", "m"), ("C2", "M"), ("A1", "m"), ("B1", "M"), ("E2", "m"), ("G1", "M"),
            ("D2", "M"), ("B1", "M"), ("E2", "m"), ("C2", "M"), ("A1", "m"), ("B1", "7")]
    for i, (root, q) in enumerate(prog):
        r = midi(root)
        tones = chord_tones(r + 24, q)
        for j, m in enumerate(tones[:3]):
            tr.add(horn(m, 8 * tr.beat - 0.3), tr.at(2 * i), (j - 1) * 0.3, 0.9, 0.45)
        for b in range(16):  # driving eighth-note bass ostinato
            m = r + (12 if b % 8 == 6 else 0) + (7 if b % 8 == 3 else 0)
            tr.add(bass(m, 0.45 * tr.beat, 0.5), tr.at(2 * i, b * 0.5), -0.05, 0.5 if b % 2 == 0 else 0.35, 0.15)
        for bar in (0, 1):
            tr.add(drum(55, 0.35, 0.3, tr.rng, 1.5), tr.at(2 * i + bar, 0), 0.0, 0.8, 0.25)
            tr.add(drum(62, 0.3, 0.3, tr.rng, 1.5), tr.at(2 * i + bar, 2), 0.1, 0.6, 0.25)
            tr.add(anvil(midi("E5"), tr.rng), tr.at(2 * i + bar, 1), 0.45, 0.7, 0.35)
            tr.add(anvil(midi("B4"), tr.rng), tr.at(2 * i + bar, 3), -0.45, 0.55, 0.35)
            if i >= 6:
                tr.add(anvil(midi("E5"), tr.rng), tr.at(2 * i + bar, 3.5), 0.45, 0.3, 0.35)
    # Second half: a low horn tune on top.
    tune = [(0, 64, 2), (2, 67, 1), (3, 66, 1), (4, 64, 3), (7, 62, 1), (8, 60, 2), (10, 64, 2), (12, 59, 4)]
    for start in (12, 16, 20):
        melody(tr, [(b, m + (0 if start < 20 else 2) * 0, d) for b, m, d in tune], start, horn, 0.15, 1.6, 0.4)
    drone(tr, [midi("E1"), midi("B1")], 0.14, 4)
    return tr.render(2.6)


def tidewrack() -> np.ndarray:
    # 6/8 at an eighth of 0.25 s (bar 1.5 s), 48 bars (72 s). G mixolydian folk tune on a whistle, sea bed.
    tr = Track(240, 6, 48, seed=53)
    prog = [("G2", "M"), ("F2", "M"), ("C2", "M"), ("G2", "M"), ("E2", "m"), ("C2", "M"),
            ("D2", "M"), ("G2", "M"), ("G2", "M"), ("F2", "M"), ("D2", "m"), ("D2", "M")]
    for i, (root, q) in enumerate(prog):
        r = midi(root)
        tones = chord_tones(r + 24, q)
        tr.add(pad(tones, 4 * 6 * tr.beat, 1.6, 1.6, 6, 0.1), tr.at(4 * i), 0.0, 0.7, 0.5)
        for bar in range(4):
            b0 = 4 * i + bar
            tr.add(bass(r + 12, 2.6 * tr.beat, 0.1), tr.at(b0, 0), -0.1, 0.45, 0.2)
            tr.add(bass(r + 19, 2.6 * tr.beat, 0.1), tr.at(b0, 3), -0.1, 0.35, 0.2)
            for k, e in enumerate((0, 2, 3, 5)):  # lilting harp accompaniment
                tr.add(pluck(tones[k % 3] + 12, tr.beat, 0.6, 2.0), tr.at(b0, e), -0.35, 0.28, 0.4)
            tr.add(drum(90, 0.16, 0.5, tr.rng), tr.at(b0, 0), 0.1, 0.35, 0.25)
            tr.add(drum(120, 0.09, 0.7, tr.rng), tr.at(b0, 3), -0.1, 0.2, 0.25)
    phrase_a = [(0, 74, 3), (3, 72, 1), (4, 71, 1), (5, 69, 1), (6, 67, 3), (9, 69, 2), (11, 71, 1),
                (12, 72, 2), (14, 71, 1), (15, 69, 3), (18, 65, 2), (20, 67, 1), (21, 69, 3)]
    phrase_b = [(0, 71, 2), (2, 72, 1), (3, 74, 3), (6, 76, 2), (8, 74, 1), (9, 72, 3),
                (12, 74, 2), (14, 72, 1), (15, 71, 2), (17, 69, 1), (18, 67, 6)]
    wl = lambda m, d: whistle(m, d, tr.rng)
    for start, phrase in ((8, phrase_a), (12, phrase_b), (16, phrase_a), (20, phrase_b),
                          (32, phrase_a), (36, phrase_b), (40, phrase_a), (44, phrase_b)):
        melody(tr, phrase, start, wl, 0.2, 1.0, 0.4)
    wash(tr, 150, 1400, 12, 0.11, 0.9)
    return tr.render(3.0)


def combat() -> np.ndarray:
    # 120 BPM, 4/4, 24 bars (48 s). A minor; section B (bars 8-15) adds toms and a higher riff.
    tr = Track(120, 4, 24, seed=71)
    prog = [("A1", "m"), ("F1", "M"), ("G1", "M"), ("E1", "M")] * 3
    riff = [(0, 0, 0.5), (0.5, 0, 0.5), (1, 3, 0.5), (1.5, 0, 0.5), (2, 7, 0.75), (3, 5, 0.5), (3.5, 3, 0.5)]
    for i, (root, q) in enumerate(prog):
        r = midi(root)
        tones = chord_tones(r + 24, q)
        section_b = 4 <= i < 8
        tr.add(pad(tones, 8 * tr.beat, 0.4, 0.6, 8, 0.15), tr.at(2 * i), 0.0, 0.55, 0.35)
        for b in range(16):
            tr.add(bass(r + (12 if b % 4 == 2 else 0), 0.4 * tr.beat, 0.6), tr.at(2 * i, b * 0.5), 0.0, 0.55, 0.1)
        for bar in (0, 1):
            b0 = 2 * i + bar
            for beat in (0, 1.5, 2) if bar == 0 else (0, 2, 2.75):
                tr.add(drum(52, 0.22, 0.5, tr.rng, 3.0), tr.at(b0, beat), 0.0, 0.9, 0.1)
            for beat in (1, 3):
                tr.add(snare(tr.rng), tr.at(b0, beat), 0.05, 0.8, 0.25)
            for e in range(8):
                tr.add(hat(tr.rng, e == 7), tr.at(b0, e * 0.5), 0.3, 0.5 if e % 2 else 0.8, 0.1)
            if section_b:
                tr.add(drum(110, 0.2, 0.4, tr.rng, 1.2), tr.at(b0, 3.5), -0.4, 0.45, 0.3)
                tr.add(drum(80, 0.25, 0.4, tr.rng, 1.2), tr.at(b0, 3.75), 0.4, 0.45, 0.3)
            shift = 12 if section_b else 0
            base = midi("A4") + shift
            # Riff follows the chord root, relative to A.
            off = (r - midi("A1")) % 12
            off = off - 12 if off > 6 else off
            for beat, step, length in riff:
                m = base + off + step + (0 if step not in (3,) or q == "m" else 1)
                tr.add(pluck(m, length * tr.beat, 1.0, 4.0), tr.at(b0, beat), 0.25, 0.75, 0.25)
    drone(tr, [midi("A1")], 0.08, 6)
    return tr.render(1.8)


def walk_melody(tr: Track, prog: list, bars_per_chord: int, scale: list, inst, patterns: list, pan: float = 0.2,
                gain: float = 1.0, send: float = 0.35, start_pos: int = 4, rest_every: int = 0) -> None:
    # A seeded walk on `scale`; the first note of each bar lands on a chord tone. prog: [(root midi, quality)].
    pos = start_pos
    for i, (r, q) in enumerate(prog):
        pcs = [t % 12 for t in chord_tones(r, q)]
        for b in range(bars_per_chord):
            bar = i * bars_per_chord + b
            if rest_every and bar % rest_every == rest_every - 1:
                pattern = [(0, tr.bpb - 1)]
            else:
                pattern = patterns[int(tr.rng.integers(0, len(patterns)))]
            for beat, length in pattern:
                pos = int(np.clip(pos + tr.rng.integers(-2, 3), 0, len(scale) - 1))
                m = scale[pos]
                if beat == 0:
                    while m % 12 not in pcs:
                        m -= 1
                tr.add(inst(m, length * tr.beat), tr.at(bar, beat), pan, gain, send)


def town() -> np.ndarray:
    # 88 BPM, 4/4, 24 bars (65 s). Warm C major: lute arpeggios, whistle tune, light drum.
    tr = Track(88, 4, 24, seed=101)
    prog = [(midi(n), q) for n, q in (("C2", "M"), ("G1", "M"), ("A1", "m"), ("F1", "M"), ("C2", "M"), ("F1", "M"),
                                      ("G1", "M"), ("C2", "M"), ("A1", "m"), ("E1", "m"), ("F1", "M"), ("G1", "M"))]
    for i, (r, q) in enumerate(prog):
        tones = chord_tones(r + 24, q)
        tr.add(pad(tones, 8 * tr.beat, 0.8, 1.2, 6, 0.08), tr.at(2 * i), 0.0, 0.5, 0.45)
        for bar in (0, 1):
            b0 = 2 * i + bar
            tr.add(bass(r + 12, 1.8 * tr.beat, 0.1), tr.at(b0, 0), -0.1, 0.45, 0.2)
            tr.add(bass(r + 19, 1.8 * tr.beat, 0.1), tr.at(b0, 2), -0.1, 0.35, 0.2)
            for e in range(8):
                m = tones[[0, 1, 2, 1, 0, 1, 2, 3 % len(tones)][e] % len(tones)] + 12
                tr.add(pluck(m, 0.5 * tr.beat, 0.7, 2.4), tr.at(b0, e * 0.5), -0.35, 0.22, 0.35)
            tr.add(drum(100, 0.12, 0.6, tr.rng), tr.at(b0, 0), 0.0, 0.25, 0.2)
            tr.add(drum(140, 0.07, 0.8, tr.rng), tr.at(b0, 2), 0.1, 0.15, 0.2)
    scale = [midi(n) for n in ("G4", "A4", "C5", "D5", "E5", "G5", "A5")]
    wl = lambda m, d: whistle(m, d, tr.rng)
    walk_melody(tr, prog, 2, scale, wl, [[(0, 1.5), (1.5, 0.5), (2, 2)], [(0, 1), (1, 1), (2, 2)], [(0, 2), (2, 1), (3, 1)]],
                0.2, 0.9, 0.35, 3, 4)
    return tr.render(2.4)


def danger() -> np.ndarray:
    # 70 BPM, 4/4, 16 bars (55 s). B minor with a flat second: drone, heartbeat drum, sparse high plucks, wind.
    tr = Track(70, 4, 16, seed=131)
    roots = ["B1", "C2", "B1", "G1", "B1", "C2", "F#1", "F#1"]
    for i, root in enumerate(roots):
        r = midi(root)
        tr.add(pad([r + 24, r + 27, r + 31], 8 * tr.beat, 2.5, 2.0, 5, 0.18), tr.at(2 * i), 0.0, 0.6, 0.55)
        for bar in (0, 1):
            b0 = 2 * i + bar
            tr.add(drum(48, 0.3, 0.2, tr.rng, 1.0), tr.at(b0, 0), 0.0, 0.7, 0.2)
            tr.add(drum(48, 0.25, 0.2, tr.rng, 1.0), tr.at(b0, 0.4), 0.0, 0.45, 0.2)
            tr.add(bass(r, 3 * tr.beat, 0.4), tr.at(b0, 0), 0.0, 0.35, 0.2)
            if (i + bar) % 2 == 1:
                hi = [midi("B5"), midi("C6"), midi("F#5"), midi("D6")][int(tr.rng.integers(0, 4))]
                tr.add(pluck(hi, tr.beat, 1.0, 0.9), tr.at(b0, 2.5), 0.5 - bar, 0.35, 0.7)
        if i % 2 == 1:
            tr.add(anvil(midi("F5"), tr.rng), tr.at(2 * i + 1, 3), -0.4, 0.25, 0.8)
    drone(tr, [midi("B0"), midi("F#1"), midi("C2")], 0.2, 3)
    wash(tr, 200, 1600, 4, 0.08)
    return tr.render(4.0)


def shop() -> np.ndarray:
    # 100 BPM waltz, 3/4, 32 bars (58 s). Cosy F major: oom-pah bass, plucked chords, whistle tune.
    tr = Track(100, 3, 32, seed=151)
    prog = [(midi(n), q) for n, q in (("F1", "M"), ("C2", "M"), ("D2", "m"), ("Bb1", "M"), ("F1", "M"), ("Bb1", "M"),
                                      ("C2", "7"), ("F1", "M"))]
    for i, (r, q) in enumerate(prog):
        tones = chord_tones(r + 24, q)
        for bar in range(4):
            b0 = 4 * i + bar
            tr.add(bass(r + 12 + (7 if bar % 2 else 0), 0.9 * tr.beat, 0.2), tr.at(b0, 0), -0.1, 0.5, 0.15)
            for beat in (1, 2):
                for k, m in enumerate(tones[:3]):
                    tr.add(pluck(m + 12, 0.5 * tr.beat, 0.6, 4.0), tr.at(b0, beat), (k - 1) * 0.3, 0.12, 0.25)
            tr.add(drum(150, 0.06, 0.9, tr.rng), tr.at(b0, 0), 0.2, 0.15, 0.2)
    scale = [midi(n) for n in ("F4", "G4", "A4", "Bb4", "C5", "D5", "E5", "F5")]
    wl = lambda m, d: whistle(m, d, tr.rng)
    walk_melody(tr, prog, 4, scale, wl, [[(0, 2), (2, 1)], [(0, 1), (1, 1), (2, 1)], [(0, 3)]], 0.15, 0.85, 0.3, 4)
    return tr.render(1.8)


def travel() -> np.ndarray:
    # 92 BPM, 4/4, 24 bars (63 s). Open D mixolydian: marching snare, horn tune, broad pads.
    tr = Track(92, 4, 24, seed=171)
    prog = [(midi(n), q) for n, q in (("D2", "M"), ("C2", "M"), ("G1", "M"), ("D2", "M"), ("B1", "m"), ("G1", "M"),
                                      ("A1", "M"), ("D2", "M"), ("D2", "M"), ("C2", "M"), ("G1", "M"), ("A1", "sus"))]
    for i, (r, q) in enumerate(prog):
        tones = chord_tones(r + 24, q)
        tr.add(pad(tones, 8 * tr.beat, 0.6, 1.0, 7, 0.1), tr.at(2 * i), 0.0, 0.6, 0.45)
        for bar in (0, 1):
            b0 = 2 * i + bar
            for beat in (0, 1, 2, 3):
                tr.add(bass(r + 12 + (7 if beat == 2 else 0), 0.8 * tr.beat, 0.25), tr.at(b0, beat), 0.0, 0.4, 0.15)
            tr.add(drum(65, 0.25, 0.3, tr.rng, 1.5), tr.at(b0, 0), 0.0, 0.6, 0.2)
            tr.add(drum(65, 0.25, 0.3, tr.rng, 1.5), tr.at(b0, 2), 0.0, 0.45, 0.2)
            for beat in (1, 3, 3.5) if bar else (1, 3):
                tr.add(snare(tr.rng), tr.at(b0, beat), 0.1, 0.35, 0.3)
    scale = [midi(n) for n in ("D4", "E4", "F#4", "G4", "A4", "B4", "C5", "D5", "E5")]
    walk_melody(tr, prog, 2, scale, horn, [[(0, 1.5), (1.5, 0.5), (2, 2)], [(0, 2), (2, 1), (3, 1)], [(0, 1), (1, 1), (2, 2)]],
                0.15, 1.5, 0.4, 4, 4)
    return tr.render(2.4)


def boss() -> np.ndarray:
    # 140 BPM, 4/4, 32 bars (55 s). D minor: heavy toms, anvil hits, eighth-note bass, horn calls.
    tr = Track(140, 4, 32, seed=191)
    prog = [("D1", "m"), ("Bb0", "M"), ("C1", "M"), ("A0", "M")] * 4
    for i, (root, q) in enumerate(prog):
        r = midi(root) + 12
        tones = chord_tones(r + 24, q)
        second_half = i >= 8
        tr.add(pad(tones, 8 * tr.beat, 0.3, 0.5, 8, 0.2), tr.at(2 * i), 0.0, 0.5, 0.35)
        for b in range(16):
            tr.add(bass(r + (12 if b % 8 == 7 else 0), 0.45 * tr.beat, 0.8), tr.at(2 * i, b * 0.5), 0.0, 0.6, 0.1)
        for bar in (0, 1):
            b0 = 2 * i + bar
            for beat in (0, 0.5, 2, 2.75):
                tr.add(drum(50, 0.25, 0.5, tr.rng, 3.0), tr.at(b0, beat), 0.0, 0.85, 0.1)
            for beat in (1, 3):
                tr.add(snare(tr.rng), tr.at(b0, beat), 0.05, 0.85, 0.25)
            for e in range(8):
                tr.add(hat(tr.rng), tr.at(b0, e * 0.5), 0.3, 0.6, 0.1)
            tr.add(anvil(midi("D5"), tr.rng), tr.at(b0, 1), 0.5, 0.4, 0.3)
            if second_half:
                tr.add(drum(95, 0.2, 0.4, tr.rng, 1.2), tr.at(b0, 3.5), -0.4, 0.5, 0.3)
                tr.add(drum(75, 0.25, 0.4, tr.rng, 1.2), tr.at(b0, 3.75), 0.4, 0.5, 0.3)
        call = [(0, tones[0], 1.5), (1.5, tones[1], 0.5), (2, tones[2], 2), (4, tones[0] + 12, 3)]
        if second_half or i % 2 == 0:
            melody(tr, [(b, m + (12 if second_half else 0), d) for b, m, d in call], 2 * i, horn, 0.1, 1.7, 0.35)
    drone(tr, [midi("D1")], 0.1, 8)
    return tr.render(1.6)


def bell(m: float, dur: float = 3.2) -> np.ndarray:
    """Warm bronze handbell / cathedral chime tuned near fundamental m."""
    n = int(max(dur, 1.5) * SR)
    t = np.arange(n) / SR
    f = hz(m)
    sig = np.zeros(n)
    for ratio, amp, dec in ((0.5, 0.35, 0.9), (1.0, 1.0, 1.25), (1.183, 0.4, 1.7),
                            (1.5, 0.45, 2.0), (2.0, 0.55, 2.4), (2.76, 0.25, 3.8)):
        if f * ratio < SR / 2.2:
            sig += amp * np.sin(2 * np.pi * f * ratio * t) * np.exp(-t * dec)
    return sig * np.minimum(1.0, t / 0.008) * 0.13


def story_opening() -> np.ndarray:
    """56 BPM (dotted-quarter, 6/8), 48 bars (~102.9s). Solemn D minor cradle hymn with bell, no drums."""
    tr = Track(168, 6, 48, seed=211)
    prog = [("D2", "m"), ("D2", "m"), ("G1", "m"), ("A1", "sus"),
            ("D2", "m"), ("Bb1", "M"), ("G1", "m"), ("A1", "M")] * 2
    phrase_a = [(0, "D3", 1.5), (1.5, "F3", 0.5), (2, "A3", 1), (3, "G3", 1.5), (4.5, "F3", 0.5), (5, "E3", 1)]
    phrase_b = [(0, "D3", 1.5), (1.5, "F3", 0.5), (2, "Bb3", 1), (3, "A3", 1.5), (4.5, "G3", 0.5), (5, "E3", 1)]
    for i, (root, q) in enumerate(prog):
        r = midi(root)
        tones = chord_tones(r + 12, q)
        tr.add(pad(tones, 18 * tr.beat, 1.0, 1.8, 5, 0.14), tr.at(3 * i), 0.0, 0.75, 0.55)
        tr.add(bell(midi("D4") if i % 2 == 0 else midi("A3"), 4.2), tr.at(3 * i, 0), -0.35, 0.55, 0.75)
        tr.add(bell(midi("D3"), 4.5), tr.at(3 * i + 2, 0), 0.35, 0.38, 0.8)
        notes = phrase_a if i % 2 == 0 else phrase_b
        second_pass = i >= 8
        for bar_off in (0, 1):
            mel = [(b, midi(n) + (12 if second_pass and bar_off else 0), d * 0.9) for b, n, d in notes]
            melody(tr, mel, 3 * i + bar_off, lambda m, d: pad([m], d, 0.12, 0.35, 4, 0.08),
                   0.1 if bar_off else -0.1, 1.45, 0.65)
            tr.add(pluck(midi(notes[0][1]) + 12, 2.0 * tr.beat, 0.5, 2.2), tr.at(3 * i + bar_off, 0), -0.25, 0.3, 0.55)
    drone(tr, [midi("D1"), midi("A1"), midi("D2")], 0.16, 4)
    wash(tr, 140, 850, 3, 0.04)
    return tr.render(3.6)


def prologue_tension() -> np.ndarray:
    """84 BPM, 4/4, 32 bars (~91.4s). Hunted D minor stealth: uneven bass pulse, broken dulcimer motif, iron shutter."""
    tr = Track(84, 4, 32, seed=223)
    prog = [("D1", "m"), ("D1", "m"), ("Bb0", "M"), ("A0", "sus"),
            ("D1", "m"), ("G1", "m"), ("Bb0", "M"), ("A0", "M")] * 2
    lullaby_frags = [[(0.5, "D5", 0.8), (1.5, "F5", 1.2)], [(1.0, "A4", 0.8), (2.25, "G4", 1.2)],
                     [(0.5, "F4", 0.8), (2.0, "E4", 1.5)], [(1.5, "D4", 1.0), (3.0, "Bb4", 1.0)]]
    for i, (root, q) in enumerate(prog):
        r = midi(root) + 12
        tones = chord_tones(r + 24, q)
        tr.add(pad(tones[:2], 8 * tr.beat, 0.9, 1.4, 4, 0.18), tr.at(2 * i), 0.0, 0.42, 0.65)
        for bar in (0, 1):
            b0 = 2 * i + bar
            for beat in (0, 1.5, 2.75):
                tr.add(bass(r, 0.45 * tr.beat, 0.35), tr.at(b0, beat), -0.15, 0.38, 0.25)
                tr.add(drum(50, 0.18, 0.45, tr.rng, 1.6), tr.at(b0, beat), 0.0, 0.36, 0.35)
            tr.add(drum(46, 0.22, 0.3, tr.rng, 1.2), tr.at(b0, 3.5), 0.15, 0.25, 0.4)
            if (b0 + i) % 2 == 0:
                frag = lullaby_frags[(i + bar) % len(lullaby_frags)]
                melody(tr, [(b, midi(n), d) for b, n, d in frag], b0,
                       lambda m, d: pluck(m, d, 0.85, 2.0), 0.28, 0.72, 0.68)
        if i % 2 == 1:
            tr.add(anvil(midi("E5"), tr.rng), tr.at(2 * i + 1, 3.25), -0.45, 0.18, 0.82)
    drone(tr, [midi("D1"), midi("A1")], 0.19, 3)
    wash(tr, 180, 1400, 4, 0.07)
    return tr.render(3.8)


def story_escape() -> np.ndarray:
    """132 BPM, 4/4, 56 bars (~101.8s). Breathless D minor flight: staccato strings, chains, double-time Lullaby."""
    tr = Track(132, 4, 56, seed=233)
    prog = [("D1", "m"), ("C1", "M"), ("Bb0", "M"), ("A0", "M"),
            ("D1", "m"), ("F1", "M"), ("G1", "m"), ("A0", "M")] * 3 + [("D1", "m"), ("C1", "M"), ("Bb0", "M"), ("A0", "M")]
    phrase = [(0, "D5", 0.5), (0.5, "F5", 0.5), (1.0, "A5", 1.0), (2.0, "G5", 0.5), (2.5, "F5", 0.5), (3.0, "E5", 1.0),
              (4.0, "D5", 0.5), (4.5, "F5", 0.5), (5.0, "Bb5", 1.0), (6.0, "A5", 1.0), (7.0, "E5", 1.0)]
    for i, (root, q) in enumerate(prog):
        r = midi(root) + 12
        tones = chord_tones(r + 24, q)
        tr.add(pad(tones, 8 * tr.beat, 0.15, 0.4, 7, 0.16), tr.at(2 * i), 0.0, 0.46, 0.3)
        for b in range(16):
            tr.add(bass(r + (12 if b % 4 == 3 else 0), 0.35 * tr.beat, 0.7), tr.at(2 * i, b * 0.5), -0.1, 0.52, 0.12)
        for bar in (0, 1):
            b0 = 2 * i + bar
            for beat in (0, 0.75, 1.5, 2.0, 3.0):
                tr.add(drum(58, 0.18, 0.55, tr.rng, 2.8), tr.at(b0, beat), 0.0, 0.75, 0.15)
            for beat in (1, 2.5, 3.5):
                tr.add(snare(tr.rng), tr.at(b0, beat), 0.25, 0.65, 0.28)
            tr.add(anvil(midi("D5") + (7 if bar else 0), tr.rng), tr.at(b0, 1.5), 0.45, 0.3, 0.45)
        lead_notes = [(b, midi(n) - (12 if i < 14 else 0), d * 0.85) for b, n, d in phrase]
        melody(tr, lead_notes, 2 * i, horn, 0.1, 1.1, 0.36)
        melody(tr, lead_notes, 2 * i, lambda m, d: pluck(m, d, 1.3, 3.0), -0.15, 0.55, 0.4)
        if i % 3 == 2:
            tr.add(bell(midi("A4"), 2.2), tr.at(2 * i + 1, 2.75), -0.4, 0.4, 0.55)
    return tr.render(2.0)


def story_goodbye() -> np.ndarray:
    """60 BPM (dotted-quarter, 6/8), 56 bars (~112.0s). Quiet D minor -> F major Lullaby on music box and low viola."""
    tr = Track(180, 6, 56, seed=241)
    prog = [("D2", "m"), ("Bb1", "M"), ("F1", "M"), ("C2", "M"),
            ("D2", "m"), ("G1", "m"), ("Bb1", "M"), ("A1", "M")] * 3 + [("D2", "m"), ("Bb1", "M"), ("F1", "M"), ("A1", "M")]
    theme = [(0, "D5", 1.5), (1.5, "F5", 0.5), (2, "A5", 1), (3, "G5", 1.5), (4.5, "F5", 0.5), (5, "E5", 1),
             (6, "D5", 1.5), (7.5, "F5", 0.5), (8, "C5", 1), (9, "A4", 2.0), (11, "G4", 1.0)]
    answer = [(0, "A3", 2.5), (3, "Bb3", 1.5), (4.5, "A3", 1.5), (6, "F3", 2.5), (9, "E3", 3.0)]
    for i, (root, q) in enumerate(prog):
        r = midi(root)
        tones = chord_tones(r + 12, q)
        tr.add(pad(tones, 12 * tr.beat, 0.9, 1.4, 5, 0.1), tr.at(2 * i), 0.0, 0.58, 0.5)
        melody(tr, [(b, midi(n), d) for b, n, d in theme], 2 * i,
               lambda m, d: pluck(m, d, 1.25, 2.4), -0.18, 0.9, 0.48)
        if i % 2 == 1:
            melody(tr, [(b, midi(n), d) for b, n, d in answer], 2 * i, horn, 0.25, 0.95, 0.58)
        if i >= 16:
            melody(tr, [(b, midi(n), d) for b, n, d in theme], 2 * i,
                   lambda m, d: whistle(m, d, tr.rng), 0.32, 0.55, 0.65)
    drone(tr, [midi("D1"), midi("A1")], 0.12, 3)
    return tr.render(3.2)


def sunk_ward_danger() -> np.ndarray:
    """72 BPM, 4/4, 16 bars (~53.3s). C Phrygian damp drains: deep wash, metallic pipe drips, muted heartbeat."""
    tr = Track(72, 4, 16, seed=251)
    prog = [("C1", "m"), ("Db1", "M"), ("F1", "m"), ("G0", "sus"),
            ("C1", "m"), ("Ab0", "M"), ("F1", "m"), ("G0", "M")]
    for i, (root, q) in enumerate(prog):
        r = midi(root) + 12
        tones = chord_tones(r + 24, q)
        tr.add(pad(tones[:3], 8 * tr.beat, 1.1, 1.4, 5, 0.2), tr.at(2 * i), 0.0, 0.5, 0.65)
        for bar in (0, 1):
            b0 = 2 * i + bar
            tr.add(drum(45, 0.35, 0.15, tr.rng, 1.0), tr.at(b0, 0), 0.0, 0.72, 0.28)
            tr.add(drum(45, 0.28, 0.1, tr.rng, 0.9), tr.at(b0, 0.65), 0.0, 0.45, 0.28)
            drip = tones[int(tr.rng.integers(0, len(tones)))] + 24
            tr.add(bell(drip, 1.8), tr.at(b0, 2.25 if bar else 3.0), 0.45 - 0.9 * bar, 0.28, 0.85)
        if i % 2 == 1:
            tr.add(anvil(midi("G4"), tr.rng), tr.at(2 * i + 1, 3.25), -0.4, 0.22, 0.8)
    drone(tr, [midi("C1"), midi("G1"), midi("Db2")], 0.18, 3)
    wash(tr, 160, 1200, 4, 0.09)
    return tr.render(4.2)


def sunk_ward_boss() -> np.ndarray:
    """136 BPM, 4/4, 32 bars (~56.5s). F minor Inquisition dread: organ chords, pounding drums, cathedral bell."""
    tr = Track(136, 4, 32, seed=263)
    prog = [("F1", "m"), ("Db1", "M"), ("Eb1", "M"), ("C1", "M")] * 4
    for i, (root, q) in enumerate(prog):
        r = midi(root) + 12
        tones = chord_tones(r + 24, q)
        second = i >= 8
        tr.add(pad(tones + [tones[0] - 12], 8 * tr.beat, 0.25, 0.55, 8, 0.18), tr.at(2 * i), 0.0, 0.56, 0.42)
        for b in range(16):
            tr.add(bass(r + (7 if b % 8 == 6 else 0), 0.42 * tr.beat, 0.85), tr.at(2 * i, b * 0.5), 0.0, 0.58, 0.1)
        for bar in (0, 1):
            b0 = 2 * i + bar
            for beat in (0, 0.5, 2, 2.75):
                tr.add(drum(48, 0.26, 0.55, tr.rng, 3.0), tr.at(b0, beat), 0.0, 0.88, 0.12)
            for beat in (1, 3):
                tr.add(snare(tr.rng), tr.at(b0, beat), 0.05, 0.8, 0.25)
                tr.add(anvil(midi("F5"), tr.rng), tr.at(b0, beat), 0.42, 0.34, 0.35)
            for e in range(8):
                tr.add(hat(tr.rng, e == 7), tr.at(b0, e * 0.5), -0.3, 0.5, 0.12)
        tr.add(bell(midi("F3"), 3.0), tr.at(2 * i, 0), -0.35, 0.65, 0.6)
        call = [(0, tones[0], 1.5), (1.5, tones[1], 0.5), (2, tones[2], 2.0), (4, tones[0] + 12, 3.0)]
        if second or i % 2 == 0:
            melody(tr, [(b, m + (12 if second else 0), d) for b, m, d in call], 2 * i, horn, 0.1, 1.55, 0.38)
    drone(tr, [midi("F0"), midi("C1")], 0.12, 6)
    return tr.render(2.1)


def sunk_ward_shop() -> np.ndarray:
    """96 BPM, 3/4, 24 bars (~45.0s). Candlelit D minor chandler waltz: sparse lute, glass chimes, quiet whistle."""
    tr = Track(96, 3, 24, seed=271)
    prog = [("D1", "m"), ("G1", "m"), ("C1", "7"), ("F1", "M"),
            ("Bb0", "M"), ("G1", "m"), ("A0", "7"), ("D1", "m")]
    for i, (root, q) in enumerate(prog):
        r = midi(root) + 12
        tones = chord_tones(r + 24, q)
        for bar in range(3):
            b0 = 3 * i + bar
            tr.add(bass(r + (7 if bar == 1 else 0), 0.85 * tr.beat, 0.22), tr.at(b0, 0), -0.15, 0.45, 0.18)
            for beat in (1, 2):
                for k, m in enumerate(tones[:3]):
                    tr.add(pluck(m + 12, 0.65 * tr.beat, 0.55, 3.2), tr.at(b0, beat), (k - 1) * 0.3, 0.14, 0.32)
            tr.add(drum(135, 0.06, 0.8, tr.rng), tr.at(b0, 0), 0.2, 0.14, 0.22)
            if bar == 2:
                tr.add(bell(tones[-1] + 24, 1.4), tr.at(b0, 2.0), 0.38, 0.16, 0.6)
    scale = [midi(n) for n in ("D4", "E4", "F4", "G4", "A4", "Bb4", "C5", "D5")]
    walk_melody(tr, [(midi(r) + 12, q) for r, q in prog], 3, scale, lambda m, d: whistle(m, d, tr.rng),
                [[(0, 2), (2, 1)], [(0, 1), (1, 1), (2, 1)], [(0, 3)]], 0.12, 0.72, 0.42, 3)
    return tr.render(2.2)


def sunk_ward_travel() -> np.ndarray:
    """88 BPM, 4/4, 24 bars (~65.5s). Soot-haze D Dorian march: damp cobble pulse, low horn & whistle duet."""
    tr = Track(88, 4, 24, seed=281)
    prog = [("D1", "m"), ("C1", "M"), ("Bb0", "M"), ("A0", "m"),
            ("D1", "m"), ("G1", "m"), ("Bb0", "M"), ("A0", "sus"),
            ("D1", "m"), ("C1", "M"), ("G1", "m"), ("A0", "M")]
    for i, (root, q) in enumerate(prog):
        r = midi(root) + 12
        tones = chord_tones(r + 24, q)
        tr.add(pad(tones, 8 * tr.beat, 0.75, 1.1, 6, 0.12), tr.at(2 * i), 0.0, 0.52, 0.5)
        for bar in (0, 1):
            b0 = 2 * i + bar
            for beat in (0, 1, 2, 3):
                tr.add(bass(r + (7 if beat == 2 else 0), 0.75 * tr.beat, 0.3), tr.at(b0, beat), 0.0, 0.38, 0.18)
            tr.add(drum(56, 0.24, 0.35, tr.rng, 1.4), tr.at(b0, 0), 0.0, 0.55, 0.22)
            tr.add(drum(56, 0.22, 0.3, tr.rng, 1.3), tr.at(b0, 2), 0.0, 0.42, 0.22)
            for beat in (1, 3, 3.5) if bar else (1, 3):
                tr.add(snare(tr.rng), tr.at(b0, beat), 0.15, 0.26, 0.35)
    scale = [midi(n) for n in ("D4", "E4", "F4", "G4", "A4", "B4", "C5", "D5")]
    walk_melody(tr, [(midi(r) + 12, q) for r, q in prog], 2, scale, horn,
                [[(0, 1.5), (1.5, 0.5), (2, 2)], [(0, 2), (2, 1), (3, 1)], [(0, 1), (1, 1), (2, 2)]],
                -0.18, 1.25, 0.45, 3, 4)
    walk_melody(tr, [(midi(r) + 12, q) for r, q in prog], 2, [m + 12 for m in scale],
                lambda m, d: whistle(m, d, tr.rng),
                [[(0, 2), (2, 2)], [(1, 1), (2, 2)]], 0.28, 0.42, 0.55, 5, 3)
    wash(tr, 180, 1100, 3, 0.05)
    return tr.render(2.8)


def one_shot(stereo: np.ndarray, seconds: float, fade: float = 0.8) -> np.ndarray:
    # A sting is not a loop: keep the head, fade the tail out to silence.
    n = int(seconds * SR)
    out = stereo[:, :n].copy()
    k = int(fade * SR)
    out[:, -k:] *= np.linspace(1.0, 0.0, k) ** 2
    return out


def combat_sting() -> np.ndarray:
    """~3 s one-shot (TRANSITIONS1, AI-18): the ash wipe's hit into battle. A minor cluster stab, low drum, anvil."""
    tr = Track(100, 4, 4, seed=455684)
    tr.add(drum(40, 0.9, 0.6, tr.rng, 2.6), 0.0, 0.0, 1.0, 0.25)
    tr.add(anvil(midi("A4"), tr.rng), 0.02, -0.2, 0.55, 0.45)
    tr.add(pad([midi(n) for n in ("A2", "C3", "E3", "A#3")], 1.4, 0.01, 0.9, 8, 0.2), 0.0, 0.0, 0.7, 0.4)
    tr.add(drum(46, 0.5, 0.5, tr.rng, 2.2), 0.6, 0.1, 0.7, 0.25)
    tr.add(horn(midi("E3"), 1.1), 0.62, 0.15, 0.55, 0.35)
    return one_shot(tr.render(1.6, -1.0), 3.0)


def victory_stinger() -> np.ndarray:
    """~5.5 s one-shot (TRANSITIONS1, AI-18): Continue into the hub. D minor turns to F major, rising horn, a bell."""
    tr = Track(90, 4, 4, seed=455684)
    for i, n in enumerate(("D4", "F4", "A4")):
        tr.add(horn(midi(n), 0.5), i * 0.36, 0.1, 0.6, 0.35)
    tr.add(horn(midi("C5"), 2.6), 1.08, 0.1, 0.7, 0.4)
    tr.add(pad([midi(n) for n in ("F2", "C3", "F3", "A3", "C4")], 3.6, 0.3, 1.8, 7, 0.12), 1.0, 0.0, 0.6, 0.45)
    tr.add(bell(midi("F5"), 3.4), 1.08, -0.2, 0.45, 0.5)
    tr.add(drum(44, 0.8, 0.4, tr.rng, 2.4), 1.06, 0.0, 0.7, 0.2)
    return one_shot(tr.render(2.2, -1.5), 5.5, 1.6)


# --- ambience beds (AI-30, PR7-34 AMBIENCE1): seamless 48 s loops for the ambience bus, no melody ---------------

def ambience_level(stereo: np.ndarray, rms_db: float = -26.0, peak_db: float = -9.0) -> np.ndarray:
    # Sit low and steady: an even loudness (RMS) under a peak ceiling, so the bus volume is the only knob.
    rms = np.sqrt(np.mean(stereo ** 2))
    out = stereo * (10 ** (rms_db / 20.0) / max(rms, 1e-9))
    peak = np.max(np.abs(out))
    ceil = 10 ** (peak_db / 20.0)
    return out * min(1.0, ceil / max(peak, 1e-9))


def brown(tr: Track, cutoff: float) -> np.ndarray:
    # A circular brown-ish noise bed (integrated white noise, looped through the FFT so it has no seam).
    x = tr.rng.standard_normal(tr.n)
    f = np.fft.rfftfreq(tr.n, 1.0 / SR)
    spec = np.fft.rfft(x) / np.maximum(f, 20.0) / (1.0 + (f / cutoff) ** 4)
    y = np.fft.irfft(spec, tr.n)
    return y / np.max(np.abs(y))


def drip(rng: np.random.Generator) -> np.ndarray:
    # One water drop: a short sine ping that slides up (the bubble), then a tiny splash.
    n = int(0.35 * SR)
    t = np.arange(n) / SR
    f0 = rng.uniform(900, 1900)
    ph = 2 * np.pi * np.cumsum(f0 * (1 + 0.9 * np.minimum(t / 0.025, 1.0))) / SR
    sig = np.sin(ph) * np.exp(-t / 0.018)
    sig += lowpass(rng.standard_normal(n), 3000.0, 600.0) * np.exp(-t / 0.01) * 0.08
    return sig * 0.5


def pipe_groan(rng: np.random.Generator) -> np.ndarray:
    # A distant pipe: a low tone that bends and fades, with a little metal ring.
    dur = rng.uniform(2.5, 4.0)
    n = int(dur * SR)
    t = np.arange(n) / SR
    f = rng.uniform(55, 80) * (1 + 0.06 * np.sin(2 * np.pi * t / dur))
    ph = 2 * np.pi * np.cumsum(f) / SR
    e = np.sin(np.pi * t / dur) ** 2
    return (np.sin(ph) + 0.3 * np.sin(2.01 * ph) + 0.12 * np.sin(5.3 * ph)) * e * 0.5


def crackle(rng: np.random.Generator) -> np.ndarray:
    # One hearth pop: a click of bright noise, sometimes a short sputter after it.
    n = int(rng.uniform(0.01, 0.05) * SR)
    t = np.arange(n) / SR
    sig = lowpass(rng.standard_normal(n), rng.uniform(3000, 7000), 800.0) * np.exp(-t / rng.uniform(0.002, 0.01))
    return sig


def murmur(tr: Track, voices: int, gain: float) -> None:
    # Low talk across the room, no words: voiced noise through two moving vowel bands, gated in syllables.
    t = np.arange(tr.n) / SR
    bed = np.zeros((2, tr.n))
    for v in range(voices):
        L = tr.n / SR
        snap = lambda f: round(f * L) / L              # whole cycles over the loop: no seam
        f0 = snap(tr.rng.uniform(95, 190))
        src = np.zeros(tr.n)
        for h in range(1, 14):
            src += np.sin(2 * np.pi * f0 * h * (t + 0.003 * np.sin(2 * np.pi * snap(0.21) * t + v))) / h
        vowel = lowpass(src, tr.rng.uniform(700, 1100), tr.rng.uniform(250, 400))
        rate = snap(tr.rng.uniform(2.5, 4.0))
        syll = np.clip(np.sin(2 * np.pi * rate * t + tr.rng.uniform(0, 6)), 0, 1) ** 1.5
        k = int(round(tr.rng.uniform(3, 6)))
        phrase = np.clip(np.sin(2 * np.pi * k * t / (tr.n / SR) + v * 1.7) * 1.4 - 0.3, 0, 1)
        sig = vowel * syll * phrase
        sig = lowpass(sig, 1400.0)
        sig /= max(np.max(np.abs(sig)), 1e-9)
        pan = tr.rng.uniform(-0.7, 0.7)
        bed[0] += sig * np.cos((pan + 1) * np.pi / 4)
        bed[1] += sig * np.sin((pan + 1) * np.pi / 4)
    tr.add_bed(bed / voices, gain, 0.6)


def amb_drains() -> np.ndarray:
    """48 s: the drains and culverts. A low water rush, slow drips in a wet echo, a far pipe now and then."""
    tr = Track(60, 4, 12, seed=30101)
    flow = np.stack([brown(tr, 260.0) for _ in range(2)])
    flow *= 0.75 + 0.25 * np.stack([tr.lfo(3), tr.lfo(4, 1.3)])
    tr.add_bed(flow, 0.55, 0.3)
    wash(tr, 300.0, 1400.0, 2, 0.08, 0.5)
    t = 0.4
    while t < tr.n / SR:
        tr.add(drip(tr.rng), t, tr.rng.uniform(-0.8, 0.8), tr.rng.uniform(0.25, 0.6), 0.75)
        t += tr.rng.uniform(0.6, 2.8)
    for k in range(3):
        tr.add(pipe_groan(tr.rng), 6.0 + k * 15.0 + tr.rng.uniform(0, 5), tr.rng.uniform(-0.6, 0.6), 0.35, 0.6)
    return ambience_level(tr.render(3.2, -1.5))


def amb_streets() -> np.ndarray:
    """48 s: above ground. Wind through the rows that swells and drops, a far bell tolling twice, a loose shutter."""
    tr = Track(60, 4, 12, seed=30102)
    wash(tr, 180.0, 900.0, 3, 0.55, 0.7)
    wash(tr, 900.0, 3200.0, 5, 0.12, 0.9)
    for start in (9.0, 33.0):
        for i in range(3):
            tr.add(bell(midi("D3"), 6.0), start + i * 3.2, -0.35, 0.42, 0.85)
    for k in range(4):
        t0 = 4.0 + k * 12.0 + tr.rng.uniform(0, 4)
        for j in range(2):
            n = int(0.06 * SR)
            tt = np.arange(n) / SR
            knock = lowpass(tr.rng.standard_normal(n), 900.0, 120.0) * np.exp(-tt / 0.02)
            tr.add(knock, t0 + j * 0.18, 0.6, 0.25, 0.4)
    return ambience_level(tr.render(2.6, -1.5))


def amb_hub() -> np.ndarray:
    """48 s: the hub hall. A crackling hearth (a soft roar under pops), low murmurs across the room, a far drip."""
    tr = Track(60, 4, 12, seed=30103)
    roar = np.stack([brown(tr, 500.0) for _ in range(2)])
    roar *= 0.8 + 0.2 * np.stack([tr.lfo(7), tr.lfo(9, 0.7)])
    tr.add_bed(roar, 0.35, 0.15)
    t = 0.3
    while t < tr.n / SR:
        tr.add(crackle(tr.rng), t, tr.rng.uniform(-0.35, -0.05), tr.rng.uniform(0.04, 0.18), 0.15)
        t += tr.rng.exponential(0.35)
    murmur(tr, 5, 0.45)
    t = 2.0
    while t < tr.n / SR:
        tr.add(drip(tr.rng), t, 0.7, 0.12, 0.8)
        t += tr.rng.uniform(5.0, 9.0)
    return ambience_level(tr.render(1.8, -1.5))


AMBIENCE = {"amb_drains": amb_drains, "amb_streets": amb_streets, "amb_hub": amb_hub}   # audio/ambience/<key>.ogg


STINGS = {"combat_sting", "victory_stinger"}   # one-shots in audio/stings/ (AI-18 placeholders until Flow Music)
TRACKS = {"ashlands": ashlands, "iron_reaches": iron_reaches,
          "tidewrack": tidewrack, "combat": combat,
          "town": town, "danger": danger, "shop": shop, "travel": travel, "boss": boss,
          "story_opening": story_opening, "prologue_tension": prologue_tension,
          "story_escape": story_escape, "story_goodbye": story_goodbye,
          "sunk_ward_danger": sunk_ward_danger, "sunk_ward_boss": sunk_ward_boss,
          "sunk_ward_shop": sunk_ward_shop, "sunk_ward_travel": sunk_ward_travel,
          "combat_sting": combat_sting, "victory_stinger": victory_stinger, **AMBIENCE}
# ASSETORG2 layers: the kit's mood loops; every other track is the game's
WORKSPACE = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
LAYERS = {"kit": ("kits", "rpg", "assets"), "game": ("games", "emberwake", "assets")}
KIT_TRACKS = {"combat", "boss", "town", "shop", "danger", "travel"}


def layer_of(name: str) -> str:
    return "kit" if name in KIT_TRACKS else "game"


def track_path(root: str, name: str) -> str:
    folder = "stings" if name in STINGS else "ambience" if name in AMBIENCE else "music"
    return os.path.normpath(os.path.join(root, *LAYERS[layer_of(name)], "audio", folder, name + ".ogg"))


def find_ffmpeg() -> str | None:
    exe = shutil.which("ffmpeg")
    if exe:
        return exe
    try:
        import imageio_ffmpeg
        return imageio_ffmpeg.get_ffmpeg_exe()
    except Exception:
        return None


def write_ogg(stereo: np.ndarray, path: str, ffmpeg_exe: str) -> None:
    pcm = (np.clip(stereo.T, -1, 1) * 32767).astype("<i2")
    with tempfile.TemporaryDirectory() as tmp:
        wav_path = os.path.join(tmp, "x.wav")
        with wave.open(wav_path, "wb") as w:
            w.setnchannels(2)
            w.setsampwidth(2)
            w.setframerate(SR)
            w.writeframes(pcm.tobytes())
        subprocess.run([ffmpeg_exe, "-y", "-loglevel", "error", "-i", wav_path, "-c:a", "libvorbis",
                        "-q:a", "5", path], check=True)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("tracks", nargs="*")
    ap.add_argument("--layer", action="append", choices=list(LAYERS), help="only the tracks of these layers (default all)")
    ap.add_argument("--root", default=WORKSPACE, help="workspace root holding kits/rpg/assets and games/emberwake/assets")
    ap.add_argument("--list", action="store_true", help="print each track's layer and path, render nothing")
    args = ap.parse_args()
    names = args.tracks or list(TRACKS)
    for name in names:
        if name not in TRACKS:
            print("§ unknown track %s (known: %s)" % (name, ", ".join(TRACKS)))
            return 2
    names = [n for n in names if layer_of(n) in (args.layer or list(LAYERS))]
    if args.list:
        for name in names:
            print("%s %s %s" % (name, layer_of(name), track_path(args.root, name)))
        return 0
    ffmpeg_exe = find_ffmpeg()
    if ffmpeg_exe is None:
        print("§ ffmpeg not found")
        return 1
    for name in names:
        stereo = TRACKS[name]()
        path = track_path(args.root, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        write_ogg(stereo, path, ffmpeg_exe)
        print("§ wrote %s %.1fs" % (path, stereo.shape[1] / SR))
    return 0


if __name__ == "__main__":
    sys.exit(main())

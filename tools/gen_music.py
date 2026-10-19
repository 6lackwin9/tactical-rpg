#!/usr/bin/env python3
# Writes the generated music loops (MUSIC1-A): original tracks synthesized from notes, no samples.
#   python3 tools/gen_music.py [track ...] [--out DIR]      (default: every track, DIR = games/emberwake/assets/audio/music)
# Needs numpy and ffmpeg (libvorbis). Each track renders as one circular buffer, so notes, reverb and
# noise beds that run past the end wrap onto the start and the loop has no seam. Prints "§ wrote <path> <s>s".
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

def menu_theme() -> np.ndarray:
    # 64 BPM, 4/4, 16 bars (60 s). A minor; drum joins on the second pass of the progression.
    tr = Track(64, 4, 16, seed=11)
    prog = [("A2", "m"), ("F2", "M"), ("C3", "M"), ("G2", "M"), ("A2", "m"), ("D3", "m"), ("F2", "M"), ("E2", "M")]
    for i, (root, q) in enumerate(prog):
        r = midi(root)
        tones = chord_tones(r + 12, q)
        tr.add(pad(tones, 2 * 4 * tr.beat, 1.5, 1.8), tr.at(2 * i), 0.0, 1.0, 0.5)
        tr.add(bass(r, 3.5 * tr.beat, 0.1), tr.at(2 * i), -0.1, 0.55, 0.2)
        tr.add(bass(r, 3.5 * tr.beat, 0.1), tr.at(2 * i + 1), -0.1, 0.45, 0.2)
        for b in range(8):  # slow arpeggio in the low register
            m = tones[b % len(tones)] + (12 if b % 4 == 3 else 0)
            tr.add(pluck(m, tr.beat, 0.5, 2.2), tr.at(2 * i, b), -0.35, 0.35, 0.45)
    lines = [
        [(0, 69, 1.5), (1.5, 67, 0.5), (2, 64, 2), (4, 60, 1), (5, 62, 1), (6, 64, 2)],
        [(0, 65, 1.5), (1.5, 64, 0.5), (2, 60, 2), (4, 57, 1), (5, 60, 1), (6, 62, 2)],
        [(0, 64, 1), (1, 67, 1), (2, 72, 2), (4, 71, 1), (5, 67, 1), (6, 64, 2)],
        [(0, 62, 1.5), (1.5, 64, 0.5), (2, 67, 2), (4, 62, 2), (6, 59, 2)],
        [(0, 72, 1.5), (1.5, 71, 0.5), (2, 69, 2), (4, 64, 1), (5, 67, 1), (6, 69, 2)],
        [(0, 65, 1.5), (1.5, 64, 0.5), (2, 62, 2), (4, 65, 1), (5, 69, 1), (6, 74, 2)],
        [(0, 72, 1), (1, 69, 1), (2, 65, 2), (4, 64, 1), (5, 65, 1), (6, 67, 2)],
        [(0, 68, 1.5), (1.5, 69, 0.5), (2, 71, 2), (4, 68, 2), (6, 64, 2)],
    ]
    for i, line in enumerate(lines):
        melody(tr, line, 2 * i, lambda m, d: pluck(m, d, 1.0, 1.3), 0.2, 0.9, 0.4)
    for bar in range(8, 16):
        tr.add(drum(70, 0.25, 0.4, tr.rng), tr.at(bar, 0), 0.0, 0.5, 0.2)
        tr.add(drum(95, 0.15, 0.6, tr.rng), tr.at(bar, 2.5), 0.15, 0.25, 0.25)
        tr.add(drum(85, 0.18, 0.5, tr.rng), tr.at(bar, 3), -0.1, 0.3, 0.2)
    drone(tr, [midi("A1"), midi("E2")], 0.12)
    return tr.render(3.2)


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


TRACKS = {"menu_theme": menu_theme, "ashlands": ashlands, "iron_reaches": iron_reaches,
          "tidewrack": tidewrack, "combat": combat,
          "town": town, "danger": danger, "shop": shop, "travel": travel, "boss": boss}


def write_ogg(stereo: np.ndarray, path: str) -> None:
    pcm = (np.clip(stereo.T, -1, 1) * 32767).astype("<i2")
    with tempfile.TemporaryDirectory() as tmp:
        wav_path = os.path.join(tmp, "x.wav")
        with wave.open(wav_path, "wb") as w:
            w.setnchannels(2)
            w.setsampwidth(2)
            w.setframerate(SR)
            w.writeframes(pcm.tobytes())
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", wav_path, "-c:a", "libvorbis",
                        "-q:a", "5", path], check=True)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("tracks", nargs="*")
    ap.add_argument("--out", default=os.path.join(os.path.dirname(__file__), "..", "games", "emberwake", "assets", "audio", "music"))
    args = ap.parse_args()
    if shutil.which("ffmpeg") is None:
        print("§ ffmpeg not found")
        return 1
    names = args.tracks or list(TRACKS)
    for name in names:
        if name not in TRACKS:
            print("§ unknown track %s (known: %s)" % (name, ", ".join(TRACKS)))
            return 2
        stereo = TRACKS[name]()
        path = os.path.normpath(os.path.join(args.out, name + ".ogg"))
        write_ogg(stereo, path)
        print("§ wrote %s %.1fs" % (path, stereo.shape[1] / SR))
    return 0


if __name__ == "__main__":
    sys.exit(main())

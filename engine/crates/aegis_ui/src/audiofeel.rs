//! ENGINE-55 (AUDIO-FEEL1): the sound part of a `feel` replay (`shell::headless`, with any output: no window runs on
//! the virtual device, `audio_out`): each sound and track started during it, from the input or frame that asked for it
//! to its first audible sample, and what the output did (its buffer, underruns, the longest mix of a buffer).
//!
//! - `after_input_ms`: heard minus the last input before the ask (within `INPUT_WINDOW_MS`): what a player waits for an
//!   interface sound.
//! - `after_ask_ms`: heard minus the ask. A track is asked for on the first frame its screen is on top (after a
//!   transition the screen chose to show first): its `after_ask_ms` is the music change's own delay.
//! - `beat_ms`: an effect asked during a frame (a hit, a footstep: shown with its flash or step on that frame), heard
//!   minus when that frame is shown (its start plus one frame period, the next vsync of a paced 60 Hz frame): within
//!   one frame either way is on the beat.
//! - `decode_ms`: reading and decoding on the ask (an effect's first play; a track's decoder), on the frame.
//!
//! Every time is wall-clock ms from the replay's start.

use crate::audio::Bus;
use crate::audio_out::Snapshot;
use aegis_core::clock::Instant;
use serde_json::{json, Value};

/// An ask this soon after an input is that input's (a handler's sound, or the music of the screen it opened).
pub const INPUT_WINDOW_MS: f64 = 250.0;

/// The budgets a `feel` step may set on its sound report (the scenario runner's `AUDIO_BUDGETS`): the worst interface
/// sound after its input, the worst effect off its frame (either way), the worst music change after its ask (ms),
/// underruns (count) and the worst decode on an ask (ms).
pub const AUDIO_BUDGETS: [&str; 5] = ["ui_sound_ms", "hit_beat_ms", "music_ms", "underruns", "decode_ms"];

/// A sound started: (file, bus, asked, heard, decode ms) as `Audio::sounds_since` lists them.
pub type Started = (String, Bus, Instant, Option<Instant>, f64);

pub fn bus_name(b: Bus) -> &'static str {
    match b {
        Bus::Music => "music",
        Bus::Sfx => "sfx",
        Bus::Ui => "ui",
        Bus::Ambience => "ambience",
    }
}

fn ms(t: Instant, t0: Instant) -> f64 {
    let d = if t >= t0 { t.saturating_duration_since(t0).as_secs_f64() } else { -t0.saturating_duration_since(t).as_secs_f64() };
    (d * 1_000_000.0).round() / 1000.0
}

fn r3(x: f64) -> f64 {
    crate::perf::round_to(x, 3)
}

/// The report: `sounds` (each: `rel`, `bus`, `asked_ms`, `heard_ms`, `after_ask_ms`, `after_input_ms`, `beat_ms`,
/// `decode_ms`), the worst of each budgeted measure (null when nothing measured it), `unheard` (asked, never heard by
/// the end), and the output's `buffer_frames`, `buffer_ms`, `rate`, `virtual`, `underruns`, `underrun_ms`,
/// `underruns_stalled`, `underrun_stalled_ms` (the audio thread stopped by the host, ENGINE-61) and `worst_fill_ms`. `inputs`: when each input of the replay came; `frames`: each frame's start and end; `period_ms`:
/// the frame period a frame is shown after its start.
pub fn report(sounds: &[Started], t0: Instant, inputs: &[Instant], frames: &[(Instant, Instant)], out: &Snapshot, period_ms: f64) -> Value {
    let mut rows = Vec::new();
    let (mut ui, mut hit, mut music, mut decode) = (None::<f64>, None::<f64>, None::<f64>, None::<f64>);
    let worst = |w: &mut Option<f64>, v: f64| *w = Some(w.map_or(v, |x| x.max(v)));
    let mut unheard = 0;
    for (rel, bus, asked, heard, decode_ms) in sounds {
        let asked_ms = ms(*asked, t0);
        let heard_ms = heard.map(|h| ms(h, t0));
        unheard += heard.is_none() as u32;
        let input = inputs.iter().rev().map(|i| ms(*i, t0)).find(|i| *i <= asked_ms + 1e-9).filter(|i| asked_ms - i <= INPUT_WINDOW_MS);
        let after_input = heard_ms.zip(input).map(|(h, i)| r3(h - i));
        let frame = frames.iter().find(|(s, e)| s <= asked && asked <= e).map(|(s, _)| ms(*s, t0));
        let beat = match (bus, heard_ms, frame) {
            (Bus::Sfx, Some(h), Some(f)) => Some(r3(h - (f + period_ms))),
            _ => None,
        };
        match bus {
            Bus::Ui => after_input.into_iter().for_each(|v| worst(&mut ui, v)),
            Bus::Music => heard_ms.map(|h| r3(h - asked_ms)).into_iter().for_each(|v| worst(&mut music, v)),
            _ => {}
        }
        if let Some(b) = beat {
            worst(&mut hit, b.abs());
        }
        worst(&mut decode, *decode_ms);
        rows.push(json!({"rel": rel, "bus": bus_name(*bus), "asked_ms": asked_ms, "heard_ms": heard_ms, "after_ask_ms": heard_ms.map(|h| r3(h - asked_ms)),
            "after_input_ms": after_input, "beat_ms": beat, "decode_ms": r3(*decode_ms)}));
    }
    json!({
        "sounds": rows,
        "ui_sound_ms": ui, "hit_beat_ms": hit, "music_ms": music, "decode_ms": decode.map(r3), "unheard": unheard,
        "underruns": out.underruns.len(), "underrun_ms": out.underruns.iter().map(|u| ms(*u, t0)).collect::<Vec<f64>>(),
        "underruns_stalled": out.underruns_stalled.len(), "underrun_stalled_ms": out.underruns_stalled.iter().map(|u| ms(*u, t0)).collect::<Vec<f64>>(),
        "buffer_frames": out.buffer_frames, "buffer_ms": r3(out.buffer_ms()), "rate": out.rate, "virtual": out.virtual_device,
        "buffers": out.buffers, "worst_fill_ms": r3(out.worst_fill_ms),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::clock::Duration;

    fn at(t0: Instant, ms: u64) -> Instant {
        t0 + Duration::from_millis(ms)
    }

    /// A click's sound heard 15 ms after the click, a hit asked in a frame heard 6 ms after the frame shows, a track
    /// 20 ms after its ask; the worst of each (null when nothing measured it).
    #[test]
    fn each_sound_is_timed_from_its_input_or_its_frame() {
        let t0 = Instant::wall();
        let out = Snapshot { rate: 48_000, channels: 2, buffer_frames: 512, buffers: 10, underruns: vec![at(t0, 400)], underruns_stalled: vec![], worst_fill_ms: 0.5, virtual_device: true };
        let sounds: Vec<Started> = vec![
            ("sfx/ui_click.wav".into(), Bus::Ui, at(t0, 100), Some(at(t0, 115)), 0.2),
            ("sfx/hit.wav".into(), Bus::Sfx, at(t0, 202), Some(at(t0, 222)), 0.0),
            ("music/x.ogg".into(), Bus::Music, at(t0, 310), Some(at(t0, 330)), 4.0),
        ];
        let frames = [(at(t0, 200), at(t0, 205))];
        let r = report(&sounds, t0, &[at(t0, 100), at(t0, 300)], &frames, &out, 16.0);
        assert_eq!((r["ui_sound_ms"].as_f64(), r["music_ms"].as_f64()), (Some(15.0), Some(20.0)), "the track: 20 ms after its ask");
        assert_eq!(r["sounds"][1]["beat_ms"].as_f64(), Some(6.0), "heard at 222, the frame shown at 216");
        assert_eq!((r["underruns"].as_u64(), r["underrun_ms"][0].as_f64(), r["buffer_ms"].as_f64()), (Some(1), Some(400.0), Some(10.667)));
        assert_eq!((r["hit_beat_ms"].as_f64(), r["decode_ms"].as_f64(), r["unheard"].as_u64()), (Some(6.0), Some(4.0), Some(0)));
        let none = report(&[], t0, &[], &[], &out, 16.0);
        assert!(none["ui_sound_ms"].is_null() && none["hit_beat_ms"].is_null(), "nothing measured: null");
    }
}

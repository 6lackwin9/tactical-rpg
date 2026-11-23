//! ENGINE-55 (AUDIO-FEEL1): where the sounds go: the engine's own output (a mixer the device's callback pulls, or with no
//! window a virtual device that pulls it on a thread of its own in real time, as a device would), so the buffer is the
//! engine's choice and every sound can be timed to its first audible sample.
//!
//! - **The buffer**: `BUFFER_FRAMES` a callback where the device allows a fixed size (else its own, recorded); a sound
//!   asked for between two callbacks is mixed into the next one, and that buffer plays once the one before it has: from
//!   one to two buffers after the ask.
//! - **Heard**: each sound plays through a `Probe`, which notes the output frame its first non-silent sample is mixed
//!   into; `Stats::heard_at` turns that frame into the time it plays (the virtual device: exactly; a real device: from
//!   its last callback).
//! - **Underruns**: a callback whose buffer was not ready when the one before it ran out (the mixing took longer than a
//!   buffer, or the callback came late) counts one, with when.
//!
//! The mixing is rodio's (`dynamic_mixer`: each sink's queue converted to the output's channels and rate); the sounds
//! are the caller's (`crate::audio`).

use aegis_core::clock::{Duration, Instant};
use rodio::cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::cpal::{self, FromSample, SizedSample};
use rodio::dynamic_mixer::{self, DynamicMixer, DynamicMixerController};
use rodio::{Sample, Sink, Source};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// The frames (one sample per channel) a callback fills where the device lets the engine choose: 512 at 48 kHz is
/// 10.7 ms, so a sound asked for is heard 11 to 21 ms later.
pub const BUFFER_FRAMES: u32 = 512;
/// The virtual device's rate and channels (a common device's).
pub const VIRTUAL_RATE: u32 = 48_000;
pub const VIRTUAL_CHANNELS: u16 = 2;
/// A sample under this (of full scale) is silence: a sound is heard from its first sample over it.
pub const SILENT: f32 = 1.0 / 1024.0;
/// Underrun times kept (the latest).
const UNDERRUNS_KEPT: usize = 256;
/// Not heard yet.
const UNHEARD: u64 = u64::MAX;

/// What the output did, shared with its callback.
pub struct Stats {
    pub rate: u32,
    pub channels: u16,
    /// Output frames handed to the device so far (a probe reads it while its sample is mixed).
    pos: AtomicU64,
    /// The frames of the last callback.
    buffer_frames: AtomicU64,
    buffers: AtomicU64,
    underruns: Mutex<Vec<Instant>>,
    /// The longest a callback took to mix its buffer, in µs.
    worst_fill_us: AtomicU64,
    /// (the time frame `.1` plays, frame `.1`): the last callback's.
    anchor: Mutex<Option<(Instant, u64)>>,
}

/// A snapshot of `Stats`.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub rate: u32,
    pub channels: u16,
    pub buffer_frames: u64,
    pub buffers: u64,
    pub underruns: Vec<Instant>,
    pub worst_fill_ms: f64,
    pub virtual_device: bool,
}

impl Snapshot {
    /// One buffer's length in ms.
    pub fn buffer_ms(&self) -> f64 {
        self.buffer_frames as f64 * 1000.0 / self.rate.max(1) as f64
    }
}

impl Stats {
    fn new(rate: u32, channels: u16) -> Self {
        Self { rate, channels, pos: AtomicU64::new(0), buffer_frames: AtomicU64::new(0), buffers: AtomicU64::new(0), underruns: Mutex::new(Vec::new()), worst_fill_us: AtomicU64::new(0), anchor: Mutex::new(None) }
    }

    /// When output frame `frame` plays (None before the first callback).
    pub fn heard_at(&self, frame: u64) -> Option<Instant> {
        let (t, at) = (*self.anchor.lock().ok()?)?;
        let secs = |n: u64| Duration::from_secs_f64(n as f64 / self.rate.max(1) as f64);
        if frame >= at {
            t.checked_add(secs(frame - at))
        } else {
            t.checked_sub(secs(at - frame))
        }
    }

    fn underrun(&self, at: Instant) {
        if let Ok(mut u) = self.underruns.lock() {
            if u.len() >= UNDERRUNS_KEPT {
                u.remove(0);
            }
            u.push(at);
        }
    }

    /// Mixes one buffer (`out`, interleaved) from `mixer`; `plays`: when its first frame plays. Notes the frames as they
    /// are mixed (for the probes), the callback, and how long the mixing took.
    fn fill(&self, mixer: &mut DynamicMixer<f32>, out: &mut [f32], plays: Instant) {
        let start = Instant::wall();
        let channels = self.channels.max(1) as usize;
        let first = self.pos.load(Ordering::Relaxed);
        if let Ok(mut a) = self.anchor.lock() {
            *a = Some((plays, first));
        }
        for (i, frame) in out.chunks_mut(channels).enumerate() {
            self.pos.store(first + i as u64, Ordering::Relaxed);
            for s in frame {
                *s = mixer.next().unwrap_or(0.0);
            }
        }
        let frames = (out.len() / channels) as u64;
        self.pos.store(first + frames, Ordering::Relaxed);
        self.buffer_frames.store(frames, Ordering::Relaxed);
        self.buffers.fetch_add(1, Ordering::Relaxed);
        self.worst_fill_us.fetch_max(start.wall_elapsed().as_micros() as u64, Ordering::Relaxed);
    }

    fn snapshot(&self, virtual_device: bool) -> Snapshot {
        Snapshot {
            rate: self.rate,
            channels: self.channels,
            buffer_frames: self.buffer_frames.load(Ordering::Relaxed),
            buffers: self.buffers.load(Ordering::Relaxed),
            underruns: self.underruns.lock().map(|u| u.clone()).unwrap_or_default(),
            worst_fill_ms: self.worst_fill_us.load(Ordering::Relaxed) as f64 / 1000.0,
            virtual_device,
        }
    }

    fn reset(&self) {
        if let Ok(mut u) = self.underruns.lock() {
            u.clear();
        }
        self.worst_fill_us.store(0, Ordering::Relaxed);
        self.buffers.store(0, Ordering::Relaxed);
    }
}

/// What keeps the output running.
enum Hold {
    Device { _stream: cpal::Stream },
    #[cfg(not(target_arch = "wasm32"))]
    Virtual { _device: VirtualDevice },
}

/// The output the sounds play on.
pub struct Output {
    mixer: Arc<DynamicMixerController<f32>>,
    stats: Arc<Stats>,
    hold: Hold,
}

impl Output {
    /// The default output device (any other that opens when it does not), at its own rate and channels, with a
    /// `BUFFER_FRAMES` buffer where it allows one; None when none opens.
    pub fn device() -> Option<Self> {
        let host = cpal::default_host();
        let first = host.default_output_device();
        first.into_iter().chain(host.output_devices().into_iter().flatten()).find_map(|d| Self::on(&d))
    }

    fn on(device: &cpal::Device) -> Option<Self> {
        let format = device.default_output_config().ok()?;
        let mut config = format.config();
        let fixed = match format.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => (*min..=*max).contains(&BUFFER_FRAMES),
            cpal::SupportedBufferSize::Unknown => false,
        };
        // a fixed buffer where the device takes one; its own when that fails to open
        let sizes = if fixed { vec![cpal::BufferSize::Fixed(BUFFER_FRAMES), cpal::BufferSize::Default] } else { vec![cpal::BufferSize::Default] };
        sizes.into_iter().find_map(|size| {
            config.buffer_size = size;
            let stats = Arc::new(Stats::new(config.sample_rate.0, config.channels));
            let (mixer, rx) = dynamic_mixer::mixer::<f32>(config.channels, config.sample_rate.0);
            let stream = match format.sample_format() {
                cpal::SampleFormat::F32 => stream::<f32>(device, &config, rx, stats.clone()),
                cpal::SampleFormat::I16 => stream::<i16>(device, &config, rx, stats.clone()),
                cpal::SampleFormat::U16 => stream::<u16>(device, &config, rx, stats.clone()),
                cpal::SampleFormat::I32 => stream::<i32>(device, &config, rx, stats.clone()),
                cpal::SampleFormat::F64 => stream::<f64>(device, &config, rx, stats.clone()),
                _ => None,
            }?;
            stream.play().ok()?;
            Some(Self { mixer, stats, hold: Hold::Device { _stream: stream } })
        })
    }

    /// A virtual device (no window: scenarios, the gate): `VIRTUAL_RATE`, `VIRTUAL_CHANNELS`, `BUFFER_FRAMES` a buffer,
    /// pulled on a thread of its own in real time as a device's callback would be.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn virtual_device() -> Self {
        let stats = Arc::new(Stats::new(VIRTUAL_RATE, VIRTUAL_CHANNELS));
        let (mixer, rx) = dynamic_mixer::mixer::<f32>(VIRTUAL_CHANNELS, VIRTUAL_RATE);
        let hold = Hold::Virtual { _device: VirtualDevice::start(rx, stats.clone()) };
        Self { mixer, stats, hold }
    }

    /// A sink on this output (its sounds join the mix at the next callback).
    pub fn sink(&self) -> Sink {
        let (sink, rx) = Sink::new_idle();
        self.mixer.add(rx);
        sink
    }

    pub fn stats(&self) -> &Arc<Stats> {
        &self.stats
    }

    /// The virtual device (nothing is heard).
    pub fn is_virtual(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return matches!(self.hold, Hold::Virtual { .. });
        #[cfg(target_arch = "wasm32")]
        return false;
    }

    /// What the output did since `reset`.
    pub fn snapshot(&self) -> Snapshot {
        self.stats.snapshot(self.is_virtual())
    }

    /// Forgets the underruns, the worst fill and the buffer count (a feel replay starts).
    pub fn reset(&self) {
        self.stats.reset();
    }
}

/// A device stream of sample type `T` whose callback mixes `mixer`. Each callback's buffer plays once the one before
/// it has; a callback later than that (or that came after one buffer's time and took longer than that to mix) is an
/// underrun.
fn stream<T: SizedSample + FromSample<f32> + Send + 'static>(device: &cpal::Device, config: &cpal::StreamConfig, mut mixer: DynamicMixer<f32>, stats: Arc<Stats>) -> Option<cpal::Stream> {
    let mut mixed: Vec<f32> = Vec::new();
    let mut last: Option<(Instant, Duration)> = None;
    let channels = config.channels.max(1) as usize;
    let rate = config.sample_rate.0.max(1) as f64;
    device
        .build_output_stream::<T, _, _>(
            config,
            move |data: &mut [T], _| {
                let now = Instant::wall();
                let span = Duration::from_secs_f64((data.len() / channels) as f64 / rate);
                // the buffer before this one ran out before this callback came
                if last.is_some_and(|(t, d)| now.saturating_duration_since(t) > d + span) {
                    stats.underrun(now);
                }
                mixed.resize(data.len(), 0.0);
                stats.fill(&mut mixer, &mut mixed, now + span);
                for (d, s) in data.iter_mut().zip(&mixed) {
                    *d = <T as cpal::Sample>::from_sample(*s);
                }
                last = Some((now, span));
            },
            |e| eprintln!("audio output: {e}"),
            None,
        )
        .ok()
}

/// The virtual device's thread: buffer `k` plays from `t0 + k x buffer`; its callback comes when buffer `k - 1` starts
/// playing, and a buffer not mixed by the time it should play is an underrun.
#[cfg(not(target_arch = "wasm32"))]
struct VirtualDevice {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl VirtualDevice {
    fn start(mut mixer: DynamicMixer<f32>, stats: Arc<Stats>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let thread = std::thread::Builder::new()
            .name("aegis-audio".into())
            .spawn(move || {
                let span = Duration::from_secs_f64(BUFFER_FRAMES as f64 / VIRTUAL_RATE as f64);
                let mut buf = vec![0.0f32; BUFFER_FRAMES as usize * VIRTUAL_CHANNELS as usize];
                let mut t0 = Instant::wall();
                let mut k: u32 = 1;
                while !stopping.load(Ordering::Relaxed) {
                    let call = t0 + span * (k - 1);
                    let now = Instant::wall();
                    if let Some(wait) = call.checked_duration_since(now) {
                        std::thread::sleep(wait.into());
                    }
                    let plays = call + span;
                    stats.fill(&mut mixer, &mut buf, plays);
                    let done = Instant::wall();
                    if done > plays {
                        stats.underrun(done);
                        // far behind (a stopped process): start over from now rather than mix the time missed
                        if done.saturating_duration_since(plays) > span * 4 {
                            t0 = done;
                            k = 0;
                        }
                    }
                    k += 1;
                }
            })
            .ok();
        Self { stop, thread }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for VirtualDevice {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// A sound's first audible sample: the output frame it was mixed into (`heard`), once.
pub struct Probe<S> {
    inner: S,
    heard: Arc<AtomicU64>,
    stats: Arc<Stats>,
}

impl<S> Probe<S> {
    /// `inner` noting into a new mark (`unheard` until its first sample over `SILENT` is mixed).
    pub fn new(inner: S, stats: Arc<Stats>) -> (Self, Heard) {
        let heard = Heard::new(stats);
        (heard.probe(inner), heard)
    }
}

impl<S: Source> Iterator for Probe<S>
where
    S::Item: Sample,
{
    type Item = S::Item;

    #[inline]
    fn next(&mut self) -> Option<S::Item> {
        let s = self.inner.next()?;
        if self.heard.load(Ordering::Relaxed) == UNHEARD && s.to_f32().abs() > SILENT {
            self.heard.store(self.stats.pos.load(Ordering::Relaxed), Ordering::Relaxed);
        }
        Some(s)
    }
}

impl<S: Source> Source for Probe<S>
where
    S::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
}

/// When a probed sound is heard.
#[derive(Clone)]
pub struct Heard {
    frame: Arc<AtomicU64>,
    stats: Arc<Stats>,
}

impl Heard {
    /// A mark not heard yet, for a sound made later (`probe`: a track whose decoder is made on a thread).
    pub fn new(stats: Arc<Stats>) -> Self {
        Self { frame: Arc::new(AtomicU64::new(UNHEARD)), stats }
    }

    /// `inner` noting into this mark.
    pub fn probe<S>(&self, inner: S) -> Probe<S> {
        Probe { inner, heard: self.frame.clone(), stats: self.stats.clone() }
    }

    /// When its first audible sample plays; None until it is mixed (or when it is all silence).
    pub fn at(&self) -> Option<Instant> {
        match self.frame.load(Ordering::Relaxed) {
            UNHEARD => None,
            f => self.stats.heard_at(f),
        }
    }
}

/// A sound decoded once (`crate::audio`'s effects): its samples, played without decoding again.
pub struct Pcm {
    pub channels: u16,
    pub rate: u32,
    pub data: Arc<[i16]>,
}

impl Pcm {
    /// The whole of `source`.
    pub fn of<S: Source<Item = i16>>(source: S) -> Self {
        let (channels, rate) = (source.channels(), source.sample_rate());
        Self { channels, rate, data: source.collect::<Vec<i16>>().into() }
    }

    /// A playing of it.
    pub fn play(self: &Arc<Self>) -> PcmPlay {
        PcmPlay { pcm: self.clone(), at: 0 }
    }
}

/// `Pcm::play`.
pub struct PcmPlay {
    pcm: Arc<Pcm>,
    at: usize,
}

impl Iterator for PcmPlay {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        let s = *self.pcm.data.get(self.at)?;
        self.at += 1;
        Some(s)
    }
}

impl Source for PcmPlay {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.pcm.data.len() - self.at)
    }
    fn channels(&self) -> u16 {
        self.pcm.channels
    }
    fn sample_rate(&self) -> u32 {
        self.pcm.rate
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        let frames = self.pcm.data.len() as f64 / self.pcm.channels.max(1) as f64;
        Some(std::time::Duration::from_secs_f64(frames / self.pcm.rate.max(1) as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A virtual device plays a sound one to two buffers after it is asked for, from its first non-silent sample: a
    /// sound that opens with 100 ms of silence is heard 100 ms later.
    #[test]
    fn a_sound_is_heard_one_to_two_buffers_after_it_is_asked_for() {
        let out = Output::virtual_device();
        let span = BUFFER_FRAMES as f64 * 1000.0 / VIRTUAL_RATE as f64;
        for lead_ms in [0.0, 100.0] {
            let lead = (VIRTUAL_RATE as f64 * lead_ms / 1000.0) as usize;
            let mut data = vec![0i16; lead];
            data.extend(std::iter::repeat_n(8000i16, 4800));
            let pcm = Arc::new(Pcm { channels: 1, rate: VIRTUAL_RATE, data: data.into() });
            let sink = out.sink();
            let asked = Instant::wall();
            let (probe, heard) = Probe::new(pcm.play(), out.stats().clone());
            sink.append(probe);
            let t = Instant::wall();
            while heard.at().is_none() && t.wall_elapsed() < Duration::from_secs(2) {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            let at = heard.at().expect("heard");
            let ms = at.saturating_duration_since(asked).as_secs_f64() * 1000.0 - lead_ms;
            assert!(ms >= span - 1.0 && ms <= 2.0 * span + 1.0, "heard {ms} ms after the ask (lead {lead_ms}), buffers of {span} ms");
        }
        let s = out.snapshot();
        assert_eq!((s.buffer_frames, s.rate, s.virtual_device), (BUFFER_FRAMES as u64, VIRTUAL_RATE, true));
    }
}

//! MUSICVAR1 (2): music loop points. An Ogg Vorbis track tagged `LOOPSTART` / `LOOPLENGTH` (sample frames, in its
//! Vorbis comments) plays its intro once and then loops that span; an untagged track (or a WAV) loops whole, as before.
//! The tags are read from the bytes (`loop_points`); `looped` builds the source the music player appends.
//! Fixtures `tests/fixtures/music/loop_{tagged,untagged}.ogg`: the same 1 s mono 8 kHz chirp from ffmpeg (libvorbis),
//! the first tagged `LOOPSTART=2000`, `LOOPLENGTH=4000`.

use aegis_ui::audio::{looped, loop_points, IntroLoop, LoopPoints};
use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, Source};
use std::io::Cursor;
use std::path::Path;

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/music").join(name)).unwrap()
}

/// The lacing values of one packet.
fn lacing(len: usize) -> Vec<u8> {
    let mut out = vec![255u8; len / 255];
    out.push((len % 255) as u8);
    out
}

/// An Ogg file of `packets` (one logical stream, at most `per_page` segments a page, header pages at granule 0) and a
/// last page with a one-byte audio packet at granule `total`. CRCs are left 0: the tag reader does not check them.
fn ogg(packets: &[Vec<u8>], total: u64, per_page: usize) -> Vec<u8> {
    let mut segs: Vec<&[u8]> = Vec::new();
    for p in packets {
        let mut at = 0;
        for lace in lacing(p.len()) {
            segs.push(&p[at..at + lace as usize]);
            at += lace as usize;
        }
    }
    let mut out = Vec::new();
    let page = |out: &mut Vec<u8>, segs: &[&[u8]], granule: u64, seq: u32| {
        out.extend_from_slice(b"OggS\0\0");
        out.extend_from_slice(&granule.to_le_bytes());
        out.extend_from_slice(&7u32.to_le_bytes());
        out.extend_from_slice(&seq.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.push(segs.len() as u8);
        out.extend(segs.iter().map(|s| s.len() as u8));
        for s in segs {
            out.extend_from_slice(s);
        }
    };
    let mut seq = 0;
    for chunk in segs.chunks(per_page) {
        page(&mut out, chunk, 0, seq);
        seq += 1;
    }
    page(&mut out, &[&[0u8][..]], total, seq);
    out
}

fn vorbis_file(comments: &[&str], total: u64, per_page: usize) -> Vec<u8> {
    let mut id = b"\x01vorbis".to_vec();
    id.extend_from_slice(&[0u8; 23]);
    let mut c = b"\x03vorbis".to_vec();
    let vendor = b"test vendor";
    c.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    c.extend_from_slice(vendor);
    c.extend_from_slice(&(comments.len() as u32).to_le_bytes());
    for k in comments {
        c.extend_from_slice(&(k.len() as u32).to_le_bytes());
        c.extend_from_slice(k.as_bytes());
    }
    c.push(1);
    ogg(&[id, c, b"\x05vorbis setup".to_vec()], total, per_page)
}

#[test]
fn loop_tags_are_read_from_the_vorbis_comments() {
    let at = |start, length| Some(LoopPoints { start, length });
    assert_eq!(loop_points(&vorbis_file(&["TITLE=x", "LOOPSTART=44100", "LOOPLENGTH=88200"], 200_000, 255)), at(44_100, 88_200));
    assert_eq!(loop_points(&vorbis_file(&["loopstart=10", "LoopLength=20"], 100, 255)), at(10, 20), "keys in any case");
    assert_eq!(loop_points(&vorbis_file(&["LOOPSTART=30"], 100, 255)), at(30, 70), "no length: to the end");
    assert_eq!(loop_points(&vorbis_file(&["LOOPSTART=30", "LOOPLENGTH=0"], 100, 255)), at(30, 70), "length 0: to the end");
    assert_eq!(loop_points(&vorbis_file(&["LOOPSTART=30", "LOOPLENGTH=500"], 100, 255)), at(30, 70), "a span past the end stops there");
    let long = "ARTIST=".to_string() + &"a".repeat(1_000);
    assert_eq!(loop_points(&vorbis_file(&[&long, "LOOPSTART=5", "LOOPLENGTH=9"], 100, 2)), at(5, 9), "a comment header over several pages");
}

#[test]
fn untagged_or_unusable_tags_loop_the_whole_file() {
    for comments in [&["TITLE=x"][..], &[], &["LOOPSTART=100"], &["LOOPSTART=250"], &["LOOPSTART=0"], &["LOOPSTART=0", "LOOPLENGTH=100"], &["LOOPSTART=ten"], &["LOOPLENGTH=50"]] {
        assert_eq!(loop_points(&vorbis_file(comments, 100, 255)), None, "{comments:?}");
    }
    assert_eq!(loop_points(&vorbis_file(&["LOOPSTART=0", "LOOPLENGTH=40"], 100, 255)), Some(LoopPoints { start: 0, length: 40 }), "a short span from 0 is a loop");
    let mut cut = vorbis_file(&["LOOPSTART=10"], 100, 255);
    cut.truncate(cut.len() - 1);
    assert_eq!(loop_points(&cut), None, "a cut file has no length to check the tags against");
    assert_eq!(loop_points(b"RIFF\0\0\0\0WAVEfmt "), None, "a WAV has no Vorbis comments");
    assert_eq!(loop_points(b""), None);
}

fn mono(n: i16) -> SamplesBuffer<i16> {
    SamplesBuffer::new(1, 8000, (0..n).collect::<Vec<i16>>())
}

#[test]
fn a_tagged_source_plays_its_intro_then_loops_from_loopstart() {
    let got: Vec<i16> = IntroLoop::new(mono(100), LoopPoints { start: 30, length: 40 }).take(150).collect();
    let want: Vec<i16> = (0..70).chain(30..70).chain(30..70).collect();
    assert_eq!(got, want[..150].to_vec());
    // frames, not samples: a stereo span starts and ends on whole frames
    let stereo = SamplesBuffer::new(2, 44_100, (0..100).collect::<Vec<i16>>());
    let src = IntroLoop::new(stereo, LoopPoints { start: 10, length: 20 });
    assert_eq!((src.channels(), src.sample_rate(), src.total_duration()), (2, 44_100, None));
    let got: Vec<i16> = src.take(140).collect();
    let want: Vec<i16> = (0..60).chain(20..60).chain(20..60).collect();
    assert_eq!(got, want[..140].to_vec());
    // a span past the source's end repeats what it had of the span; one starting past it ends with the source
    let got: Vec<i16> = IntroLoop::new(mono(100), LoopPoints { start: 90, length: 50 }).take(120).collect();
    assert_eq!(got, (0..100).chain(90..100).chain(90..100).collect::<Vec<i16>>());
    assert_eq!(IntroLoop::new(mono(100), LoopPoints { start: 200, length: 5 }).count(), 100);
}

#[test]
fn an_untagged_source_loops_whole_as_before() {
    let got: Vec<i16> = looped(mono(100), None).take(250).collect();
    let want: Vec<i16> = (0..100).cycle().take(250).collect();
    assert_eq!(got, want);
    let got: Vec<i16> = looped(mono(100), Some(LoopPoints { start: 30, length: 40 })).take(110).collect();
    assert_eq!(got, (0..70).chain(30..70).collect::<Vec<i16>>(), "with points `looped` is the intro loop");
}

/// The fixtures through rodio's own decoder (the music player's path): the tagged file's tags are found, and after its
/// span's end it plays the samples from `LOOPSTART` again; the untagged file restarts at its first sample.
#[test]
fn a_real_vorbis_file_loops_from_its_loopstart_tag_and_an_untagged_one_from_its_start() {
    let tagged = fixture("loop_tagged.ogg");
    let p = loop_points(&tagged).expect("the fixture's LOOPSTART / LOOPLENGTH");
    assert_eq!(p, LoopPoints { start: 2000, length: 4000 });
    let once: Vec<i16> = Decoder::new(Cursor::new(tagged.clone())).unwrap().collect();
    let (start, end) = (p.start as usize, (p.start + p.length) as usize);
    assert!(once.len() >= end, "the decoded file ({} samples) holds the span", once.len());
    let played: Vec<i16> = looped(Decoder::new(Cursor::new(tagged)).unwrap(), Some(p)).take(end + 2 * (end - start) + 10).collect();
    assert_eq!(played[..end], once[..end], "the intro and the first pass play as decoded");
    assert_eq!(played[end..2 * end - start], once[start..end], "then the span from LOOPSTART");
    assert_eq!(played[2 * end - start..3 * end - 2 * start], once[start..end], "and again");
    assert_eq!(played[3 * end - 2 * start..], once[start..start + 10]);

    let untagged = fixture("loop_untagged.ogg");
    assert_eq!(loop_points(&untagged), None);
    let once: Vec<i16> = Decoder::new(Cursor::new(untagged.clone())).unwrap().collect();
    let played: Vec<i16> = looped(Decoder::new(Cursor::new(untagged)).unwrap(), None).take(once.len() + 100).collect();
    assert_eq!(played[..once.len()], once[..]);
    assert_eq!(played[once.len()..], once[..100], "the whole file again from its start");
}

/// Review M2, L1: malformed files never panic or allocate what a header claims; they loop whole (None).
#[test]
fn malformed_headers_and_lengths_loop_the_whole_file() {
    let tagged = |c: Vec<u8>, total: u64| {
        let mut id = b"\x01vorbis".to_vec();
        id.extend_from_slice(&[0u8; 23]);
        ogg(&[id, c, b"\x05vorbis setup".to_vec()], total, 255)
    };
    let comment = |vendor_len: u32, count: u32, entries: &[(u32, &str)]| {
        let mut c = b"\x03vorbis".to_vec();
        c.extend_from_slice(&vendor_len.to_le_bytes());
        c.extend_from_slice(&count.to_le_bytes());
        for (len, text) in entries {
            c.extend_from_slice(&len.to_le_bytes());
            c.extend_from_slice(text.as_bytes());
        }
        c
    };
    // a vendor length past the packet (u32::MAX: on wasm32 4 + it overflows usize), a comment length past it, a count
    // larger than the comments there
    assert_eq!(loop_points(&tagged(comment(u32::MAX, 1, &[(12, "LOOPSTART=10")]), 100)), None);
    assert_eq!(loop_points(&tagged(comment(0, 1, &[(u32::MAX, "LOOPSTART=10")]), 100)), None);
    assert_eq!(loop_points(&tagged(comment(0, u32::MAX, &[(12, "LOOPSTART=10")]), 100)), None);
    assert_eq!(loop_points(&tagged(comment(0, 1, &[(12, "LOOPSTART=10")]), 100)), Some(LoopPoints { start: 10, length: 90 }), "the same, well formed");
    // a header cut mid-packet (the file ends inside the comment packet)
    let whole = vorbis_file(&["LOOPSTART=10"], 100, 255);
    for cut in [30, 60, 75] {
        assert_eq!(loop_points(&whole[..cut]), None, "cut at {cut}");
    }
    // a first stream that is not Vorbis (Opus)
    let opus = ogg(&[b"OpusHead\x01\x02".to_vec(), b"OpusTags\0\0\0\0\x01\0\0\0\x0cLOOPSTART=10".to_vec()], 100, 255);
    assert_eq!(loop_points(&opus), None);
    // a last granule the bytes could never hold (u64::MAX - 1, and just past the bound): no length to trust
    for total in [u64::MAX - 1, 1 << 40] {
        assert_eq!(loop_points(&vorbis_file(&["LOOPSTART=10", "LOOPLENGTH=20"], total, 255)), None, "granule {total}");
    }
}

/// Review M2: the points are public, so any span is safe: nothing is reserved for what the source never decodes.
#[test]
fn a_huge_span_neither_panics_nor_reserves_it() {
    let got: Vec<i16> = IntroLoop::new(mono(100), LoopPoints { start: 10, length: u64::MAX }).take(150).collect();
    assert_eq!(got, (0..100).chain(10..60).collect::<Vec<i16>>());
    let got: Vec<i16> = IntroLoop::new(mono(100), LoopPoints { start: u64::MAX, length: u64::MAX }).take(150).collect();
    assert_eq!(got, (0..100).collect::<Vec<i16>>(), "a start past everything: the source once");
}

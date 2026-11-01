//! INPUT1-AP: the pip gauge (`aegis_ui::pips`): its reading for the spec's worked rows (INPUT1.md 7, AP7), the pip
//! scale above the cap (AP6), the debt and shortfall labels and the alert (AP8), the live (spending) look (AP10), its
//! widget and its drawing on a 640x360 canvas with the engine font, and the Gallery samples.

use aegis_ui::canvas::Canvas;
use aegis_ui::painter::{Fonts, View};
use aegis_ui::pips::{count, pips_w, samples, unit, PipGauge, LABEL_GAP, PIP_GAP, PIP_H, PIP_W};
use aegis_ui::ui::{Gfx, Kit, Ui, BG, DIM, FOE, GOLD, TEXT};

fn gfx() -> Gfx {
    let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font), kit: Kit::default() };
    g.canvas.clear(BG);
    g
}

fn px(g: &Gfx, x: i32, y: i32) -> [u8; 4] {
    let i = ((y * g.canvas.w + x) * 4) as usize;
    [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2], g.canvas.px[i + 3]]
}

/// AP7 written out: pip `i` full when `i <= left`, a ghost fill when `left < i <= next`, a solid outline when `i <= now`.
fn by_rule(now: i32, left: i32, next: i32, max: i32) -> String {
    (1..=max)
        .map(|i| match (i <= left, left < i && i <= next, i <= now) {
            (true, _, _) => "F",
            (false, true, true) => "Hg",
            (false, false, true) => "H",
            (false, true, false) => "Sg",
            _ => "S",
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The four worked rows of INPUT1.md 7 read exactly as the spec's table, which is AP7's rule.
#[test]
fn the_worked_rows_read_as_the_spec_table() {
    let rows = [
        ((10, 3, 7, 10), "F F F Hg Hg Hg Hg H H H"),
        ((10, 4, 8, 10), "F F F F Hg Hg Hg Hg H H"),
        ((10, 2, 6, 10), "F F Hg Hg Hg Hg H H H H"),
        ((3, 0, 5, 10), "Hg Hg Hg Sg Sg S S S S S"),
    ];
    for ((now, left, next, max), want) in rows {
        let g = PipGauge::planned(now, left, next, max, 12);
        assert_eq!(g.reading(), want, "now {now} left {left} next {next}");
        assert_eq!(g.reading(), by_rule(now, left, next, max));
        assert_eq!((g.slots, g.unit, g.alert, g.before.clone(), g.after.clone()), (max, 1, false, None, None));
    }
}

/// AP6: up to the cap a pip is a point; above it a pip is `ceil(max / cap)` points and every count is divided by it and
/// rounded up (a 14 point budget: 7 pips at 2 each).
#[test]
fn above_the_cap_a_pip_stands_for_several_points() {
    assert_eq!((unit(10, 12), unit(12, 12), unit(13, 12), unit(14, 12), unit(25, 12)), (1, 1, 2, 2, 3));
    assert_eq!((count(0, 2), count(1, 2), count(2, 2), count(3, 2), count(-4, 2)), (0, 1, 1, 2, 0));
    let g = PipGauge::planned(14, 9, 12, 14, 12);
    assert_eq!((g.slots, g.unit), (7, 2));
    assert_eq!(g.reading(), "F F F F F Hg H", "left 9 -> 5 pips, next 12 -> 6, now 14 -> 7");
    assert_eq!(pips_w(12), 12 * PIP_W + 11 * PIP_GAP);
    assert_eq!(pips_w(12), 59, "12 pips take 59 px (INPUT1 AP6)");
}

/// AP8: a debt draws every pip as an empty slot with "-<debt>" before them; a pick beyond the budget turns the outlines
/// to the alert and puts "-<short>" after them.
#[test]
fn a_debt_and_a_shortfall_label_the_gauge() {
    let debt = PipGauge::planned(-2, -2, 1, 10, 12);
    assert_eq!(debt.reading(), vec!["S"; 10].join(" "));
    assert_eq!((debt.before.as_deref(), debt.after.as_deref(), debt.alert), (Some("-2"), None, false));
    let short = PipGauge::planned(3, 3, 5, 10, 12).short(1);
    assert_eq!((short.before.as_deref(), short.after.as_deref(), short.alert), (None, Some("-1"), true));
    assert_eq!(short.reading(), PipGauge::planned(3, 3, 5, 10, 12).reading(), "the alert changes colours only");
    let both = PipGauge::planned(-2, -2, 1, 10, 12).short(6);
    assert_eq!((both.before.as_deref(), both.after.as_deref(), both.alert), (Some("-2"), None, true), "a debt keeps its label");
}

/// AP10: spending, full pips up to what is held now and hollow ones up to what was held at the start; a fractional
/// level (an animation) reads from its half.
#[test]
fn the_live_look_and_fractional_levels() {
    let g = PipGauge::live(10, 6, 10, 12);
    assert_eq!(g.reading(), "F F F F F F H H H H", "4 points paid: four hollow pips");
    let mut mid = g.clone();
    mid.fill = 7.6;
    assert_eq!(mid.reading(), "F F F F F F F F H H", "7.6 reads as 8 full");
    mid.fill = 7.4;
    assert_eq!(mid.reading(), "F F F F F F F H H H");
    assert_eq!(PipGauge::live(5, 8, 10, 12).reading(), "F F F F F F F F S S", "a gain mid-spend: the outline follows");
}

/// The gauge draws its pips (a full one filled, a slot dim), its labels in the alert colour, and pushes one info widget
/// with its reading and hover card over the whole of it.
#[test]
fn the_gauge_draws_and_pushes_its_widget() {
    let mut g = gfx();
    let gauge = PipGauge::planned(3, 1, 5, 10, 12).short(2).with_tip(vec![("numbers".into(), GOLD)]);
    let (rect, width, widgets) = {
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        let width = gauge.width(&mut ui);
        let rect = gauge.draw(&mut ui, "ap", 20, 40);
        (rect, width, ui.widgets)
    };
    let w = widgets.iter().find(|w| w.id == "ap").expect("the gauge's widget");
    assert_eq!((w.kind, w.text.as_str(), w.rect, w.action.is_none()), ("pips", "F Hg Hg Sg Sg S S S S S", rect, true));
    assert_eq!(w.tip, vec![("numbers".to_string(), GOLD)]);
    assert_eq!(rect.w, width, "the drawn width is `width`");
    assert!(rect.w > pips_w(10) + LABEL_GAP, "the label after the pips");
    let y = 40 - PIP_H / 2;
    // pip 1 is full: its inside is the fill colour, its outline the alert's
    let inside = px(&g, 20 + 1, y + 2);
    assert_eq!(&inside[..3], &GOLD[..3]);
    assert_eq!(&px(&g, 20, y)[..3], &FOE[..3]);
    // pip 10 is an empty slot: the inside stays the background
    let p10 = 20 + 9 * (PIP_W + PIP_GAP);
    assert_eq!(&px(&g, p10 + 1, y + 2)[..3], &BG[..3]);
    // without the alert: a solid outline is TEXT, a slot's DIM
    let mut g2 = gfx();
    {
        let mut ui: Ui<()> = Ui::new(&mut g2, (-1, -1), true);
        PipGauge::planned(3, 1, 5, 10, 12).draw(&mut ui, "ap", 20, 40);
    }
    assert_eq!(&px(&g2, 20, y)[..3], &TEXT[..3]);
    assert_eq!(&px(&g2, p10, y)[..3], &DIM[..3]);
}

/// The Gallery's samples: the four worked rows, a red one, a debt, a 14 point budget at 2 a pip, a spending one; none
/// wider than 12 pips.
#[test]
fn the_samples_cover_the_states() {
    let s = samples();
    let read: Vec<String> = s.iter().map(|x| x.1.reading()).collect();
    assert!(read.contains(&"F F F Hg Hg Hg Hg H H H".to_string()));
    assert!(read.contains(&"Hg Hg Hg Sg Sg S S S S S".to_string()));
    assert!(s.iter().any(|x| x.1.alert && x.1.after.is_some()), "red");
    assert!(s.iter().any(|x| x.1.before.is_some()), "a debt");
    assert!(s.iter().any(|x| x.1.unit == 2 && x.1.slots == 7), "14 at 2 a pip");
    assert!(s.iter().all(|x| x.1.slots <= 12 && !x.1.tip.is_empty()));
}

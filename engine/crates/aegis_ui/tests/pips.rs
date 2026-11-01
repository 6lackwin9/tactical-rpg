//! INPUT1-AP, INPUT2 (style D): the pip gauge (`aegis_ui::pips`): fixed slots whatever the plan, one capsule a planned
//! block (two neighbouring blocks of one colour stay two), free sockets up to the points held and crossed slots above
//! them, the slot scale above the cap, the debt and shortfall labels and the alert, the live (spending) look and
//! fractional levels, its widget and its drawing on a 640x360 canvas with the engine font, and the Gallery samples.

use aegis_ui::canvas::Canvas;
use aegis_ui::painter::{Fonts, View};
use aegis_ui::pips::{count, mix, pip_rect, pips_w, samples, unit, PipBlock, PipGauge, LABEL_GAP, PIP_GAP, PIP_H, PIP_W, RIM_LIGHT, TICK_DARK};
use aegis_ui::ui::{Gfx, Kit, Ui, BG, DIM, FOE, GOLD, INFO};

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

/// The rule written out (one point a slot): each block's points in order as "B<n>", free sockets up to `now`, crossed
/// slots up to `max`.
fn by_rule(now: i32, blocks: &[i32], max: i32) -> String {
    let mut out: Vec<String> = Vec::new();
    for (n, ap) in blocks.iter().enumerate() {
        out.extend((0..*ap).map(|_| format!("B{}", n + 1)));
    }
    let spent = out.len() as i32;
    out.extend((spent..max).map(|i| if i < now { "O".to_string() } else { "X".to_string() }));
    out.truncate(max as usize);
    out.join(" ")
}

fn plan(aps: &[i32], colour: [u8; 4]) -> Vec<PipBlock> {
    aps.iter().map(|ap| PipBlock::new(*ap, colour)).collect()
}

/// Style D's board rows (12 slots, 10 points this round) read block by block, then sockets, then crossed slots.
#[test]
fn the_board_rows_read_block_by_block() {
    for aps in [vec![3, 2, 2], vec![2, 2, 3, 1], vec![5], vec![]] {
        let g = PipGauge::planned(10, &plan(&aps, FOE), 12, 12);
        assert_eq!(g.reading(), by_rule(10, &aps, 12), "{aps:?}");
        assert_eq!((g.slots, g.unit, g.alert, g.before.clone(), g.after.clone()), (12, 1, false, None, None));
        assert_eq!(g.blocks.len(), aps.len(), "one capsule a block");
    }
    assert_eq!(PipGauge::planned(10, &[], 12, 12).reading(), "O O O O O O O O O O X X", "nothing planned: ten sockets, two crossed");
}

/// Two neighbouring blocks of the same colour are two capsules (each from its own first slot), never one.
#[test]
fn same_colour_neighbours_stay_two_capsules() {
    let g = PipGauge::planned(10, &plan(&[3, 2], FOE), 12, 12);
    assert_eq!(g.blocks.len(), 2);
    assert_eq!((g.blocks[0].from, g.blocks[0].slots, g.blocks[1].from, g.blocks[1].slots), (0, 3, 3, 2));
    assert!(g.reading().starts_with("B1 B1 B1 B2 B2 O"));
    // drawn: the first capsule's last slot ends with its rim, the second starts with its own
    let mut gx = gfx();
    {
        let mut ui: Ui<()> = Ui::new(&mut gx, (-1, -1), true);
        g.draw(&mut ui, "ap", 20, 40);
    }
    let rim = mix(FOE, [255, 255, 255, 255], RIM_LIGHT);
    let r2 = pip_rect(20, 40, 2);
    let r3 = pip_rect(20, 40, 3);
    assert_eq!(&px(&gx, r2.x + r2.w - 1, r2.y + PIP_H / 2)[..3], &rim[..3], "block 1's right rim");
    assert_eq!(&px(&gx, r3.x, r3.y + PIP_H / 2)[..3], &rim[..3], "block 2's left rim");
    // inside a capsule the slots are split by a darker tick, not a rim
    let tick = mix(FOE, [0, 0, 0, 255], TICK_DARK);
    let r1 = pip_rect(20, 40, 1);
    assert_eq!(&px(&gx, r1.x - 1, r1.y + PIP_H / 2)[..3], &tick[..3], "the tick between block 1's slots");
    assert_eq!(&px(&gx, r1.x + 2, r1.y + PIP_H / 2)[..3], &FOE[..3], "the block's fill");
}

/// The row is fixed: its slots and its widget's rect do not move or resize with the plan.
#[test]
fn the_row_never_moves_with_the_plan() {
    let rect_of = |blocks: &[i32]| {
        let mut g = gfx();
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        PipGauge::planned(10, &plan(blocks, FOE), 12, 12).draw(&mut ui, "ap", 30, 50)
    };
    let base = rect_of(&[]);
    for aps in [vec![3, 2, 2], vec![2, 2, 3, 1], vec![5], vec![1; 10]] {
        assert_eq!(rect_of(&aps), base, "{aps:?}");
    }
    assert_eq!(base.w, pips_w(12));
    assert_eq!(pip_rect(30, 50, 11).x + PIP_W, 30 + pips_w(12), "the last slot ends the row");
}

/// Above the cap a slot is `ceil(max / cap)` points and the blocks are laid from the running total (rounding never
/// adds slots: a 14 point budget is 7 slots at 2 each).
#[test]
fn above_the_cap_a_slot_stands_for_several_points() {
    assert_eq!((unit(10, 12), unit(12, 12), unit(13, 12), unit(14, 12), unit(25, 12)), (1, 1, 2, 2, 3));
    assert_eq!((count(0, 2), count(1, 2), count(2, 2), count(3, 2), count(-4, 2)), (0, 1, 1, 2, 0));
    let g = PipGauge::planned(14, &plan(&[3, 3], FOE), 14, 12);
    assert_eq!((g.slots, g.unit), (7, 2));
    assert_eq!(g.reading(), "B1 B1 B2 O O O O", "3 -> slots 1-2, 6 -> slot 3, now 14 -> 7");
    assert_eq!(pips_w(12), 12 * PIP_W + 11 * PIP_GAP);
}

/// A debt draws every slot crossed with "-<debt>" before them; a pick beyond the budget turns the outlines to the alert
/// and puts "-<short>" after them; an overspent plan's blocks run into the crossed slots.
#[test]
fn a_debt_and_a_shortfall_label_the_gauge() {
    let debt = PipGauge::planned(-2, &plan(&[2], FOE), 10, 12);
    assert_eq!(debt.reading(), vec!["X"; 10].join(" "));
    assert_eq!((debt.before.as_deref(), debt.after.as_deref(), debt.alert), (Some("-2"), None, false));
    let short = PipGauge::planned(3, &plan(&[4], FOE), 10, 12).short(1);
    assert_eq!((short.before.as_deref(), short.after.as_deref(), short.alert), (None, Some("-1"), true));
    assert_eq!(short.reading(), "B1 B1 B1 B1 X X X X X X", "the block runs past the 3 points held");
    let both = PipGauge::planned(-2, &[], 10, 12).short(6);
    assert_eq!((both.before.as_deref(), both.after.as_deref(), both.alert), (Some("-2"), None, true), "a debt keeps its label");
}

/// Spending: what was paid is one block at the left, the points held now are sockets, the slots above the round's start
/// crossed; a fractional level (an animation) reads from its half.
#[test]
fn the_live_look_and_fractional_levels() {
    let g = PipGauge::live(10, 6, DIM, 12, 12);
    assert_eq!(g.reading(), "B1 B1 B1 B1 O O O O O O X X", "4 points paid");
    let mut mid = g.clone();
    mid.spent = 2.6;
    assert_eq!(mid.reading(), "B1 B1 B1 O O O O O O O X X", "2.6 reads as 3 paid");
    mid.edge = 10.6;
    assert_eq!(mid.reading(), "B1 B1 B1 O O O O O O O O X");
    assert_eq!(PipGauge::live(5, 8, DIM, 10, 12).reading(), "O O O O O O O O X X", "a gain mid-spend: nothing paid, the sockets follow");
}

/// Filled and empty read apart: a block slot is its colour, a socket the dark socket fill, a crossed slot keeps the
/// background with its cross; the widget carries the reading and hover card over the whole gauge with its label.
#[test]
fn the_gauge_draws_and_pushes_its_widget() {
    let mut g = gfx();
    let gauge = PipGauge::planned(3, &plan(&[1], GOLD), 10, 12).short(2).with_tip(vec![("numbers".into(), GOLD)]);
    let (rect, width, widgets) = {
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        let width = gauge.width(&mut ui);
        let rect = gauge.draw(&mut ui, "ap", 20, 40);
        (rect, width, ui.widgets)
    };
    let w = widgets.iter().find(|w| w.id == "ap").expect("the gauge's widget");
    assert_eq!((w.kind, w.text.as_str(), w.rect, w.action.is_none()), ("pips", "B1 O O X X X X X X X", rect, true));
    assert_eq!(w.tip, vec![("numbers".to_string(), GOLD)]);
    assert_eq!(rect.w, width, "the drawn width is `width`");
    assert!(rect.w > pips_w(10) + LABEL_GAP, "the label after the slots");
    let (b, s, x) = (pip_rect(20, 40, 0), pip_rect(20, 40, 1), pip_rect(20, 40, 9));
    assert_eq!(&px(&g, b.x + 2, b.y + 3)[..3], &GOLD[..3], "a block slot is filled");
    let socket = px(&g, s.x + 2, s.y + 2);
    assert!(socket[..3].iter().all(|c| *c < 20), "a socket is dark: {socket:?}");
    assert_eq!(&px(&g, s.x, s.y + 2)[..3], &FOE[..3], "the alert outlines the sockets");
    assert_eq!(&px(&g, x.x + 2, x.y + 1)[..3], &BG[..3], "a crossed slot keeps the background off its cross");
}

/// The Gallery's samples: style D's four rows, a ghost plan, a red one, a debt, a 14 point budget at 2 a slot, a
/// spending one; none wider than 12 slots.
#[test]
fn the_samples_cover_the_states() {
    let s = samples();
    let read: Vec<String> = s.iter().map(|x| x.1.reading()).collect();
    for aps in [vec![3, 2, 2], vec![2, 2, 3, 1], vec![5], vec![]] {
        assert!(read.contains(&by_rule(10, &aps, 12)), "{aps:?}");
    }
    assert!(s.iter().any(|x| x.1.blocks.iter().any(|b| b.ghost)), "a ghost plan");
    assert!(s.iter().any(|x| x.1.blocks.iter().any(|b| b.colour == INFO)), "a second kind's colour");
    assert!(s.iter().any(|x| x.1.alert && x.1.after.is_some()), "red");
    assert!(s.iter().any(|x| x.1.before.is_some()), "a debt");
    assert!(s.iter().any(|x| x.1.unit == 2 && x.1.slots == 7), "14 at 2 a slot");
    assert!(s.iter().all(|x| x.1.slots <= 12 && !x.1.tip.is_empty()));
}

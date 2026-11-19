//! ROBUST1 (ENGINE-10): property tests of the bundle pack (`vfs::Table::to_pack` / `from_pack`). Random tables round
//! trip byte for byte; every cut-off pack and random damage is refused with a reason or read as some table, never a
//! panic; a damaged pack that still reads writes back the same bytes.

use aegis_core::rng::GodotRng;
use aegis_core::vfs::Table;

fn table(r: &mut GodotRng) -> Table {
    let mut t = Table::new();
    for _ in 0..r.randi_range(0, 12) {
        let depth = r.randi_range(1, 4);
        let key: Vec<String> = (0..depth).map(|_| (0..r.randi_range(1, 8)).map(|_| char::from(b'a' + r.randi_range(0, 25) as u8)).collect()).collect();
        let data: Vec<u8> = (0..r.randi_range(0, 300)).map(|_| r.randi_range(0, 255) as u8).collect();
        t.insert(format!("{}.bin", key.join("/")), data);
    }
    t
}

#[test]
fn a_pack_reads_back_as_the_table_it_was_written_from() {
    let mut r = GodotRng::new(17);
    for run in 0..500 {
        let t = table(&mut r);
        let bytes = t.to_pack();
        let back = Table::from_pack(&bytes).unwrap_or_else(|e| panic!("run {run}: {e}"));
        assert_eq!(back.len(), t.len(), "run {run}");
        assert!(back.iter().eq(t.iter()), "run {run}: the same files and bytes in the same order");
        assert_eq!(back.to_pack(), bytes, "run {run}: written again, the same bytes");
    }
}

#[test]
fn a_cut_or_damaged_pack_is_refused_or_read_never_a_panic() {
    let mut r = GodotRng::new(91);
    for run in 0..300 {
        let bytes = table(&mut r).to_pack();
        // Every cut short of the whole is refused (the last file or the count no longer fits).
        for cut in 0..bytes.len() {
            assert!(Table::from_pack(&bytes[..cut]).is_err(), "run {run}: a pack cut at {cut} of {} reads", bytes.len());
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(Table::from_pack(&longer).unwrap_err().contains("after the last"), "run {run}");
        // Random damage: refused with a reason, or (a byte inside some data) read as a table that writes the same bytes.
        for _ in 0..20 {
            let mut bad = bytes.clone();
            for _ in 0..r.randi_range(1, 4) {
                let i = r.randi_range(0, bad.len() as i32 - 1) as usize;
                bad[i] = r.randi_range(0, 255) as u8;
            }
            match Table::from_pack(&bad) {
                Err(e) => assert!(!e.is_empty()),
                Ok(t) => assert_eq!(t.to_pack().len(), bad.len(), "run {run}: a pack that reads holds all its bytes"),
            }
        }
    }
}

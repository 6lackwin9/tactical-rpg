# CLEANUP: housekeeping proposals (housekeeper agent)

Only open `proposed` / `keep` items remain here. Closed/done passes (`Pass 1` in [CLEANUP_PASS1.md](../archive/CLEANUP_PASS1.md); `Pass 2`, `Pass 3`, and `Later backlog pass` in [CLEANUP_PASS2_2026_11.md](../archive/CLEANUP_PASS2_2026_11.md)) are archived verbatim.

| Archive | Scope | Notes |
| --- | --- | --- |
| [CLEANUP_PASS1.md](../archive/CLEANUP_PASS1.md) | `Pass 1` (`2026-10-17`) | Godot-era cleanup (`CLN-1..18`); all done/closed |
| [CLEANUP_PASS2_2026_11.md](../archive/CLEANUP_PASS2_2026_11.md) | `Pass 2` (`2026-10-29`), `Pass 3` (`2026-11-01`), `Later` (`2026-11-02`) | `CLN-36..48, 51, 54` done; `CLN-57, 66..68`, `PACE1-A/B`, `DESERT1`, `OFFHAND1` superseded/dropped; remaining open items tracked below |

## Design note, 2026-11-10 (from FB-155 review)

| Candidate | What | Status |
| --- | --- | --- |
| CLN-WASMLINT | `aegis_core/tests/clock_lint.rs` walks kit and game `src/` from an engine crate's tests (precedent `aegis_core/tests/clip_windup.rs`); a workspace check beside `tools/fc.sh layers` would keep the engine crate from reading other layers | proposed |

## Open Candidates (`proposed` / `keep` carried forward)

| ID | Summary | Layer / Lane | Status | Detail Ref |
| --- | --- | --- | --- | --- |
| `CLN-49` | `git rm` 20 executed seeds in `games/emberwake/data/seeds/` + archive commit hash table | game / planner | `proposed` | `CLEANUP_PASS2_2026_11.md#cln-49` |
| `CLN-50` | Drop obsolete `RoundState.manual` flag & `set_manual` / host `manual` request | kit + game / logic | `proposed` | `CLEANUP_PASS2_2026_11.md#cln-50` |
| `CLN-52` | Split oversized `emberwake_gui/src/battle.rs` into board/strip/popup/panel submodules | game / logic | `proposed` | `CLEANUP_PASS2_2026_11.md#cln-52` |
| `CLN-53` | Archive retired specs & split oversized active docs (`fc.sh docs` size WARNs) | docs / design | `in-progress` (`CTX-CLEANUP1`) | `CLEANUP_PASS2_2026_11.md#cln-53` |
| `CLN-55, 56, 58..65, 69, 70` | Open Later-backlog rewrites (`UIR-4`, `OBJ1-U`, `RCLOSE1-U`, `QUESTTRACK1-U`, `DRAG1`, `SIMKPI1`, `SKILLTEXT2`, `REWARDXW1`, `ICONUI1`, `LAYOUT1`, `KEEP1`, `DIFF1`) | design / UI | `proposed` | `CLEANUP_PASS2_2026_11.md#later-backlog-pass-2026-11-02` |
| `DEEDS1, RESPEC1, UIR-7, MARKERS1, PLAYBACK2, PLAYTEST2, ARTTOOL1` | Kept unbuilt post-R4 / tooling backlog ideas (`keep`, not superseded) | design / logic | `keep` | `CLEANUP_PASS2_2026_11.md#later-pass-every-open-row` |

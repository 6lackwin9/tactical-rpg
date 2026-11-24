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

## Pass 4: this week's engine additions (ENGINE-65 TIDY3, 2026-11-24)

Scope: the feel probe and its reports (`feel`, `walkfeel`, `switchfeel`, `audiofeel`, `perf`), `reach`, `crashlog`,
`live`, the gamepad tuning and the headless `feel` request (ENGINE-45 to ENGINE-64). Read-only; proposals only. No
dead module found: every report module has a caller in `shell/headless.rs` and `aegis_cli::scenario`. The stall rule
is already shared (`audio_out::mix_stalled` reads `feel::STALL_CPU_SHARE`). Every item is `proposed`; none changes a
golden. A report's numbers stay the same unless the item says otherwise.

| ID | Summary | Layer / Lane | Status | Detail |
| --- | --- | --- | --- | --- |
| `CLN-72` | One "build" name: `if cfg!(debug_assertions) { "dev" } else { "release" }` is written 4 times (`crashlog.rs`, `shell/headless.rs` twice, `bin/ui_bench.rs`); add `perf::build_name()` and use it | engine / tools | `proposed` | Pure move, same strings |
| `CLN-73` | One ms rounding helper: `feel.rs` (3 closures), `perf.rs`, `shell/headless.rs` round to 0.001 with inline closures; `walkfeel.rs` and `switchfeel.rs` each have their own `r2` (0.01); `audiofeel.rs` has `r3`. Add `perf::round_to(x, places)` and keep each report's places | engine / tools | `proposed` | Same numbers when each keeps its places |
| `CLN-74` | Two medians: `walkfeel::median` takes the upper middle sample, `perf::Stats::of` the nearest-rank p50. They agree on odd counts and differ by one sample on even ones. Either name the difference in `walkfeel`, or move `walkfeel` to `Stats` (its `cadence_ms.median` can move by one gap on an even count, so `walk_feel` budgets would need a re-run) | engine / tools | `proposed` (Design call: name or merge) | Merging moves report numbers |
| `CLN-75` | Split the headless `feel` request: `Loop::feel` is about 220 lines; the `press`/`pad_press` and `stick` branches repeat the same mark, input, key and draw sequence. Pull out `hold_step(kind, what, key, down)` and a `script_step` per input kind | engine / tools | `proposed` | Behaviour unchanged; `walk_feel` and headless tests guard it |
| `CLN-76` | Duplicate path constants in `live.rs` (ENGINE-64): `THEME_FILE` repeats `theme::FILE`, and `LAYOUTS_DIR` ("ui/layouts/") repeats `screen_layout::LAYOUTS_DIR` ("ui/layouts"). Use the owners' constants | engine / tools | `proposed` | Trivial; same paths |
| `CLN-77` | Launch keys read in three places: `feel::launch_asked` reads `std::env::args` itself (`--feel`, `feel=1`), outside the parsed launch keys `start::Start` gets; `dev=1` is parsed by `Start` (`Options::live`) and again by Emberwake (`dev_asked`, `DEV_ASKED`). Have `Start` parse `dev` and `feel` once (`Start::dev`, `Options::feel`), with games reading them from there. Also list the engine's launch keys (`port`, `saves`, `screen`, `dev`, `live`, `feel`, `headless`, `fps`, `dpr`, `clock`) in one table in the engine README | engine + game / tools | `proposed` | Same keys and meanings |
| `CLN-78` | The feel probe has two switches: the launch key (`feel_launch`) and the settings key `"feel": true`. The headless `feel` request flips `feel_launch` and puts it back. Keep both (a playtester's settings vs a run's key), but name the settings key in `feel.rs`'s header as the player's and the launch key as the run's, or drop the settings key if nothing sets it (no Settings row does today) | engine / tools | `proposed` (Design call: keep or drop) | Dropping changes a dev setting only |
| `CLN-79` | Test hooks in a shipped binary: the headless `feel` script's `stall_ms` and `spike_ms` move the shot clock, and the link's `panic` answers "dev build only" in release. Both are safe (the shot clock exists only in a scenario run, and `panic` is refused in release), so keep them, but group them under one "test hooks" note in `engine/docs/authoring/scenarios.md` and `crash_safety.md` so they are not mistaken for features | engine / docs | `proposed` (`keep` the code) | Docs only |
| `CLN-80` | `controls::stick_dir` (the default dead zone) is now used only by tests; the shipped paths call `PadTuning::stick_dir`. Keep it as the documented default helper, or move its test to `PadTuning::default()` and drop it | engine / tools | `proposed` | Public API change (tiny) |
| `CLN-81` | `Audio::forget` (ENGINE-64) and `Audio::remounted` both clear the same caches (`cache`, `pcm`, `found`, `pools`) by different rules; one private `drop_cached(filter)` would keep them in step when a cache is added | engine / tools | `proposed` | Behaviour unchanged |

# Repro recordings (regression replays)

Every playtest recording here (`<id>.json`, or `<id>/<id>.json` with its note images) is a regression test:
`tools/fc.sh gate` replays each one (`tools/fc.sh repro`, REPLAY-CI1) and fails when a session's checkpoint diverges, a
recorded request is refused or the replay panics. An empty folder passes.

- **In**: Build moves a user recording here from `../inbox/` in the commit that lands its fix row (`docs/workflow/cloud.md`,
  "Playtest intake" step 4), or records one headlessly from a list of requests:
  `aegis record <requests.json> out=games/emberwake/playtests/repro id=<id>` (a list of requests is one session, a list
  of lists one session each).
- **Run one**: `tools/fc.sh replay games/emberwake/playtests/repro/<id>.json`; all of them: `tools/fc.sh repro`.
- **An intended change** (a rule or data change that moves the checkpoints): record it again on the current data,
  `aegis record games/emberwake/playtests/repro/<id>.json out=games/emberwake/playtests/repro id=<id>`, and name it in
  the commit. A divergence nobody intended is a bug to fix, not a recording to replace.

| Recording | Sessions |
| --- | --- |
| `agent-classic-and-timeline-autoplay` | 0: `goblin_ambush` seed 3, classic, six rounds of `preset balanced`, `resolve`, `advance`; 1: `goblin_skirmish` seed 7, timeline ruleset, `autoplay balanced` |

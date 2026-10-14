# Decisions (settled; do not re-litigate without asking the user)

| Decision | Choice | Why |
|---|---|---|
| Class balance | Equal stat budget (62) and growth (5/level) enforced by the catalog | Classes differ only by distribution, so balance is comparable |
| Unit variance | Authored spend pool (4 + level-1), exact | Same-class units differ without breaking budget |
| Mastery | Removed. Gear = class Proficiency dict. Traits = Passives granted by sources | Mastery was coupled to units; lore should grant traits |
| Passives | Generic lore-free templates in base catalog; sources reference them | Worlds stay modular |
| Enemy scaling | One base curve (class + tier + AI). Worlds set level and tuning deltas; "goblin" is lore | Same class behaves the same across worlds |
| Authored `power` | Absolute (tiers do not scale it) | Predictable for fixed/special units |
| Content edits | Through `ContentCatalog.upsert` in scripts, validated | Hand-edited JSON and ad-hoc string patches broke files before |
| Commits | Only when `check_project.ps1` is clean; trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` | User standing rule |
| Typing | Strict: no UNSAFE_*, untyped loop vars, unused params | User standing rule |
| README | Updated with every change | User standing rule |
| Uncertainty | Check known-issues table; max 2 failed fixes; then ask the user with findings and options. Always ask on design/gameplay choices | User request |

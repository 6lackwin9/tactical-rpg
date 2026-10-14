# Decisions (settled; do not re-litigate without asking the user)

| Decision | Choice | Why |
| --- | --- | --- |
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
| Combat extensions (S12) | Additive: influence stat inputs to the formula or wrap the base damage formula; `DamageCalculator` core stays the single formula, S1 keeps `balance-ready` | User 2026-10-15: advanced options isolated from the formula |
| Level-up spend | Automatic in test/sim (try variants to see if choices matter); player-allocated in game. Balance target: performance stays in range whatever the player picks; choices are flavour/variance with minor benefits that discourage min-maxing | User 2026-10-15 |
| Gear budget breaking | Class default loadouts follow a budget; reward/special items may break it only to offer a new playstyle or compensate a con not factored in base-class tuning | User 2026-10-15 (details of the budget rule still open) |
| Exploration encounters | Step counter for regular mobs, rare elite chance; elites, bosses and scripted fights are placed/roaming | User 2026-10-15 |
| Support identity | Add a heal/support identity if no class has it, within the six-class limit (class choice pending skill-coverage review) | User 2026-10-15 |
| Solo difficulty | Lone hero vs mob groups: losses fine when under-levelled/under-geared, improves with level; bosses hard: ~10-20% solo win at equal level, party encouraged | User 2026-10-15 |
| Passives lifetime | Permanent by default; scripted fights/events may grant temporary ones for that fight/event | User 2026-10-15 |
| Playable scope | One small campaign first; more campaigns later; unify stories across worlds after several exist; UGC story mode is a later engine goal | User 2026-10-15 |
| Build order | Systems first (S7/S13, S12-A with S4-A and S3, S8...), balance pass only after the balance-relevant rules are frozen (supersedes M1-first) | User 2026-10-15 |

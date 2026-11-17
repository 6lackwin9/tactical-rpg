# TRANSITIONS1: story to battle and battle to hub (draft)

Status: **active** (user 2026-11-17: "all recs T and lore"; the ★ options; READY `PR7-14`, Art/UI `AI-18`) (proposed 2026-11-17 after the web-build playtest note "the progression
from story vignette to battle seems abrupt and then from battle to hub"). Layer: game UI (`emberwake_gui`) with engine
fades already present; art and audio through an Art/UI request once approved. Recommended options are marked ★.

## T1. Story vignette into battle

★ A two-second "battle beat":

1. The vignette's last line fades out while the art holds.
2. An ash wipe darkens the screen into the battle stage; the music crossfades through a short combat sting.
3. A banner shows the fight's name, its objective and the enemy count; the party walks in from the edge to its start
   cells.
4. Control starts when the banner leaves.

A click skips to step 4. With Reduce motion on, it is a plain `0.3` s fade.

Alternative: a briefing screen before each fight (map preview, objective, party swap). More control, but a whole extra
screen and slower every time.

## T2. Battle back to the hub

★ A "victory beat" and an arrival card:

1. The last hit lands in brief slow motion, a Victory banner appears, and the units drop to idle.
2. The results panel slides over the dimmed battlefield: XP, loot, roster changes, and weapon wear and mastery
   (`PR7-3`).
3. Continue fades to black, with a victory stinger into the hub music.
4. The hub opens on the party with a one-line card ("Back at the Sluice Undercroft · Day 12 · Tithing 41 days"); the
   log ticker lists what changed (a new talk, new shop stock).

If the story continues right after the fight, step 3 goes to the next vignette instead of the hub.

Alternative: no arrival card, only the fade.

## On approval

Design queues one UI row for Build and an Art/UI request for the ash wipe, the two banners and the stingers.

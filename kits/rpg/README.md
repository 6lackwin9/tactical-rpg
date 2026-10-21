# RPG kit

Reusable RPG assets shared by games built on Aegis (ASSETORG2, `docs/specs/ASSETORG2.md`; decision "Asset layers and
reuse groups (user 2026-10-20)"). A game uses the kit by naming it in its `game.json` (`"kit": "rpg"`); the engine
then looks up every asset in the game's `assets/`, this kit's `assets/` and `engine/assets/`, in that order, so a game
overrides a kit file by putting its own at the same path.

The kit holds files and a manifest only, no code: the lookup chains that pick a body, portrait or icon live in the
game until a second game needs them (SPLIT1 principle 2).

`kit.json` is written through `aegis content manifest --kit rpg <json>` (id = this folder's name, a semver version,
the engine range it needs).

What is here (`assets/`):

- `units/<class>/<variant>/sheet.png` + `clips.json`: the six class bodies (`base`), the goblin variants
  (`brawler/goblin`, `rogue/goblin`, `ranger/goblin`, `priest/goblin`, `brawler/goblin_berserker`,
  `rogue/goblin_scout`) and the training dummy.
- `portraits/common/<class>_<variant>.png`: the portraits mobs and minor characters share.
- `items/<family>/base.png` and generic variants (`sword/greatsword`, `accessory/amulet`, `consumable/bomb`, ...);
  `items/any/`.
- `skills/kinds/<kind>.png`, `statuses/<id>.png`, `map/markers/<marker>.png`, `ui/{pins,intents,badges,chips}/`.
- `audio/sfx/` (combat sounds) and `audio/music/` (mood music: combat, boss, town, shop, danger, travel).

Placeholder and baseline art (ART1-A generators, the sprite rig, ART2) comes from the repository's `tools/`; the
tools write the layered tree from ASSETORG2-T on.

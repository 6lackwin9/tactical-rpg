"""Native SpriteForge character specification, Emberwake canon palettes,
offline regex trait interpreter, and chroma-key review strip composer.

Zero external AI/API calls: strictly deterministic data structures, color
ramps fitting rig.SLOTS_MAX = 16, regex trait extraction, and PIL composition.
"""
from __future__ import annotations

from dataclasses import dataclass, field, replace
import json
import os
import re
from typing import Iterable, Mapping, Sequence

from PIL import Image

import rig

EMBERWAKE_OUTLINE_RGB: tuple[int, int, int] = (20, 17, 15)
EMBERWAKE_OUTLINE_HEX: str = "#14110F"
assert EMBERWAKE_OUTLINE_RGB == rig.OUT, "Canon outline must stay locked to rig.OUT"

SLOTS_MAX: int = rig.SLOTS_MAX  # 16

SPRITE_STYLES: tuple[str, ...] = (
    "pixel-16",
    "cel-ink",
    "vector-flat",
    "dark-painted",
    "chibi-arcade",
)
SPRITE_POSES: tuple[str, ...] = (
    "idle",
    "walk",
    "jump",
    "attack",
    "cast",
    "guard",
    "hit",
    "victory",
)
POSE_SHEET_ORDER: tuple[str, ...] = SPRITE_POSES
SPRITE_PERSPECTIVES: tuple[str, ...] = (
    "side-right",
    "side-left",
    "front-down",
    "back-up",
    "three-quarter",
    "top-down",
)
RIG_BODY_TYPES: tuple[str, ...] = ("humanoid", "quadruped", "crawler")
TAINT_TIER_AXES: tuple[str, ...] = ("clean", "tainted")
CHARACTER_ARCHETYPES: tuple[str, ...] = (
    "knight",
    "rogue",
    "mage",
    "ranger",
    "cyber",
    "samurai",
    "brawler",
    "creature",
)
BUILD_AXES: tuple[str, ...] = ("lithe", "balanced", "heavy", "compact")
HEADGEAR_AXES: tuple[str, ...] = (
    "none",
    "knight-helm",
    "kettle-helm",
    "hood",
    "wizard-hat",
    "visor",
    "goggles",
    "spectacles",
    "iron-mask",
    "mitre",
    "horns",
    "crown",
    "kabuto",
    "cat-ears",
)
HAIR_STYLE_AXES: tuple[str, ...] = (
    "short",
    "spiky",
    "long-flowing",
    "ponytail",
    "bob",
    "bald",
)
EYE_STYLE_AXES: tuple[str, ...] = (
    "determined",
    "glowing",
    "visor-slit",
    "wide-expressive",
    "cyclops",
)
OUTFIT_AXES: tuple[str, ...] = (
    "plate-armor",
    "trench-coat",
    "arcane-robes",
    "stealth-garb",
    "mech-suit",
    "samurai-haori",
    "tunic",
)
BACK_ACCESSORY_AXES: tuple[str, ...] = (
    "none",
    "tattered-cape",
    "royal-cloak",
    "jetpack",
    "wings",
    "scarf",
    "quiver",
)
WEAPON_AXES: tuple[str, ...] = (
    "none",
    "greatsword",
    "billhook",
    "katana",
    "twin-daggers",
    "bone-saw",
    "arcane-staff",
    "censer",
    "energy-rifle",
    "sprayer",
    "warhammer",
    "bow",
    "arbalest",
    "shield-sword",
    "buckler-axe",
    "cestus",
    "scourge",
)
VFX_AURA_AXES: tuple[str, ...] = (
    "none",
    "embers",
    "arcane-runes",
    "lightning",
    "shadow-wisps",
    "frost",
)
MOTIF_AXES: tuple[str, ...] = (
    "none",
    "sun",
    "crescent",
    "diamond",
    "skull",
    "bolt",
)
CHROMA_KEY_MODES: tuple[str, ...] = ("auto", "green", "magenta", "blue")
CHROMA_KEY_SPECS: dict[str, tuple[str, tuple[int, int, int]]] = {
    "green": ("#00FF00", (0, 255, 0)),
    "magenta": ("#FF00FF", (255, 0, 255)),
    "blue": ("#0047FF", (0, 71, 255)),
}


def parse_hex_rgb(value: str | tuple[int, int, int]) -> tuple[int, int, int]:
    """Parse '#RRGGBB' / 'RRGGBB' or an RGB 3-tuple into clamped ints."""
    if isinstance(value, (tuple, list)):
        if len(value) != 3:
            raise ValueError(f"Expected 3-channel RGB tuple, got {value!r}")
        return tuple(max(0, min(255, int(c))) for c in value)  # type: ignore[return-value]
    s = str(value).strip().lstrip("#")
    if len(s) != 6:
        raise ValueError(f"Invalid hex color: {value!r}")
    return (int(s[0:2], 16), int(s[2:4], 16), int(s[4:6], 16))


def rgb_to_hex(rgb: tuple[int, int, int], *, upper: bool = False) -> str:
    """Format an RGB tuple as '#rrggbb' (or uppercase '#RRGGBB')."""
    r, g, b = parse_hex_rgb(rgb)
    fmt = f"#{r:02x}{g:02x}{b:02x}"
    return fmt.upper() if upper else fmt


def shade_rgb(rgb: str | tuple[int, int, int], amount: float) -> tuple[int, int, int]:
    """Port of SpriteForge `shadeHex`: lighten toward 255 (amount >= 0) or darken toward 0 (amount < 0)."""
    r, g, b = parse_hex_rgb(rgb)
    if amount >= 0:
        return (
            max(0, min(255, round(r + (255 - r) * amount))),
            max(0, min(255, round(g + (255 - g) * amount))),
            max(0, min(255, round(b + (255 - b) * amount))),
        )
    factor = 1.0 + amount
    return (
        max(0, min(255, round(r * factor))),
        max(0, min(255, round(g * factor))),
        max(0, min(255, round(b * factor))),
    )


def shade_ramp(
    rgb: str | tuple[int, int, int],
    *,
    tones: int = 3,
    lo_amount: float = -0.35,
    hi_amount: float = 0.25,
) -> tuple[tuple[int, int, int], ...]:
    """Build a 2-tone (lo, base) or 3-tone (lo, base, hi) color ramp from `rgb`."""
    base = parse_hex_rgb(rgb)
    lo = shade_rgb(base, lo_amount)
    if tones == 2:
        return (lo, base)
    if tones == 3:
        hi = shade_rgb(base, hi_amount)
        return (lo, base, hi)
    raise ValueError(f"Unsupported ramp tone count {tones!r}; expected 2 or 3")


@dataclass(frozen=True)
class MasterHue:
    index: int
    name: str
    hex: str
    role: str

    @property
    def rgb(self) -> tuple[int, int, int]:
        return parse_hex_rgb(self.hex)


EMBERWAKE_MASTER_HUES: tuple[MasterHue, ...] = (
    MasterHue(1, "Void Abyss", "#0A0B10", "Blighted shadow & crucible void"),
    MasterHue(2, "Damp Basalt", "#14171F", "Masonry shadow, wet iron & Inquisition cloth"),
    MasterHue(3, "Wet Slate", "#1E232D", "Midtone cobble & slate trim"),
    MasterHue(4, "Rotting Oak", "#2C221E", "Timbers, dried grime & heavy leather"),
    MasterHue(5, "Scavenged Iron", "#4B5563", "Worn steel, chain & dull blade"),
    MasterHue(6, "Polished Steel", "#9CA3AF", "Specular edge, silver trim & highlights"),
    MasterHue(7, "Bleached Linen", "#D9CBB4", "Bone, scroll parchment, Clerisy & ash-hair"),
    MasterHue(8, "Cinder Ash", "#6B7280", "Neutral ash stone & muted wraps"),
    MasterHue(9, "Boiled Leather", "#92400E", "Pauldrons, straps, belts & gloves"),
    MasterHue(10, "Tallow Amber", "#D97706", "Rendered tallow, kiln fire & lantern light"),
    MasterHue(11, "Candle Gold", "#FBBF24", "Solar trim, gilt insignia & spark core"),
    MasterHue(12, "Arterial Red", "#E11D48", "Fresh blood, scarf ribbons & vitriol flare"),
    MasterHue(13, "Rust Crimson", "#991B1B", "Castellan crest, dried gore & wax seal"),
    MasterHue(14, "Blight Violet", "#9333EA", "Inquisition taint & marrow infection"),
    MasterHue(15, "Resonant Orchid", "#C084FC", "Awakening Chorus bloom & astral sparks"),
    MasterHue(16, "Salt Cyan", "#06B6D4", "Arcane ward, phosphor fluid & cold brine"),
)
assert len(EMBERWAKE_MASTER_HUES) == 16

EMBERWAKE_SKIN_RAMP: dict[str, str] = {
    "sunk_ward_deep": "#6B4A3A",
    "sunk_ward_base": "#A47A5F",
    "sunk_ward_pale": "#D4A988",
}


@dataclass(frozen=True)
class CharacterPalette:
    primary: str
    secondary: str
    accent: str
    skin: str
    hair: str
    outline: str = EMBERWAKE_OUTLINE_HEX

    def __post_init__(self) -> None:
        # Validate every hex channel at construction time.
        for attr in ("primary", "secondary", "accent", "skin", "hair", "outline"):
            object.__setattr__(self, attr, rgb_to_hex(parse_hex_rgb(getattr(self, attr)), upper=True))

    @property
    def rgb_list(self) -> tuple[tuple[int, int, int], ...]:
        return tuple(
            parse_hex_rgb(c)
            for c in (self.primary, self.secondary, self.accent, self.skin, self.hair, self.outline)
        )


@dataclass(frozen=True)
class FactionSwatch:
    id: str
    name: str
    class_hint: str
    palette: CharacterPalette


EMBERWAKE_FACTION_SWATCHES: dict[str, FactionSwatch] = {
    "severed_root": FactionSwatch(
        id="severed_root",
        name="Severed Root",
        class_hint="warrior",
        palette=CharacterPalette("#4B5563", "#991B1B", "#D97706", "#A47A5F", "#D9CBB4"),
    ),
    "castellan_watch": FactionSwatch(
        id="castellan_watch",
        name="Castellan Watch",
        class_hint="warrior",
        palette=CharacterPalette("#4B5563", "#991B1B", "#FBBF24", "#A47A5F", "#2C221E"),
    ),
    "iron_inquisition": FactionSwatch(
        id="iron_inquisition",
        name="Iron Inquisition",
        class_hint="warrior",
        palette=CharacterPalette("#14171F", "#4B5563", "#9333EA", "#6B7280", "#D9CBB4"),
    ),
    "sovereign_clerisy": FactionSwatch(
        id="sovereign_clerisy",
        name="Sovereign Clerisy",
        class_hint="priest",
        palette=CharacterPalette("#D9CBB4", "#991B1B", "#FBBF24", "#D4A988", "#D9CBB4"),
    ),
    "tallow_guild": FactionSwatch(
        id="tallow_guild",
        name="Tallow Guild",
        class_hint="brawler",
        palette=CharacterPalette("#92400E", "#2C221E", "#D97706", "#6B4A3A", "#14171F"),
    ),
    "awakening_chorus": FactionSwatch(
        id="awakening_chorus",
        name="Awakening Chorus",
        class_hint="mage",
        palette=CharacterPalette("#6B7280", "#0A0B10", "#C084FC", "#D9CBB4", "#0A0B10"),
    ),
}
assert len(EMBERWAKE_FACTION_SWATCHES) == 6


@dataclass(frozen=True)
class FounderTaintOverlay:
    accent: str
    vfx_aura: str
    tainted_suffix: str = "(Tainted)"
    eyes: str | None = None

    def __post_init__(self) -> None:
        object.__setattr__(self, "accent", rgb_to_hex(parse_hex_rgb(self.accent), upper=True))
        if self.vfx_aura not in VFX_AURA_AXES:
            raise ValueError(f"Invalid vfx_aura {self.vfx_aura!r}")
        if self.eyes is not None and self.eyes not in EYE_STYLE_AXES:
            raise ValueError(f"Invalid eyes {self.eyes!r}")

    def apply(self, spec: "CharacterSpec") -> "CharacterSpec":
        """Return a tainted copy of `spec` with this overlay applied."""
        new_pal = replace(spec.palette, accent=self.accent)
        suffix = f" {self.tainted_suffix}".strip()
        new_name = spec.name if spec.name.endswith(self.tainted_suffix) else f"{spec.name} {suffix}"
        return replace(
            spec,
            name=new_name,
            taint_tier="tainted",
            vfx_aura=self.vfx_aura,
            eyes=self.eyes or spec.eyes,
            palette=new_pal,
        )


def _check_axis(name: str, value: str, allowed: tuple[str, ...]) -> None:
    if value not in allowed:
        raise ValueError(f"Invalid {name}={value!r}; expected one of {allowed}")


@dataclass(frozen=True)
class CharacterSpec:
    id: str
    name: str
    archetype: str = "knight"
    build: str = "balanced"
    headgear: str = "none"
    hair_style: str = "short"
    eyes: str = "determined"
    outfit: str = "plate-armor"
    back_accessory: str = "none"
    weapon: str = "greatsword"
    vfx_aura: str = "none"
    motif: str = "none"
    taint_tier: str = "clean"
    taint_overlay: FounderTaintOverlay | None = None
    rig_body: str = "humanoid"
    style: str = "pixel-16"
    perspective: str = "side-right"
    chroma_key_mode: str = "auto"
    description: str = ""
    theme: str = "Emberwake (Low-Fantasy Tallow & Ash)"
    palette: CharacterPalette = field(
        default_factory=lambda: EMBERWAKE_FACTION_SWATCHES["severed_root"].palette
    )
    recommended_style: str = "pixel-16"
    art_notes: str = ""
    class_id: str = "warrior"
    look_key: str = "warrior"

    def __post_init__(self) -> None:
        _check_axis("archetype", self.archetype, CHARACTER_ARCHETYPES)
        _check_axis("build", self.build, BUILD_AXES)
        _check_axis("headgear", self.headgear, HEADGEAR_AXES)
        _check_axis("hair_style", self.hair_style, HAIR_STYLE_AXES)
        _check_axis("eyes", self.eyes, EYE_STYLE_AXES)
        _check_axis("outfit", self.outfit, OUTFIT_AXES)
        _check_axis("back_accessory", self.back_accessory, BACK_ACCESSORY_AXES)
        _check_axis("weapon", self.weapon, WEAPON_AXES)
        _check_axis("vfx_aura", self.vfx_aura, VFX_AURA_AXES)
        _check_axis("motif", self.motif, MOTIF_AXES)
        _check_axis("taint_tier", self.taint_tier, TAINT_TIER_AXES)
        _check_axis("rig_body", self.rig_body, RIG_BODY_TYPES)
        _check_axis("style", self.style, SPRITE_STYLES)
        _check_axis("perspective", self.perspective, SPRITE_PERSPECTIVES)
        _check_axis("chroma_key_mode", self.chroma_key_mode, CHROMA_KEY_MODES)

    def with_taint(self, tier: str = "tainted", overlay: FounderTaintOverlay | None = None) -> "CharacterSpec":
        """Return a clean or tainted variant of this CharacterSpec."""
        _check_axis("taint_tier", tier, TAINT_TIER_AXES)
        if tier == "clean":
            base_name = re.sub(r"\s*\(Tainted\)\s*$", "", self.name).strip()
            return replace(self, name=base_name, taint_tier="clean")
        chosen = overlay or self.taint_overlay
        if chosen is None:
            return replace(self, taint_tier="tainted")
        return chosen.apply(self)


FOUNDER_SPECS: dict[str, CharacterSpec] = {
    "kaelen": CharacterSpec(
        id="founder_kaelen",
        name="Kaelen",
        archetype="knight",
        build="heavy",
        headgear="none",
        hair_style="short",
        eyes="determined",
        outfit="plate-armor",
        back_accessory="tattered-cape",
        weapon="billhook",
        vfx_aura="none",
        motif="none",
        taint_tier="clean",
        taint_overlay=FounderTaintOverlay(accent="#D9CBB4", vfx_aura="embers", tainted_suffix="(Tainted)"),
        palette=CharacterPalette("#4B5563", "#991B1B", "#D9CBB4", "#A47A5F", "#D4B483"),
        description="Scarred ex-warden in iron pauldrons carrying a notched billhook.",
        class_id="warrior",
        look_key="warrior/2h_polearm",
    ),
    "vesper": CharacterSpec(
        id="founder_vesper",
        name="Vesper",
        archetype="rogue",
        build="lithe",
        headgear="hood",
        hair_style="short",
        eyes="determined",
        outfit="stealth-garb",
        back_accessory="scarf",
        weapon="twin-daggers",
        vfx_aura="none",
        motif="none",
        taint_tier="clean",
        taint_overlay=FounderTaintOverlay(accent="#C084FC", vfx_aura="shadow-wisps", tainted_suffix="(Tainted)"),
        palette=CharacterPalette("#6B7280", "#2C221E", "#E11D48", "#6B4A3A", "#14171F"),
        description="Hollow-eyed gutter operative in cinder cloak wielding twin curfew bodkins.",
        class_id="rogue",
        look_key="rogue",
    ),
    "maren": CharacterSpec(
        id="founder_maren",
        name="Maren",
        archetype="mage",
        build="balanced",
        headgear="hood",
        hair_style="short",
        eyes="wide-expressive",
        outfit="arcane-robes",
        back_accessory="royal-cloak",
        weapon="censer",
        vfx_aura="none",
        motif="sun",
        taint_tier="clean",
        taint_overlay=FounderTaintOverlay(
            accent="#9333EA", vfx_aura="arcane-runes", tainted_suffix="(Tainted)", eyes="glowing"
        ),
        palette=CharacterPalette("#14171F", "#D9CBB4", "#D97706", "#D4A988", "#991B1B"),
        description="Reluctant Clerisy priestess in obsidian stole swinging a burning bronze censer.",
        class_id="priest",
        look_key="priest",
    ),
    "hollis": CharacterSpec(
        id="founder_hollis",
        name="Hollis",
        archetype="ranger",
        build="lithe",
        headgear="spectacles",
        hair_style="short",
        eyes="determined",
        outfit="trench-coat",
        back_accessory="quiver",
        weapon="arbalest",
        vfx_aura="none",
        motif="none",
        taint_tier="clean",
        taint_overlay=FounderTaintOverlay(
            accent="#D97706", vfx_aura="embers", tainted_suffix="(Tainted)", eyes="glowing"
        ),
        palette=CharacterPalette("#92400E", "#5F2A09", "#4B5563", "#A47A5F", "#2C221E"),
        description="One-eyed watch surveyor wearing wire spectacles with winch arbalest and hip quiver.",
        class_id="ranger",
        look_key="ranger",
    ),
    "dunstan": CharacterSpec(
        id="founder_dunstan",
        name="Dunstan",
        archetype="brawler",
        build="heavy",
        headgear="none",
        hair_style="bald",
        eyes="determined",
        outfit="plate-armor",
        back_accessory="none",
        weapon="cestus",
        vfx_aura="none",
        motif="none",
        taint_tier="clean",
        taint_overlay=FounderTaintOverlay(accent="#0A0B10", vfx_aura="embers", tainted_suffix="(Tainted)"),
        palette=CharacterPalette("#92400E", "#464A53", "#4B5563", "#A47A5F", "#14171F"),
        description="Bull-necked penal foundry stoker fighting with riveted iron penal shackles.",
        class_id="brawler",
        look_key="brawler/gauntlet",
    ),
    "elyse": CharacterSpec(
        id="founder_elyse",
        name="Elyse",
        archetype="cyber",
        build="lithe",
        headgear="goggles",
        hair_style="short",
        eyes="determined",
        outfit="trench-coat",
        back_accessory="jetpack",
        weapon="sprayer",
        vfx_aura="none",
        motif="none",
        taint_tier="clean",
        taint_overlay=FounderTaintOverlay(accent="#C084FC", vfx_aura="arcane-runes", tainted_suffix="(Tainted)"),
        palette=CharacterPalette("#2C221E", "#92400E", "#D97706", "#6B4A3A", "#14171F"),
        description="Bitter alchemist in vat coat and goggles carrying a brass vitriol sprayer and orb.",
        class_id="mage",
        look_key="mage/1h_sprayer",
    ),
}
assert len(FOUNDER_SPECS) == 6

# Slot schemas matching shipped base `clips.json` slot counts (all <= rig.SLOTS_MAX = 16).
CLASS_RIG_SLOT_ORDER: dict[str, tuple[str, ...]] = {
    "warrior": (
        "main",
        "leg",
        "sleeve",
        "fore",
        "trim",
        "accent",
        "emblem",
        "metal",
        "leather",
        "boot",
        "skin",
        "shield",
        "glove",
    ),
    "brawler": (
        "main",
        "leg",
        "sleeve",
        "fore",
        "accent",
        "glove",
        "boot",
        "skin",
        "hair",
        "eye",
    ),
    "ranger": (
        "main",
        "leg",
        "sleeve",
        "accent",
        "leather",
        "wood",
        "boot",
        "skin",
        "hair",
        "glove",
        "eye",
    ),
    "rogue": (
        "main",
        "leg",
        "sleeve",
        "trim",
        "accent",
        "leather",
        "boot",
        "skin",
        "glove",
        "eye",
    ),
    "priest": (
        "main",
        "leg",
        "sleeve",
        "fore",
        "trim",
        "metal",
        "wood",
        "boot",
        "skin",
        "glove",
        "eye",
    ),
    "mage": (
        "main",
        "leg",
        "sleeve",
        "trim",
        "magic",
        "wood",
        "boot",
        "skin",
        "hair",
        "glove",
        "eye",
    ),
}


def build_rig_palette(
    palette: CharacterPalette,
    *,
    class_id: str = "warrior",
    slot_names: Sequence[str] | None = None,
    tones: int = 3,
    include_ramps: bool = False,
) -> tuple[dict[str, str], dict[str, tuple[tuple[int, int, int], ...]]]:
    """Map a 6-color SpriteForge CharacterPalette into <= rig.SLOTS_MAX base slots
    and optional 2/3-tone RGB ramps per slot."""
    if tones not in (2, 3):
        raise ValueError(f"tones must be 2 or 3, got {tones!r}")
    chosen_slots = tuple(slot_names or CLASS_RIG_SLOT_ORDER.get(class_id, CLASS_RIG_SLOT_ORDER["warrior"]))
    if len(chosen_slots) > SLOTS_MAX:
        raise ValueError(f"Palette has {len(chosen_slots)} slots, exceeding rig.SLOTS_MAX={SLOTS_MAX}")

    pri = parse_hex_rgb(palette.primary)
    sec = parse_hex_rgb(palette.secondary)
    acc = parse_hex_rgb(palette.accent)
    skn = parse_hex_rgb(palette.skin)
    hr = parse_hex_rgb(palette.hair)

    mapping: dict[str, tuple[int, int, int]] = {
        "main": pri,
        "sleeve": pri,
        "fore": pri,
        "leg": shade_rgb(sec, -0.25),
        "trim": acc,
        "accent": acc,
        "emblem": acc,
        "magic": acc,
        "eye": acc,
        "metal": parse_hex_rgb("#4B5563") if pri != parse_hex_rgb("#4B5563") else parse_hex_rgb("#9CA3AF"),
        "shield": sec,
        "leather": shade_rgb(sec, -0.15),
        "wood": parse_hex_rgb("#2C221E"),
        "glove": shade_rgb(sec, -0.20),
        "boot": shade_rgb(pri, -0.45),
        "skin": skn,
        "hair": hr,
    }

    slots: dict[str, str] = {}
    ramps: dict[str, tuple[tuple[int, int, int], ...]] = {}
    for slot in chosen_slots:
        rgb = mapping.get(slot, pri)
        slots[slot] = rgb_to_hex(rgb)
        if include_ramps:
            ramps[slot] = shade_ramp(rgb, tones=tones)
    if not include_ramps:
        for slot in chosen_slots:
            ramps[slot] = shade_ramp(slots[slot], tones=tones)
    assert len(slots) <= SLOTS_MAX
    return slots, ramps


def export_palette_slots(
    source: CharacterSpec | CharacterPalette | FactionSwatch,
    *,
    class_id: str | None = None,
    slot_names: Sequence[str] | None = None,
    tones: int = 3,
) -> dict[str, dict[str, str]]:
    """Build an Emberwake `{\"slots\": {slot: \"#rrggbb\"}}` palette dictionary <= rig.SLOTS_MAX."""
    if isinstance(source, CharacterSpec):
        pal = source.palette
        cid = class_id or source.class_id
    elif isinstance(source, FactionSwatch):
        pal = source.palette
        cid = class_id or source.class_hint
    else:
        pal = source
        cid = class_id or "warrior"
    slots, _ = build_rig_palette(pal, class_id=cid, slot_names=slot_names, tones=tones)
    assert len(slots) <= SLOTS_MAX
    return {"slots": slots}


def export_founder_palettes(
    *,
    tier: str = "both",
    out_dir: str | None = None,
    tones: int = 3,
) -> dict[str, dict[str, dict[str, str]]]:
    """Build (and optionally write JSON files for) clean and/or tainted Founder palettes."""
    if tier not in ("clean", "tainted", "both"):
        raise ValueError(f"Invalid tier={tier!r}; expected clean, tainted, or both")
    results: dict[str, dict[str, dict[str, str]]] = {}
    for key, clean_spec in FOUNDER_SPECS.items():
        if tier in ("clean", "both"):
            results[key] = export_palette_slots(clean_spec, tones=tones)
        if tier in ("tainted", "both"):
            tainted_spec = clean_spec.with_taint("tainted")
            results[f"{key}_tainted"] = export_palette_slots(tainted_spec, tones=tones)
    if out_dir is not None:
        os.makedirs(out_dir, exist_ok=True)
        for name, data in results.items():
            with open(os.path.join(out_dir, f"{name}.json"), "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2)
                f.write("\n")
    return results


def export_faction_palettes(
    *,
    out_dir: str | None = None,
    tones: int = 3,
) -> dict[str, dict[str, dict[str, str]]]:
    """Build (and optionally write JSON files for) all 6 canonical faction palettes."""
    results: dict[str, dict[str, dict[str, str]]] = {}
    for key, swatch in EMBERWAKE_FACTION_SWATCHES.items():
        results[key] = export_palette_slots(swatch, tones=tones)
    if out_dir is not None:
        os.makedirs(out_dir, exist_ok=True)
        for name, data in results.items():
            with open(os.path.join(out_dir, f"{name}.json"), "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2)
                f.write("\n")
    return results


def _pick_match(text: str, rules: Sequence[tuple[str, str]], fallback: str) -> str:
    for pattern, value in rules:
        if re.search(pattern, text):
            return value
    return fallback


_CLASS_BY_ARCHETYPE: dict[str, str] = {
    "knight": "warrior",
    "rogue": "rogue",
    "mage": "priest",
    "ranger": "ranger",
    "cyber": "mage",
    "samurai": "warrior",
    "brawler": "brawler",
    "creature": "brawler",
}


def interpret_traits(
    description: str,
    base_spec: CharacterSpec | None = None,
    *,
    preferred_style: str | None = None,
) -> CharacterSpec:
    """Offline rule-based regex port of SpriteForge `interpretDescriptionLocally`."""
    current = base_spec or CharacterSpec(id="char-interpreted", name="Unit")
    lower = (description or "").lower()

    rig_body = _pick_match(
        lower,
        (
            (r"centipede|crawler|worm|serpent|arthropod|insect|larva", "crawler"),
            (r"hound|wolf|quadruped|render-beast|dog|panther|boar|stag", "quadruped"),
        ),
        "humanoid",
    )
    archetype = _pick_match(
        lower,
        (
            (r"centipede|crawler|hound|render-beast|golem|slime|demon|dragon|goblin|skeleton|creature|spirit|beast|husk|amalgam", "creature"),
            (r"samurai|ronin|shinobi|shogun|katana", "samurai"),
            (r"alchemist|sprayer|cyber|neon|android|hacker|mech|pilot|sci-fi|tech|drone", "cyber"),
            (r"mage|wizard|witch|sorcer|arcane|astral|warlock|necromancer|cleric|acolyte|scribe|prelate|cantor", "mage"),
            (r"knight|paladin|vanguard|crusader|templar|guardian|armored|footman|inquisitor", "knight"),
            (r"rogue|assassin|thief|ninja|shadow|pirate|bandit|stalker|dredger", "rogue"),
            (r"ranger|archer|arbalest|crossbow|hunter|scout|forest|druid|elf", "ranger"),
            (r"brawler|monk|fighter|barbarian|berserker|warrior|brute|flagellant|stoker", "brawler"),
        ),
        current.archetype,
    )
    build = _pick_match(
        lower,
        (
            (r"heavy|massive|hulking|giant|tank|bulky|juggernaut|broad|towering|bull-necked|corpulent", "heavy"),
            (r"lithe|slim|slender|agile|swift|wiry|rangy|gaunt|lean|acrobat", "lithe"),
            (r"compact|small|tiny|chibi|gnome|goblin|short|cute", "compact"),
        ),
        "balanced",
    )
    headgear = _pick_match(
        lower,
        (
            (r"kettle helm|kettle-helm|watch helm", "kettle-helm"),
            (r"goggle", "goggles"),
            (r"spectacle|glasses|monocle", "spectacles"),
            (r"iron mask|iron-mask|windowless mask|beak mask|face-mask", "iron-mask"),
            (r"mitre|bishop", "mitre"),
            (r"wizard hat|witch hat|pointed hat|hat", "wizard-hat"),
            (r"kabuto|samurai helm", "kabuto"),
            (r"helm|helmet|visored|knight", "knight-helm"),
            (r"hood|cowl|cloak", "hood"),
            (r"visor|cyber|optic|loupe", "visor"),
            (r"horn|demon|oni|antler|ram|tuning-fork|spur", "horns"),
            (r"crown|tiara|royal|king|queen", "crown"),
            (r"cat ear|fox ear|beast ear|ears", "cat-ears"),
        ),
        "none",
    )
    hair_style = _pick_match(
        lower,
        (
            (r"bald|shaved|shaven|skull|hairless", "bald"),
            (r"ponytail|braid|topknot", "ponytail"),
            (r"long|flowing|waist-length|mane", "long-flowing"),
            (r"spiky|wild|messy|punk|curls", "spiky"),
            (r"bob|pageboy|wavy", "bob"),
        ),
        "short",
    )
    eyes = _pick_match(
        lower,
        (
            (r"cyclops|one eye|single eye|mono-eye", "cyclops"),
            (r"visor|slit|blindfold", "visor-slit"),
            (r"glow|burning|luminous|radiant|spectral|tapetum", "glowing"),
            (r"wide|cute|bright|anime|expressive", "wide-expressive"),
        ),
        "determined",
    )
    outfit = _pick_match(
        lower,
        (
            (r"plate|armor|mail|paladin|steel|cuirass|breastplate", "plate-armor"),
            (r"mech|exosuit|pressure suit|ceramic|cybernetic|piston", "mech-suit"),
            (r"robe|arcane|vestment|scholar|priest|habit|cassock|cope", "arcane-robes"),
            (r"haori|kimono|ronin|samurai", "samurai-haori"),
            (r"trench|coat|duster|detective|prosector|vat coat", "trench-coat"),
            (r"stealth|leather|jerkin|brigandine|assassin|garb", "stealth-garb"),
        ),
        "tunic",
    )
    back_accessory = _pick_match(
        lower,
        (
            (r"tank|canister|manifold|jetpack|thruster|booster", "jetpack"),
            (r"wing|angel|seraph|bat|feather", "wings"),
            (r"quiver|arrow|bolt", "quiver"),
            (r"scarf|muffler|bandana", "scarf"),
            (r"tattered|ragged|torn cape", "tattered-cape"),
            (r"cape|cloak|mantle", "royal-cloak"),
        ),
        "none",
    )
    weapon = _pick_match(
        lower,
        (
            (r"billhook|polearm|halberd|glaive|pike", "billhook"),
            (r"censer|thurible|crook", "censer"),
            (r"bone-saw|bonesaw|saw|scalpel", "bone-saw"),
            (r"arbalest|crossbow", "arbalest"),
            (r"sprayer|nozzle|hose|flamethrower", "sprayer"),
            (r"buckler", "buckler-axe"),
            (r"cestus|shackle|cuff|fist|claw|unarmed", "cestus"),
            (r"scourge|flail|whip", "scourge"),
            (r"shield", "shield-sword"),
            (r"greatsword|claymore|broad|broadsword|sword", "greatsword"),
            (r"katana|odachi|blade", "katana"),
            (r"dagger|knives|bodkin|flensing|dual|twin", "twin-daggers"),
            (r"staff|wand|rod|lantern|scepter", "arcane-staff"),
            (r"rifle|gun|blaster|cannon|laser", "energy-rifle"),
            (r"hammer|mace|maul|axe", "warhammer"),
            (r"bow|longbow", "bow"),
        ),
        "greatsword" if rig_body == "humanoid" else "none",
    )
    vfx_aura = _pick_match(
        lower,
        (
            (r"fire|flame|ember|tallow|furnace|kiln|lava|inferno|burn", "embers"),
            (r"frost|ice|snow|glacial|frozen|winter", "frost"),
            (r"lightning|thunder|volt|plasma|electric|neon|shock", "lightning"),
            (r"shadow|void|ash-smoke|cursed|smoke|ghost|wisp", "shadow-wisps"),
            (r"arcane|magic|rune|marrow|blight|chorus|astral|celestial|mystic", "arcane-runes"),
        ),
        "none",
    )
    motif = _pick_match(
        lower,
        (
            (r"sun|solar|gold|dawn", "sun"),
            (r"moon|crescent|lunar|night", "crescent"),
            (r"skull|death|bone|ossuary|undead", "skull"),
            (r"bolt|lightning|storm", "bolt"),
            (r"diamond|crystal|gem", "diamond"),
        ),
        "none",
    )
    rec_style = preferred_style or _pick_match(
        lower,
        (
            (r"pixel|16-bit|48x48|8-bit|retro|snes|gba|emberwake", "pixel-16"),
            (r"chibi|cute|arcade|mini", "chibi-arcade"),
            (r"dark|gothic|soot|souls|gritty|grim|painted", "dark-painted"),
            (r"vector|flat|clean|hollow|minimal", "vector-flat"),
            (r"anime|cel|ink|comic|manga", "cel-ink"),
        ),
        "pixel-16",
    )

    primary = current.palette.primary
    secondary = current.palette.secondary
    accent = current.palette.accent
    hair = current.palette.hair
    skin = current.palette.skin

    if re.search(r"crimson|\bred\b|scarlet|blood|castellan", lower):
        primary, secondary, accent = "#4B5563", "#991B1B", "#FBBF24"
    elif re.search(r"emerald|moss|forest|jade|green|nature", lower):
        primary, secondary, accent = "#22543D", "#3B2F2F", "#A3E635"
    elif re.search(r"cobalt|sapphire|ocean|blue|navy", lower):
        primary, secondary, accent = "#1E3A5F", "#1E293B", "#38BDF8"
    elif re.search(r"violet|purple|marrow|blight|chorus|arcane", lower):
        primary, secondary, accent = "#1E232D", "#4B5563", "#9333EA"
    elif re.search(r"tallow|gold|amber|brass|kiln", lower):
        primary, secondary, accent = "#92400E", "#2C221E", "#D97706"
    elif re.search(r"linen|bleached|ossuary|white|silver|ivory", lower):
        primary, secondary, accent = "#D9CBB4", "#4B5563", "#9333EA"
    elif re.search(r"obsidian|basalt|inquisition|black|shadow|dark|stealth", lower):
        primary, secondary, accent = "#14171F", "#0A0B10", "#E11D48"

    if re.search(r"cyan|salt|phosphor|neon|plasma", lower):
        accent = "#06B6D4"
    if re.search(r"fire|ember|flame", lower):
        accent = "#D97706"
    if re.search(r"frost|ice", lower):
        accent = "#7DD3FC"

    if re.search(r"silver hair|white hair|pale hair", lower):
        hair = "#D9CBB4"
    elif re.search(r"blonde|straw-blond|golden hair", lower):
        hair = "#D4B483"
    elif re.search(r"red hair|rust-red|ginger|crimson hair", lower):
        hair = "#991B1B"
    elif re.search(r"black hair|ink-black|raven", lower):
        hair = "#14171F"

    if re.search(r"stone|calcified|ash-grey|golem|ceramic|mech", lower):
        skin = "#6B7280"
    elif re.search(r"pale|chalk|undead|ghost", lower):
        skin = "#D4A988"
    elif re.search(r"deeper brown|deep brown", lower):
        skin = "#6B4A3A"

    taint_tier = "tainted" if re.search(r"\btainted\b|blight-tainted|corrupted", lower) else current.taint_tier

    first_clause = re.split(r"[,.;]", (description or "").strip())[0]
    words = [w for w in first_clause.split() if w][:4]
    gen_name = " ".join(w[0].upper() + w[1:] for w in words) if words else current.name
    if len(gen_name) <= 2:
        gen_name = current.name

    return replace(
        current,
        name=gen_name,
        rig_body=rig_body,
        archetype=archetype,
        class_id=_CLASS_BY_ARCHETYPE.get(archetype, current.class_id),
        build=build,
        headgear=headgear,
        hair_style=hair_style,
        eyes=eyes,
        outfit=outfit,
        back_accessory=back_accessory,
        weapon=weapon,
        vfx_aura=vfx_aura,
        motif=motif,
        taint_tier=taint_tier,
        style=rec_style,
        recommended_style=rec_style,
        description=(description or "").strip(),
        palette=CharacterPalette(primary, secondary, accent, skin, hair, EMBERWAKE_OUTLINE_HEX),
        art_notes=f"Structured {rig_body} silhouette for {archetype} ({build} build) with {weapon.replace('-', ' ')} focal arc.",
    )


def _iter_palette_rgbs(
    palette: CharacterPalette | CharacterSpec | Mapping[str, object] | Iterable[object],
) -> list[tuple[int, int, int]]:
    if isinstance(palette, CharacterSpec):
        return list(palette.palette.rgb_list)
    if isinstance(palette, CharacterPalette):
        return list(palette.rgb_list)
    if isinstance(palette, Mapping):
        if "slots" in palette and isinstance(palette["slots"], Mapping):
            vals = list(palette["slots"].values())
        else:
            vals = list(palette.values())
        return [parse_hex_rgb(v) for v in vals if isinstance(v, (str, tuple, list))]  # type: ignore[arg-type]
    return [parse_hex_rgb(v) for v in palette]  # type: ignore[arg-type]


def resolve_chroma_key(
    palette: CharacterPalette | CharacterSpec | Mapping[str, object] | Iterable[object],
    mode: str = "auto",
) -> tuple[str, tuple[int, int, int]]:
    """Select an uncolliding high-saturation chroma-key background (Green -> Magenta -> Blue)."""
    if mode not in CHROMA_KEY_MODES:
        raise ValueError(f"Invalid chroma mode={mode!r}; expected {CHROMA_KEY_MODES}")
    if mode != "auto":
        return CHROMA_KEY_SPECS[mode]

    rgbs = _iter_palette_rgbs(palette)
    has_green = any(g > r + 24 and g > b + 24 and g > 70 for r, g, b in rgbs)
    if not has_green:
        return CHROMA_KEY_SPECS["green"]
    has_magenta = any(r > g + 35 and b > g + 35 and r > 90 and b > 90 for r, g, b in rgbs)
    if not has_magenta:
        return CHROMA_KEY_SPECS["magenta"]
    return CHROMA_KEY_SPECS["blue"]


def _fit_frames_to_count(frames: Sequence[Image.Image], count: int) -> list[Image.Image]:
    if not frames:
        return [Image.new("RGBA", (rig.W, rig.W), (0, 0, 0, 0))] * count
    if len(frames) == count:
        return list(frames)
    if len(frames) > count:
        # Evenly sample `count` keyframes including first & last when count >= 2.
        return [frames[(i * (len(frames) - 1)) // max(1, count - 1)] for i in range(count)]
    return [frames[min(i, len(frames) - 1)] for i in range(count)]


def _composite_cell(
    frame: Image.Image,
    bg_rgb: tuple[int, int, int],
    cell_size: tuple[int, int] | None,
) -> Image.Image:
    cw, ch = cell_size if cell_size else (frame.width, frame.height)
    bg = Image.new("RGBA", (cw, ch), (*bg_rgb, 255))
    scaled = frame.resize((cw, ch), Image.NEAREST) if (cw, ch) != frame.size else frame
    bg.alpha_composite(scaled)
    return bg


def render_chroma_strip(
    frames: Sequence[Image.Image],
    *,
    palette: CharacterPalette | CharacterSpec | Mapping[str, object] | Iterable[object] = (),
    mode: str = "auto",
    cols: int = 4,
    cell_size: tuple[int, int] | None = None,
) -> tuple[Image.Image, dict]:
    """Compose a single-row N-frame (default 4-frame) horizontal strip on an unblended chroma key."""
    key_hex, key_rgb = resolve_chroma_key(palette, mode=mode)
    sampled = _fit_frames_to_count(frames, cols)
    cw, ch = cell_size if cell_size else (sampled[0].width, sampled[0].height)
    strip = Image.new("RGBA", (cw * cols, ch), (*key_rgb, 255))
    for idx, fr in enumerate(sampled):
        strip.paste(_composite_cell(fr, key_rgb, (cw, ch)), (idx * cw, 0))
    return strip, {
        "layout": "strip",
        "cols": cols,
        "rows": 1,
        "cell_size": [cw, ch],
        "chroma_hex": key_hex,
        "chroma_rgb": list(key_rgb),
        "show_shadow_base": False,
    }


def render_chroma_sheet(
    rows: Mapping[str, Sequence[Image.Image]],
    *,
    pose_order: Sequence[str] | None = None,
    palette: CharacterPalette | CharacterSpec | Mapping[str, object] | Iterable[object] = (),
    mode: str = "auto",
    frames_per_pose: int = 4,
    grid_cols_poses: int = 2,
    cell_size: tuple[int, int] | None = None,
) -> tuple[Image.Image, dict]:
    """Compose an 8-pose (2x4 poses x 4 frames = 8 cols) review sheet on a resolved chroma key."""
    key_hex, key_rgb = resolve_chroma_key(palette, mode=mode)
    # Default mapping from POSE_SHEET_ORDER onto available emberwake rig clip rows.
    clip_aliases: dict[str, tuple[str, ...]] = {
        "idle": ("idle",),
        "walk": ("walk",),
        "jump": ("dash", "windup_buff", "walk"),
        "attack": ("attack", "jab", "haymaker"),
        "cast": ("windup_buff", "windup", "attack"),
        "guard": ("defend", "idle_guarded", "idle"),
        "hit": ("hurt", "stagger"),
        "victory": ("windup_buff", "idle"),
    }
    ordered = list(pose_order or POSE_SHEET_ORDER)
    selected: list[tuple[str, Sequence[Image.Image]]] = []
    for pose_name in ordered:
        chosen = None
        if pose_name in rows:
            chosen = rows[pose_name]
        else:
            for cand in clip_aliases.get(pose_name, ()):
                if cand in rows:
                    chosen = rows[cand]
                    break
        if chosen is None and rows:
            chosen = next(iter(rows.values()))
        if chosen is not None:
            selected.append((pose_name, chosen))

    sample_frame = next(iter(rows.values()))[0] if rows else Image.new("RGBA", (rig.W, rig.W), (0, 0, 0, 0))
    cw, ch = cell_size if cell_size else (sample_frame.width, sample_frame.height)
    pose_rows = max(1, (len(selected) + grid_cols_poses - 1) // grid_cols_poses)
    total_cols = grid_cols_poses * frames_per_pose
    sheet = Image.new("RGBA", (total_cols * cw, pose_rows * ch), (*key_rgb, 255))

    for pose_idx, (_, pose_frames) in enumerate(selected):
        r = pose_idx // grid_cols_poses
        c0 = (pose_idx % grid_cols_poses) * frames_per_pose
        for f_idx, fr in enumerate(_fit_frames_to_count(pose_frames, frames_per_pose)):
            sheet.paste(_composite_cell(fr, key_rgb, (cw, ch)), ((c0 + f_idx) * cw, r * ch))

    return sheet, {
        "layout": "sheet",
        "poses": [p for p, _ in selected],
        "cols": total_cols,
        "rows": pose_rows,
        "frames_per_pose": frames_per_pose,
        "cell_size": [cw, ch],
        "chroma_hex": key_hex,
        "chroma_rgb": list(key_rgb),
        "show_shadow_base": False,
    }

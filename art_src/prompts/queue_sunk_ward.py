#!/usr/bin/env python3
"""Queue the Sunk Ward props sheets and a terrain-tile probe on the Drive web queue (Art/UI 2026-11-15).

    python art_src/prompts/queue_sunk_ward.py props [--dry-run]     # sheets S4-S8, S11 and the Kiln image S9
    python art_src/prompts/queue_sunk_ward.py tiles [--dry-run]     # the tile probe: 4 prompts, before the full set
    python art_src/prompts/queue_sunk_ward.py tiles-all [--dry-run] # every ground, boundary and interior wall prompt

The prompts are read from sunk_ward_props.md and sunk_ward_walls.md (the first ```text block after each heading), so
this file holds no prompt text of its own except the nine street grounds, which use REGIONGROUND1's short
material-first form with the materials of its section 7 (A3). A request attaches the landmark references a prompt
names in its opening "References:" sentence, in that order. No API is called: requests are JSON files in
image_pipeline/queue/ for the web app's task mode.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "portraits"))
import artqueue as Q  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
BLOCK = re.compile(r"```text\n(.*?)```", re.S)
GROUND = ("Seamless top-down pixel-art floor texture, 16-bit RPG tile style: {what}. Even detail edge to edge, no "
          "objects, no shadows, no light direction, flat and matte, tiles seamlessly on all four sides.")
STREET_GROUNDS = {  # REGIONGROUND1 section 7, A3
    "soot_yard": "soot-black packed ground with fine grit",
    "cobble_road": "damp rounded cobbles",
    "flagstone": "soot-stained flagstones of many sizes",
    "mud": "black lane mud with small even pits",
    "moss": "low damp moss, close and even, no grass, no blades",
    "iron_catwalk": "riveted dark iron plates of many sizes, in no rows, faint rust",
    "tallow": "packed ground soaked with pale tallow, fine even mottling",
    "water": "still black canal water, tiny even ripples, no highlights, no reflections",
}


def block_after(text, heading):
    i = text.index(heading)
    return BLOCK.search(text, i).group(1).strip()


def refs_of(prompt):
    head = prompt.split(". A sheet", 1)[0].split(". One pixel-art", 1)[0]
    return [ROOT / "art_src" / n for n in re.findall(r"lm_[a-z0-9_]+\.jpg", head)] if prompt.startswith("References:") else []


def props():
    text = (HERE / "sunk_ward_props.md").read_text(encoding="utf-8")
    out = []
    for sid, heading in (("props_sunk_undercroft", "### Sheet S4"), ("props_sunk_drains", "### Sheet S5"),
                         ("props_sunk_streets", "### Sheet S6"), ("props_sunk_works", "### Sheet S7"),
                         ("props_sunk_ground", "### Sheet S8"), ("props_sunk_interiors", "### Sheet S11")):
        out.append((sid, block_after(text, heading), "1:1"))
    out.append(("props_sunk_kiln", block_after(text, "### Single image S9"), "16:9"))
    return out


def tiles(every):
    text = (HERE / "sunk_ward_walls.md").read_text(encoding="utf-8")
    out = [("ground_floorboards", block_after(text, "`floorboards` (`#3a2c22`)"))]
    for name, what in STREET_GROUNDS.items():
        out.append((f"ground_{name}", GROUND.format(what=what)))
    for kind, sets in (("boundary", ("drain_brick", "blind_tenements", "gorge_rock")),
                       ("walls", ("interior_plaster", "interior_stone"))):
        for s in sets:
            body = block_after(text, f"`{s}` (")
            for line in body.splitlines():
                part, prompt = line.split(": ", 1)
                out.append((f"{kind}_{s}_{part}", prompt.strip()))
    if not every:  # the probe (REGIONGROUND1: a prompt is tested on 2-3 images before a batch)
        keep = {"ground_floorboards", "ground_cobble_road", "boundary_drain_brick_wall", "boundary_drain_brick_wall_face"}
        out = [o for o in out if o[0] in keep]
    return [(i, p, "1:1") for i, p in out]


def main(argv):
    if not argv or argv[0] not in ("props", "tiles", "tiles-all"):
        sys.exit(__doc__)
    dry = "--dry-run" in argv
    pipe = Q.pipeline() if hasattr(Q, "pipeline") else None
    reqs = props() if argv[0] == "props" else tiles(argv[0] == "tiles-all")
    task = "sunk_ward_props" if argv[0] == "props" else "sunk_ward_tiles"
    for cid, prompt, aspect in reqs:
        refs = refs_of(prompt)
        for r in refs:
            if r.stat().st_size < 1000:
                sys.exit(f"{r} is a Git LFS pointer, not the image: git lfs pull --include {r.relative_to(ROOT).as_posix()}")
        req = Q.build_request(prompt, cid, refs=refs, n=2, aspect=aspect, route="other", task=task,
                              record={"prompts_file": "art_src/prompts/sunk_ward_props.md" if argv[0] == "props"
                                      else "art_src/prompts/sunk_ward_walls.md"})
        name = Q.request_name(pipe, cid)
        Q.enqueue(pipe, name, req, dry=dry)
        print(f"{name}: {'would queue' if dry else 'queued'}, {len(prompt)} chars, {aspect}, {len(refs)} reference(s)")


if __name__ == "__main__":
    main(sys.argv[1:])

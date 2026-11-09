"""Standalone 60fps Kaelen 4-Pose & Weapon Action Animation Lab Builder.

Usage:
    python3 -m tools.mock_ui.anim_lab --out /path/to/kaelen_4pose_animation_lab.html
"""
import argparse
from pathlib import Path

HERE = Path(__file__).resolve().parent

def build_anim_lab(out_path: Path) -> None:
    template = (HERE / "template.html").read_text(encoding="utf-8")
    uris_json = (HERE / "kaelen_4pose_uris.json").read_text(encoding="utf-8").strip()
    html = template.replace("__URIS_JSON__", uris_json)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(html, encoding="utf-8")
    print(f"Built standalone animation lab ({len(html):,} bytes) -> {out_path}")

def main() -> None:
    parser = argparse.ArgumentParser(description="Build standalone Kaelen 4-pose animation lab HTML.")
    parser.add_argument(
        "--out",
        type=Path,
        default=HERE.parent.parent.parent / "kaelen_4pose_animation_lab.html",
        help="Output path for the standalone animation lab HTML file.",
    )
    args = parser.parse_args()
    build_anim_lab(args.out)

if __name__ == "__main__":
    main()

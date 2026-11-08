#!/usr/bin/env python3
"""
Assembles the modular Emberwake UI prototype (`tools/mock_ui`) into a single
self-contained playable HTML5 preview (`emberwake_screen_flow.html`).

Usage:
  python3 -m tools.mock_ui [--out PATH] [--rebuild-assets]
  python3 tools/mock_ui/build.py [--out PATH] [--rebuild-assets]
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

try:
    from .assets import REPO_ROOT, load_or_build_bundle
except ImportError:
    from assets import REPO_ROOT, load_or_build_bundle

PKG_DIR = Path(__file__).resolve().parent
JS_DIR = PKG_DIR / "js"

ARTIFACT_DIR = Path(
    "/home/antigravity/.gemini/jetski/brain/v1/default/jetski_remote_agents_3p/e6a86537-0222-4bba-b985-efe38312ec80"
)
DEFAULT_OUT = (
    ARTIFACT_DIR / "emberwake_screen_flow.html"
    if ARTIFACT_DIR.is_dir()
    else REPO_ROOT / "target" / "emberwake_screen_flow.html"
)
DEFAULT_CACHE = (
    ARTIFACT_DIR / "scratch" / "assets_bundle.json"
    if ARTIFACT_DIR.is_dir()
    else Path("/tmp/emberwake_mock_ui_assets.json")
)


def assemble_html(rebuild_assets: bool = False, cache_path: Path | None = DEFAULT_CACHE) -> str:
    bundle = load_or_build_bundle(cache_path=cache_path, rebuild=rebuild_assets)

    shell_html = (PKG_DIR / "shell.html").read_text(encoding="utf-8")
    styles_css = (
        (PKG_DIR / "styles.css")
        .read_text(encoding="utf-8")
        .replace("__FONT_DISPLAY__", bundle["fonts"]["display"])
        .replace("__FONT_BODY__", bundle["fonts"]["body"])
        .replace("__FONT_MONO__", bundle["fonts"]["mono"])
        .replace("__ORNATE_CORNER__", bundle["ui"]["ornate_corner"])
    )

    js_files = sorted(JS_DIR.glob("*.js"))
    app_js = "\n\n".join(p.read_text(encoding="utf-8") for p in js_files)

    html = (
        shell_html
        .replace("__STYLES_CSS__", styles_css)
        .replace("__ASSETS_JSON__", json.dumps(bundle))
        .replace("__APP_JS__", app_js)
    )
    return html


def build(out_path: Path = DEFAULT_OUT, rebuild_assets: bool = False, cache_path: Path | None = DEFAULT_CACHE) -> Path:
    html = assemble_html(rebuild_assets=rebuild_assets, cache_path=cache_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(html, encoding="utf-8")
    print(f"[mock_ui] Wrote {out_path} ({out_path.stat().st_size / 1024:.1f} KB)")
    return out_path


def main() -> None:
    ap = argparse.ArgumentParser(description="Build the self-contained Emberwake UI prototype HTML.")
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT, help="Output HTML file path")
    ap.add_argument("--cache", type=Path, default=DEFAULT_CACHE, help="Asset bundle JSON cache path")
    ap.add_argument("--rebuild-assets", action="store_true", help="Force rebuilding the layered asset bundle")
    args = ap.parse_args()
    build(out_path=args.out, rebuild_assets=args.rebuild_assets, cache_path=args.cache)


if __name__ == "__main__":
    main()

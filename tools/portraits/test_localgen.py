# Tests of localgen.py (LOCALGEN1): the prompt record's shape and the short prompt for 77-token text encoders, with
# no model and no OpenVINO (the token count is the tool's estimate, or the model's own CLIP tokenizer files when they
# are there), plus the --dry-run smoke. LOCALGEN2: the ComfyUI backend's Klein graphs (wiring against the verified
# probe graphs), the whole prompt's order, the records and the commands over a fake HTTP layer (no server, no model).
# Run from anywhere with the plain Python (Pillow only):
#   python tools/portraits/test_localgen.py
import contextlib
import io
import json
import pathlib
import re
import shutil
import sys
import tempfile
import unittest
import urllib.error
import urllib.parse
from unittest import mock

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import generate  # noqa: E402
import localgen  # noqa: E402

BLOCK = ("Character: Wren Ash, Lamp-Keeper of the Low Gate. Stylization warm and readable: large, bright, alert eyes, "
         "a small pointed chin (the left side of the picture), a quick lopsided half-smile. A young woman of nineteen, "
         "an adult, wiry and compact. Long, thick, wild dark-brown hair: one messy loose braid over her left shoulder, "
         "no hood. Patched leather armour over greasy wool wraps, a lantern pole across her back with a cracked "
         "#d97706 glass, a satchel strap across the chest, soot on her cheeks, a rope coiled over one shoulder, "
         "a brass whistle on a cord.\nMood: quick, wary, determined.\nMain colours: boiled leather brown, grey wool.")
PROMPTS_MD = ("# Test prompts\n\n```text\nShared style block.\n```\n\n## Wren Ash (`wren`)\n\n```text\n" + BLOCK +
              "\n```\n\n## Pell (`pell`)\n\n```text\nCharacter: Pell, a moth the size of a dog. Grey dusty wings.\n"
              "Mood: still.\n```\n")


OV = ("--backend", "openvino")


def records(out):
    """The records a --dry-run printed, each after its `--- id #n` line."""
    dec = json.JSONDecoder()
    return [dec.raw_decode(chunk.split("\n", 1)[1])[0] for chunk in out.split("--- ")[1:]]


def run(*argv):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = localgen.main(list(map(str, argv)))
    return code, out.getvalue()


class ShortPrompt(unittest.TestCase):
    def test_fits_the_encoder_and_leads_with_the_subject(self):
        prompt, cut = localgen.short_prompt(BLOCK)
        self.assertLessEqual(localgen.estimate(prompt), localgen.TOKENS)
        self.assertTrue(prompt.startswith(localgen.LEAD + " a young woman of nineteen, an adult"), prompt)
        self.assertTrue(prompt.endswith(localgen.TAIL), prompt)
        self.assertTrue(cut, "a long block loses clauses, and says which")

    def test_name_parentheses_hex_and_negations_are_cut_and_listed(self):
        prompt, cut = localgen.short_prompt(BLOCK)
        for gone in ("Wren", "left side of the picture", "#d97706", "no hood", "Stylization"):
            self.assertNotIn(gone, prompt)
        self.assertIn("name: Wren Ash", cut)
        self.assertIn("title: Lamp-Keeper of the Low Gate", cut)
        self.assertIn("parentheses: the left side of the picture", cut)
        self.assertIn("negation: no hood", cut)

    def test_every_clause_is_kept_or_cut(self):
        prompt, cut = localgen.short_prompt(BLOCK)
        for clause in ("a brass whistle on a cord", "a rope coiled over one shoulder", "grey wool"):
            self.assertTrue(clause in prompt or clause in cut, clause)

    def test_breadth_first_every_sentence_gets_its_lead_clause(self):
        prompt, _ = localgen.short_prompt(BLOCK)
        for lead in ("large, bright, alert eyes", "long, thick, wild dark-brown hair", "patched leather armour"):
            self.assertIn(lead, prompt)
        self.assertLess(prompt.index("a young woman"), prompt.index("large, bright"),
                        "who comes before the stylization sentence")
        self.assertLess(prompt.index("large, bright"), prompt.index("long, thick"), "then the hair")

    def test_adjective_lists_stay_with_their_noun(self):
        self.assertEqual(localgen._clauses("Large, bright, alert eyes, a small chin."),
                         ["large, bright, alert eyes", "a small chin"])

    def test_adult_is_never_cut(self):
        who = "a young woman of nineteen, an adult, wiry and compact"  # the adult clause joined the who clause
        tight = localgen.estimate(f"{localgen.LEAD} {who}, {localgen.TAIL}")
        prompt, _ = localgen.short_prompt(BLOCK, tokens=tight)
        self.assertEqual(prompt, f"{localgen.LEAD} {who}, {localgen.TAIL}", "the who clause is tried first")
        young = "Character: Kit, Runner. A young woman, quick on her feet. Short hair."
        self.assertIn("adult, a young woman", localgen.short_prompt(young)[0])

    def test_a_creature_keeps_its_epithet(self):
        prompt, cut = localgen.short_prompt(localgen.block_prompts(self.prompts_file())["pell"])
        self.assertIn("a moth the size of a dog", prompt)
        self.assertIn("name: Pell", cut)

    def test_a_change_goes_first_and_replaces_the_mood(self):
        prompt, cut = localgen.short_prompt(BLOCK, change="afraid, wide eyes")
        self.assertTrue(prompt.startswith("afraid, wide eyes, " + localgen.LEAD), prompt)
        self.assertNotIn("determined", prompt)
        self.assertTrue(any(c.startswith("mood (the change replaces it)") for c in cut))
        self.assertLessEqual(localgen.estimate(prompt), localgen.TOKENS)

    def test_trigger_words_join_the_tail(self):
        prompt, _ = localgen.short_prompt(BLOCK, trigger="pixel art")
        self.assertTrue(prompt.endswith(localgen.TAIL + ", pixel art"))

    def test_a_whole_recorded_prompt_shortens_like_its_block(self):
        whole = "Style block. Pixel art portrait, no frame.\n\n" + BLOCK
        self.assertEqual(localgen.short_prompt(whole), localgen.short_prompt(BLOCK))

    def test_free_prompt_overflow(self):
        self.assertEqual(localgen.overflow("a b c", localgen.estimate), [])
        long = " ".join(["lantern"] * 100)
        cut = localgen.overflow(long, localgen.estimate)
        self.assertEqual(len(cut), 1)
        self.assertLessEqual(localgen.estimate(long[:len(long) - len(cut[0])]), localgen.TOKENS)

    @unittest.skipUnless((localgen.MODEL / "tokenizer" / "merges.txt").exists(), "the local model is not installed")
    def test_clip_count_with_the_models_tokenizer(self):
        count = localgen.counter()
        self.assertIsInstance(count, localgen.ClipTokens)
        # the OpenVINO tokenizer of the model gives 26 tokens between its start and end marks (checked 2026-10-26)
        self.assertEqual(count("head and shoulders portrait of a young woman of nineteen, an adult, wiry and "
                               "compact, large, bright, alert eyes"), 26)
        self.assertLessEqual(count(localgen.short_prompt(BLOCK, count)[0]), localgen.TOKENS)

    def prompts_file(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_test_"))
        self.addCleanup(shutil.rmtree, tmp, True)
        path = tmp / "PROMPTS.md"
        path.write_text(PROMPTS_MD, encoding="utf-8")
        return path


class Families(unittest.TestCase):
    def model(self, name, cls=None):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_model_"))
        self.addCleanup(shutil.rmtree, tmp, True)
        folder = tmp / name
        folder.mkdir()
        if cls:
            (folder / "model_index.json").write_text(json.dumps({"_class_name": cls}), encoding="utf-8")
        return folder

    def test_family_from_the_pipeline_class_then_the_folder_name(self):
        self.assertEqual(localgen.family(self.model("anything", "FluxPipeline")), "flux")
        self.assertEqual(localgen.family(self.model("lcm_named", "StableDiffusionXLPipeline")), "sdxl")
        self.assertEqual(localgen.family(self.model("x", "LatentConsistencyModelPipeline")), "lcm")
        self.assertEqual(localgen.family(self.model("my_lcm_model")), "lcm")
        self.assertEqual(localgen.family(self.model("flux1_schnell_int4")), "flux")
        self.assertEqual(localgen.family(self.model("sdxl_base_int8")), "sdxl")
        self.assertEqual(localgen.family(self.model("dreamshaper_8")), "sd15")

    def test_family_defaults_fill_what_was_not_given(self):
        flux = self.model("f", "FluxPipeline")
        a = localgen.parser().parse_args(["run", "wren", "--model", str(flux), "--steps", "6"])
        localgen.settings(a)
        self.assertEqual((a.steps, a.size, a.guidance), (6, localgen.FAMILIES["flux"]["size"], 0.0))
        rec = localgen.make_record("wren", "p", 1, model=flux, device="GPU", seed=1, steps=a.steps, size=a.size,
                                   guidance=a.guidance)
        self.assertEqual((rec["family"], rec["negative"]), ("flux", None))
        sdxl = self.model("s", "StableDiffusionXLPipeline")
        rec = localgen.make_record("wren", "p", 1, model=sdxl, device="GPU", seed=1, steps=25, size=1024,
                                   guidance=7.0)
        self.assertEqual(rec["negative"], localgen.NEGATIVE)

    def test_only_clip_only_families_take_the_short_form(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_fam_"))
        self.addCleanup(shutil.rmtree, tmp, True)
        (tmp / "P.md").write_text(PROMPTS_MD, encoding="utf-8")
        for cls, short in (("FluxPipeline", False), ("StableDiffusionXLPipeline", True),
                           ("LatentConsistencyModelPipeline", True), ("StableDiffusionPipeline", True)):
            model = self.model("m", cls)
            code, out = run("prompt", "wren", "--prompts", tmp / "P.md", "--model", model, *OV)
            self.assertEqual(code, 0)
            self.assertEqual(localgen.LEAD in out, short, cls)
            self.assertEqual("Character: Wren Ash" in out, not short, cls)

    def test_full_prompt_is_subject_first_without_the_reference_line(self):
        style = "Use the attached image only as the reference for style.\nPixel art portrait, dark background."
        prompt, cut = localgen.full_prompt(BLOCK, style, change="angry")
        self.assertTrue(prompt.startswith("angry\nCharacter: Wren Ash"), prompt)
        self.assertTrue(prompt.endswith("Pixel art portrait, dark background."))
        self.assertNotIn("attached", prompt)
        self.assertTrue(cut[0].startswith("reference line: Use the attached image"))
        whole = style + "\n\n" + BLOCK
        self.assertEqual(localgen.full_prompt(whole)[0], localgen.full_prompt(BLOCK, style)[0],
                         "a whole recorded prompt splits at Character:")

    def test_full_prompt_reports_what_passes_the_t5_reach(self):
        _, cut = localgen.full_prompt(BLOCK, "", tokens=40)
        self.assertTrue(any(c.startswith("past about 40 T5 tokens") for c in cut))
        self.assertEqual(localgen.full_prompt(BLOCK, "")[1], [], "the block alone fits 256")


class Record(unittest.TestCase):
    KW = dict(model=r"C:\x\models\lcm_dreamshaper_v7_int8", device="NPU", seed=3, steps=6, size=512, guidance=8.0)

    def test_text_record_has_generate_shape_plus_local_settings(self):
        rec = localgen.make_record("wren", "the prompt", 2, prompts_file=localgen.ART3, cut=["x"], seconds=2.5,
                                   when="2026-10-26T00:00:00+00:00", **self.KW)
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_rec_"))
        self.addCleanup(shutil.rmtree, tmp, True)
        api = json.loads(generate.record(tmp / "a.png", "gemini-x", "wren", "the prompt", False, localgen.ART3, 2,
                                         when="2026-10-26T00:00:00+00:00").read_text(encoding="utf-8"))
        self.assertEqual(list(rec)[:len(api)], list(api), "generate.py's keys first, in its order")
        self.assertEqual(rec["model"], "local:lcm_dreamshaper_v7_int8")
        self.assertEqual(rec["prompts_file"], "games/emberwake/docs/specs/ART3_PROMPTS.md")
        for k, v in dict(device="NPU", seed=3, steps=6, size=512, guidance=8.0, cut=["x"], seconds=2.5,
                         reference=None, negative=None, lora=None).items():
            self.assertEqual(rec[k], v, k)
        self.assertNotIn("source_image", rec)
        self.assertFalse(any("key" in k.lower() for k in rec), "never a key")

    def test_variant_record_names_source_and_strength(self):
        rec = localgen.make_record("wren", "p", 1, source_image=generate.SRC / "wren.png", strength=0.4,
                                   change="angry", lora=[(r"C:\l\pix.safetensors", 0.7)], **self.KW)
        self.assertEqual((rec["source_image"], rec["strength"], rec["change"]), ("art_src/wren.png", 0.4, "angry"))
        self.assertEqual(rec["lora"], [{"path": "pix.safetensors", "alpha": 0.7}])

    def test_lora_path_in_a_record_names_no_machine_path(self):
        self.assertEqual(localgen.lora_rel(localgen.HOME / "lora" / "pix.safetensors"), "$LOCALGEN_HOME/lora/pix.safetensors")
        self.assertEqual(localgen.lora_rel(localgen.ROOT / "art_src" / "pix.safetensors"), "art_src/pix.safetensors")
        self.assertEqual(localgen.lora_rel(r"D:\elsewhere\pix.safetensors"), "pix.safetensors")

    def test_negative_prompt_only_for_models_that_take_one(self):
        rec = localgen.make_record("w", "p", 1, **{**self.KW, "model": r"C:\x\models\sd15_int8"})
        self.assertEqual(rec["negative"], localgen.NEGATIVE)
        self.assertIn("hood", localgen.NEGATIVE, "the ART3B hair rule, as a negative")

    def test_lora_spec(self):
        self.assertEqual(localgen.parse_lora(r"C:\l\pix.safetensors:0.6"), (r"C:\l\pix.safetensors", 0.6))
        self.assertEqual(localgen.parse_lora("pix.safetensors"), ("pix.safetensors", 0.8))

    def test_lora_kohya_renames_keys_and_keeps_the_bytes(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_lora_"))
        self.addCleanup(shutil.rmtree, tmp, True)
        data = bytes(range(16))
        header = {"unet.down_blocks.0.attentions.0.transformer_blocks.0.attn1.to_k.lora.down.weight":
                  {"dtype": "F32", "shape": [2, 2], "data_offsets": [0, 16]}, "__metadata__": {"format": "pt"}}
        text = json.dumps(header).encode()
        (tmp / "a.safetensors").write_bytes(len(text).to_bytes(8, "little") + text + data)
        self.assertEqual(localgen.convert_lora(tmp / "a.safetensors", tmp / "b.safetensors"), 1)
        raw = (tmp / "b.safetensors").read_bytes()
        n = int.from_bytes(raw[:8], "little")
        self.assertEqual(n % 8, 0)
        self.assertEqual(set(json.loads(raw[8:8 + n])), {
            "lora_unet_down_blocks_0_attentions_0_transformer_blocks_0_attn1_to_k.lora_down.weight", "__metadata__"})
        self.assertEqual(raw[8 + n:], data)

    def test_contact_sheet_has_a_cell_pair_per_take(self):
        from PIL import Image
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_sheet_"))
        self.addCleanup(shutil.rmtree, tmp, True)
        takes = []
        for k in range(3):
            im = Image.new("RGB", (64, 64), (18, 14, 18))
            im.paste((160, 110, 80), (20, 16, 44, 56))
            path = tmp / f"wren_loc1_{k + 1}.png"
            im.save(path)
            localgen.write_record(path, localgen.make_record("wren", "p", k + 1, **self.KW))
            takes.append(path)
        out = localgen.contact(tmp / "sheet.png", takes, cols=2, cell=60)
        with Image.open(out) as sheet:
            self.assertEqual(sheet.size, (2 * 60 * 2, 2 * (60 + 34)))

    def test_take_name(self):
        self.assertEqual(localgen.take_path(generate.SRC, "tess", 2, 1792950000).name, "tess_loc1792950000_2.png")


class DryRun(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_dry_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)
        (self.tmp / "P.md").write_text(PROMPTS_MD, encoding="utf-8")

    def test_run_dry_prints_the_records_and_writes_nothing(self):
        code, out = run("run", "wren", *OV, "--prompts", self.tmp / "P.md", "--n", "2", "--seed", "5", "--device", "gpu",
                        "--out", self.tmp / "out", "--dry-run")
        self.assertEqual(code, 0)
        recs = records(out)
        self.assertEqual([(r["n"], r["seed"], r["device"]) for r in recs], [(1, 5, "GPU"), (2, 6, "GPU")])
        self.assertTrue(recs[0]["prompt"].startswith(localgen.LEAD))
        self.assertFalse((self.tmp / "out").exists())

    def test_vary_dry_names_each_strength(self):
        code, out = run("vary", "wren", *OV, "--from", self.tmp / "wren.png", "--prompts", self.tmp / "P.md",
                        "--change", "hurt, wincing", "--strength", "0.3", "--strength", "0.5", "--dry-run")
        self.assertEqual(code, 0)
        recs = records(out)
        self.assertEqual([r["strength"] for r in recs], [0.3, 0.5])
        self.assertTrue(all(r["prompt"].startswith("hurt, wincing, ") for r in recs))

    def test_prompt_lists_the_cut(self):
        code, out = run("prompt", "wren", *OV, "--prompts", self.tmp / "P.md")
        self.assertEqual(code, 0)
        self.assertIn("cut: name: Wren Ash", out)


# ------------------------------------------------------------------------------------------- ComfyUI backend (Klein)

# The wiring of the graphs verified on this machine (C:\dev\localgen\probe\comfy_run.py, comfy_faces.py, 2026-10-26):
# node -> (class_type, {input: source node}); the edit adds nodes 20-25 and re-links the guider, scheduler and latent.
PROBE_TEXT = {
    "1": ("UNETLoader", {}), "2": ("CLIPLoader", {}), "3": ("VAELoader", {}),
    "4": ("CLIPTextEncode", {"clip": "2"}), "5": ("ConditioningZeroOut", {"conditioning": "4"}),
    "6": ("CFGGuider", {"model": "1", "positive": "4", "negative": "5"}), "7": ("KSamplerSelect", {}),
    "8": ("Flux2Scheduler", {}), "9": ("EmptyFlux2LatentImage", {}), "10": ("RandomNoise", {}),
    "11": ("SamplerCustomAdvanced", {"noise": "10", "guider": "6", "sampler": "7", "sigmas": "8", "latent_image": "9"}),
    "12": ("VAEDecode", {"samples": "11", "vae": "3"}), "13": ("SaveImage", {"images": "12"}),
}
PROBE_EDIT = {
    **PROBE_TEXT,
    "6": ("CFGGuider", {"model": "1", "positive": "24", "negative": "25"}),
    "8": ("Flux2Scheduler", {"width": "22", "height": "22"}),
    "9": ("EmptyFlux2LatentImage", {"width": "22", "height": "22"}),
    "20": ("LoadImage", {}), "21": ("ImageScaleToTotalPixels", {"image": "20"}), "22": ("GetImageSize", {"image": "21"}),
    "23": ("VAEEncode", {"pixels": "21", "vae": "3"}),
    "24": ("ReferenceLatent", {"conditioning": "4", "latent": "23"}),
    "25": ("ReferenceLatent", {"conditioning": "5", "latent": "23"}),
}
STYLE = ("Use the attached image (Mara Voss) only as the reference for pixel-art style: the same technique. Do not copy "
         "her face.\nPixel art character portrait, square 1:1 image.\nPlain flat very dark background.")


def wiring(g):
    return {k: (v["class_type"], {i: x[0] for i, x in v["inputs"].items() if isinstance(x, list)}) for k, v in g.items()}


class KleinGraphs(unittest.TestCase):
    def test_text_graph_is_the_verified_wiring(self):
        g = localgen.klein_graph("the prompt", seed=7, size=768, prefix="localgen/wren_loc1_1")
        self.assertEqual(wiring(g), PROBE_TEXT)
        self.assertEqual(g["4"]["inputs"]["text"], "the prompt")
        self.assertEqual(g["10"]["inputs"]["noise_seed"], 7)
        self.assertEqual((g["8"]["inputs"]["steps"], g["6"]["inputs"]["cfg"]), (4, 1.0))
        self.assertEqual((g["8"]["inputs"]["width"], g["9"]["inputs"]["height"]), (768, 768))
        self.assertEqual(g["2"]["inputs"], {"clip_name": "qwen_3_4b_fp4_flux2.safetensors", "type": "flux2", "device": "default"})
        self.assertEqual(g["1"]["inputs"]["unet_name"], "flux-2-klein-4b.safetensors")
        self.assertEqual(g["13"]["inputs"]["filename_prefix"], "localgen/wren_loc1_1")

    def test_edit_graph_is_the_verified_wiring(self):
        g = localgen.klein_edit_graph("localgen_wren.png", "Make her angry.", seed=2, megapixels=0.6, clip="c.safetensors")
        self.assertEqual(wiring(g), PROBE_EDIT)
        self.assertEqual(g["20"]["inputs"]["image"], "localgen_wren.png")
        self.assertEqual(g["21"]["inputs"]["megapixels"], 0.6)
        self.assertEqual(g["4"]["inputs"]["text"], "Make her angry.")
        self.assertEqual(g["8"]["inputs"]["height"], ["22", 1], "the height is GetImageSize's second output")
        self.assertEqual(g["2"]["inputs"]["clip_name"], "c.safetensors")

    def test_every_link_names_a_node_of_the_graph(self):
        for g in (localgen.klein_graph("p", seed=1), localgen.klein_edit_graph("x.png", "p", seed=1)):
            for node in g.values():
                for x in node["inputs"].values():
                    if isinstance(x, list):
                        self.assertIn(x[0], g)


class KleinPrompt(unittest.TestCase):
    def test_character_block_first_then_style_without_the_reference_line(self):
        prompt, cut = localgen.klein_prompt(BLOCK, STYLE)
        self.assertTrue(prompt.startswith("Character: Wren Ash, Lamp-Keeper"), prompt)
        self.assertTrue(prompt.endswith("Plain flat very dark background."), prompt)
        self.assertLess(prompt.index("Main colours"), prompt.index("Pixel art character portrait"))
        self.assertNotIn("attached", prompt)
        self.assertNotIn("Do not copy her face", prompt, "the whole reference line goes")
        self.assertEqual(len(cut), 1)
        self.assertTrue(cut[0].startswith("reference line: Use the attached image"))

    def test_nothing_of_the_block_is_cut(self):
        long = BLOCK + " " + " ".join(["A lantern hangs from the belt."] * 60)
        prompt, cut = localgen.klein_prompt(long, STYLE)
        self.assertIn(long.strip(), prompt)
        self.assertFalse([c for c in cut if not c.startswith("reference line")])

    def test_a_whole_recorded_prompt_reorders_to_character_first(self):
        whole = STYLE + "\n\n" + BLOCK
        self.assertEqual(localgen.klein_prompt(whole), localgen.klein_prompt(BLOCK, STYLE))

    def test_face_cells_are_the_portraitfx1_lines(self):
        spec = (generate.ROOT / "docs/specs/PORTRAITFX1_TASKS.md").read_text(encoding="utf-8")
        lines = dict(re.findall(r"^- [\w-]+: (Hurt|Angry|Afraid): (.*)\.$", spec, re.M))
        self.assertEqual({k.lower(): v for k, v in lines.items()}, localgen.CELLS)
        text = localgen.face_prompt("afraid")
        self.assertTrue(text.startswith("Keep the same character"), text)
        self.assertTrue(text.endswith(f"Change only the facial expression to afraid: {localgen.CELLS['afraid']}."))


class KleinRecord(unittest.TestCase):
    WHEN = "2026-10-26T00:00:00+00:00"

    def api_keys(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_krec_"))
        self.addCleanup(shutil.rmtree, tmp, True)
        return list(json.loads(generate.record(tmp / "a.png", "gemini-x", "wren", "p", False, None, 1,
                                               when=self.WHEN).read_text(encoding="utf-8")))

    def test_text_record_has_generate_keys_then_the_backend_settings(self):
        rec = localgen.klein_record("wren", "the prompt", 2, kind="text", seed=3, size=768, cut=["reference line: x"],
                                    prompts_file=localgen.ART3, seconds=80.5, when=self.WHEN)
        keys = self.api_keys()
        self.assertEqual(list(rec)[:len(keys)], keys, "generate.py's keys first, in its order")
        self.assertEqual(list(rec)[len(keys):], ["backend", "kind", "unet", "clip", "vae", "sampler", "steps", "cfg",
                                                 "size", "seed", "tokens_estimate", "cut", "seconds"])
        self.assertEqual((rec["model"], rec["reference"], rec["n"]), ("local:flux-2-klein-4b", None, 2))
        self.assertEqual((rec["steps"], rec["cfg"], rec["size"], rec["seed"]), (4, 1.0, 768, 3))
        self.assertEqual(rec["tokens_estimate"], localgen.estimate("the prompt"))
        self.assertEqual(rec["prompts_file"], "games/emberwake/docs/specs/ART3_PROMPTS.md")
        self.assertFalse(any("key" in k.lower() for k in rec), "never a key")

    def test_edit_and_face_records_name_the_source_as_reference(self):
        src = generate.SRC / "wren.png"
        rec = localgen.klein_record("wren", "Make her angry.", 1, kind="edit", seed=1, reference=src,
                                    megapixels=1.0, change="Make her angry.", when=self.WHEN)
        self.assertEqual((rec["reference"], rec["megapixels"], rec["change"]), ("art_src/wren.png", 1.0, "Make her angry."))
        self.assertNotIn("size", rec)
        face = localgen.klein_record("wren", localgen.face_prompt("hurt"), 1, kind="face", seed=1, reference=src,
                                     expression="hurt", when=self.WHEN)
        self.assertEqual((face["kind"], face["expression"], face["reference"]), ("face", "hurt", "art_src/wren.png"))
        self.assertNotIn("change", face)

    def test_face_take_name(self):
        self.assertEqual(localgen.face_path(generate.SRC, "tess", "angry", 1, 1792950000).name,
                         "tess_face_angry_loc1792950000_1.png")


class KleinDryRun(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_kdry_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)
        (self.tmp / "P.md").write_text(PROMPTS_MD, encoding="utf-8")

    def test_klein_is_the_default_backend(self):
        self.assertEqual(localgen.parser().parse_args(["run", "wren"]).backend, "comfyui")

    def test_run_dry_sends_the_whole_prompt_character_first(self):
        code, out = run("run", "wren", "--prompts", self.tmp / "P.md", "--n", "2", "--seed", "5", "--out",
                        self.tmp / "out", "--dry-run")
        self.assertEqual(code, 0)
        recs = records(out)
        self.assertEqual([(r["n"], r["seed"], r["model"]) for r in recs],
                         [(1, 5, "local:flux-2-klein-4b"), (2, 6, "local:flux-2-klein-4b")])
        self.assertEqual(recs[0]["prompt"], BLOCK + "\n\nShared style block.")
        self.assertFalse((self.tmp / "out").exists())

    def test_vary_dry_prompt_is_the_change(self):
        code, out = run("vary", "wren", "--from", self.tmp / "wren.png", "--change", "Make her angry.", "--dry-run")
        self.assertEqual(code, 0)
        (rec,) = records(out)
        self.assertEqual((rec["prompt"], rec["change"], rec["kind"]), ("Make her angry.", "Make her angry.", "edit"))
        self.assertEqual(rec["reference"], (self.tmp / "wren.png").resolve().as_posix())

    def test_faces_dry_has_one_take_per_expression(self):
        code, out = run("faces", "wren", "pell", "--exprs", "angry", "hurt", "--src", self.tmp, "--dry-run")
        self.assertEqual(code, 0)
        recs = records(out)
        self.assertEqual([(r["id"], r["expression"]) for r in recs],
                         [("wren", "angry"), ("wren", "hurt"), ("pell", "angry"), ("pell", "hurt")])
        self.assertIn("wren_face_angry_loc", out)
        self.assertEqual(recs[0]["prompt"], localgen.face_prompt("angry"))
        self.assertNotIn("keep", recs[0])

    def test_faces_keep_names_a_detail_per_id_in_the_keep_list(self):
        (self.tmp / "pell.json").write_text(json.dumps({"id": "pell", "keep": ["her brass monocle"]}), encoding="utf-8")
        code, out = run("faces", "wren", "pell", "--exprs", "hurt", "--src", self.tmp, "--keep",
                        "wren=his iron loupe over his right eye", "--keep", "the scarf", "--dry-run")
        self.assertEqual(code, 0)
        wren, pell = records(out)
        self.assertEqual(wren["keep"], ["his iron loupe over his right eye", "the scarf"])
        self.assertEqual(pell["keep"], ["her brass monocle", "the scarf"])
        self.assertIn("outfit, his iron loupe over his right eye, the scarf, pixel-art style", wren["prompt"])
        self.assertNotIn("loupe", pell["prompt"])
        self.assertTrue(wren["prompt"].endswith(f"to hurt: {localgen.CELLS['hurt']}."))

    def test_strength_and_faces_on_the_wrong_backend_are_refused(self):
        with self.assertRaises(SystemExit):
            run("vary", "wren", "--from", "x.png", "--change", "c", "--strength", "0.4", "--dry-run")
        with self.assertRaises(SystemExit):
            run("faces", "wren", *OV, "--dry-run")

    def test_prompt_prints_the_whole_klein_prompt(self):
        code, out = run("prompt", "wren", "--prompts", self.tmp / "P.md")
        self.assertEqual(code, 0)
        self.assertIn("(klein, whole", out)
        self.assertIn("Character: Wren Ash", out)


def png_bytes(color=(40, 20, 30)):
    from PIL import Image
    buf = io.BytesIO()
    Image.new("RGB", (8, 8), color).save(buf, "PNG")
    return buf.getvalue()


def washout_bytes():
    """The red/green wash-out in miniature: the left half pure red, the right half pure green."""
    from PIL import Image
    im = Image.new("RGB", (16, 16), (230, 20, 20))
    im.paste((20, 230, 20), (8, 0, 16, 16))
    buf = io.BytesIO()
    im.save(buf, "PNG")
    return buf.getvalue()


class BrokenTake(unittest.TestCase):
    def test_a_red_green_washout_is_broken_with_or_without_a_source(self):
        self.assertTrue(localgen.broken(washout_bytes()))
        self.assertTrue(localgen.broken(washout_bytes(), png_bytes()))

    def test_a_normal_take_is_not_broken(self):
        self.assertFalse(localgen.broken(png_bytes()))
        self.assertFalse(localgen.broken(png_bytes((48, 26, 34)), png_bytes()))

    def test_an_edit_whose_colours_moved_from_the_source_is_broken(self):
        shifted = png_bytes((40, 20 + localgen.BROKEN_SHIFT + 10, 30))
        self.assertTrue(localgen.broken(shifted, png_bytes()))
        self.assertFalse(localgen.broken(shifted), "a text take has no source to compare")


class FakeComfy:
    """The ComfyUI HTTP API in miniature: the calls a run makes, each answered as the server answers."""

    def __init__(self, fail=(), views=()):
        self.calls, self.graphs, self.polls, self.fail = [], [], {}, set(fail)
        self.png = png_bytes()
        self.views = list(views)  # PNGs /view answers first (a broken take), then self.png

    def __call__(self, req, timeout=None):
        url = urllib.parse.urlsplit(req.full_url)
        self.calls.append((url.path, req.data, req.headers))
        if url.path == "/system_stats":
            body = {"system": {"comfyui_version": "0.42.5"}}
        elif url.path == "/upload/image":
            name = re.search(rb'filename="([^"]+)"', req.data).group(1).decode()
            body = {"name": name, "subfolder": "", "type": "input"}
        elif url.path == "/prompt":
            self.graphs.append(json.loads(req.data)["prompt"])
            body = {"prompt_id": f"p{len(self.graphs)}", "number": len(self.graphs)}
        elif url.path.startswith("/history/"):
            pid = url.path.rsplit("/", 1)[1]
            self.polls[pid] = self.polls.get(pid, 0) + 1
            if self.polls[pid] == 1:
                body = {}  # still queued at the first poll
            elif pid in self.fail:
                body = {pid: {"status": {"status_str": "error", "messages": [
                    ["execution_error", {"exception_message": "out of memory"}]]}, "outputs": {}}}
            else:
                prefix = self.graphs[int(pid[1:]) - 1]["13"]["inputs"]["filename_prefix"]
                body = {pid: {"status": {"status_str": "success", "messages": [
                    ["execution_start", {"timestamp": 1000}], ["execution_success", {"timestamp": 91500}]]},
                    "outputs": {"13": {"images": [{"filename": prefix.split("/")[-1] + "_00001_.png",
                                                   "subfolder": "localgen", "type": "output"}]}}}}
        elif url.path == "/view":
            return io.BytesIO(self.views.pop(0) if self.views else self.png)
        else:
            raise AssertionError(url.path)
        return io.BytesIO(json.dumps(body).encode())


class KleinOverHttp(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="localgen_http_"))
        self.addCleanup(shutil.rmtree, self.tmp, True)
        (self.tmp / "src").mkdir()
        (self.tmp / "src" / "wren.png").write_bytes(png_bytes())
        self.fake = FakeComfy()

    def go(self, *argv, fake=None):
        fake = fake or self.fake
        with mock.patch("urllib.request.urlopen", fake), mock.patch.object(localgen, "COMFY", self.tmp / "comfyui"), \
                mock.patch.object(localgen.time, "sleep", lambda s: None):
            return run(*argv, "--server", "http://fake:8188", "--out", self.tmp / "out")

    def test_faces_upload_queue_collect_and_record(self):
        code, out = self.go("faces", "wren", "--src", self.tmp / "src", "--seed", "4")
        self.assertEqual(code, 0, out)
        paths = [c[0] for c in self.fake.calls]
        self.assertEqual(paths[0], "/system_stats", "the server is checked first")
        self.assertEqual(paths.count("/upload/image"), 1, "one upload per source")
        self.assertEqual([g["20"]["inputs"]["image"] for g in self.fake.graphs], ["localgen_wren.png"] * 3)
        self.assertEqual([g["4"]["inputs"]["text"] for g in self.fake.graphs],
                         [localgen.face_prompt(e) for e in ("hurt", "angry", "afraid")])
        self.assertLess(max(i for i, p in enumerate(paths) if p == "/prompt"),
                        min(i for i, p in enumerate(paths) if p.startswith("/history/")), "all queued, then collected")
        made = sorted((self.tmp / "out").glob("wren_face_*_loc*_1.png"))
        self.assertEqual(sorted(p.name.split("_")[2] for p in made), ["afraid", "angry", "hurt"])
        rec = json.loads(made[0].with_suffix(".json").read_text(encoding="utf-8"))
        self.assertEqual((rec["model"], rec["kind"], rec["seed"], rec["seconds"]), ("local:flux-2-klein-4b", "face", 4, 90.5))
        self.assertEqual(rec["reference"], (self.tmp / "src" / "wren.png").resolve().as_posix())
        self.assertEqual(made[0].read_bytes(), self.fake.png)

    def test_upload_is_a_multipart_image_to_the_input_folder(self):
        self.go("vary", "wren", "--from", self.tmp / "src" / "wren.png", "--change", "Make her angry.")
        _, data, headers = next(c for c in self.fake.calls if c[0] == "/upload/image")
        self.assertTrue(headers["Content-type"].startswith("multipart/form-data; boundary="))
        self.assertIn(b'name="type"\r\n\r\ninput', data)
        self.assertIn(b'name="overwrite"\r\n\r\ntrue', data)
        self.assertIn(png_bytes(), data)
        (g,) = self.fake.graphs
        self.assertEqual(wiring(g), PROBE_EDIT)

    def test_run_queues_the_text_graph_with_the_whole_prompt(self):
        (self.tmp / "P.md").write_text(PROMPTS_MD, encoding="utf-8")
        code, out = self.go("run", "wren", "--prompts", self.tmp / "P.md", "--n", "2", "--size", "768")
        self.assertEqual(code, 0, out)
        self.assertEqual([wiring(g) for g in self.fake.graphs], [PROBE_TEXT] * 2)
        self.assertEqual([g["10"]["inputs"]["noise_seed"] for g in self.fake.graphs], [1, 2])
        self.assertEqual(self.fake.graphs[0]["4"]["inputs"]["text"], BLOCK + "\n\nShared style block.")
        self.assertEqual(len(list((self.tmp / "out").glob("wren_loc*_*.png"))), 2)
        self.assertFalse([c for c in self.fake.calls if c[0] == "/upload/image"], "a text run uploads nothing")

    def test_a_broken_take_is_retried_with_the_next_seed(self):
        fake = FakeComfy(views=[washout_bytes()])
        code, out = self.go("faces", "wren", "--exprs", "hurt", "--src", self.tmp / "src", "--seed", "4", fake=fake)
        self.assertEqual(code, 0, out)
        self.assertIn("retried with seed 5", out)
        self.assertEqual([g["10"]["inputs"]["noise_seed"] for g in fake.graphs], [4, 5])
        (made,) = (self.tmp / "out").glob("wren_face_hurt_loc*_1.png")
        self.assertEqual(made.read_bytes(), fake.png)
        self.assertEqual(json.loads(made.with_suffix(".json").read_text(encoding="utf-8"))["seed"], 5)

    def test_a_take_broken_three_times_is_reported_and_not_saved(self):
        (self.tmp / "P.md").write_text(PROMPTS_MD, encoding="utf-8")
        fake = FakeComfy(views=[washout_bytes()] * 3)
        code, out = self.go("run", "wren", "--prompts", self.tmp / "P.md", "--seed", "7", fake=fake)
        self.assertEqual(code, 1)
        self.assertIn("FAILED broken (red/green wash-out) with seeds 7-9", out)
        self.assertEqual([g["10"]["inputs"]["noise_seed"] for g in fake.graphs], [7, 8, 9])
        self.assertFalse(list((self.tmp / "out").glob("*.png")))

    def test_a_failed_job_is_reported_and_the_rest_kept(self):
        fake = FakeComfy(fail={"p2"})
        code, out = self.go("faces", "wren", "--src", self.tmp / "src", fake=fake)
        self.assertEqual(code, 1)
        self.assertIn("FAILED error: ", out)
        self.assertIn("out of memory", out)
        self.assertEqual(len(list((self.tmp / "out").glob("*.png"))), 2)

    def test_no_server_says_how_to_start_it(self):
        def down(req, timeout=None):
            raise urllib.error.URLError(ConnectionRefusedError(10061, "refused"))
        with self.assertRaises(SystemExit) as e:
            self.go("faces", "wren", "--src", self.tmp / "src", fake=down)
        self.assertIn("ComfyUI is not running at http://fake:8188", str(e.exception))
        self.assertIn("comfyui.bat", str(e.exception))

    def test_a_rejected_graph_shows_the_servers_reason(self):
        def reject(req, timeout=None):
            raise urllib.error.HTTPError(req.full_url, 400, "Bad Request", {}, io.BytesIO(b'{"error": "bad node"}'))
        client = localgen.Comfy("http://fake:8188", opener=reject)
        with self.assertRaises(SystemExit) as e:
            client.queue({})
        self.assertIn("bad node", str(e.exception))

    def test_the_image_comes_from_the_output_folder_when_it_is_there(self):
        home = self.tmp / "comfyui"
        (home / "output" / "localgen").mkdir(parents=True)
        (home / "output" / "localgen" / "a.png").write_bytes(b"local")
        client = localgen.Comfy("http://fake:8188", home=home, opener=self.fake)
        entry = {"outputs": {"13": {"images": [{"filename": "a.png", "subfolder": "localgen", "type": "output"}]}}}
        self.assertEqual(client.image(entry), b"local")
        entry["outputs"]["13"]["images"][0]["filename"] = "b.png"
        self.assertEqual(client.image(entry), self.fake.png, "else through /view")


if __name__ == "__main__":
    unittest.main()

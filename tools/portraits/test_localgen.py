# Tests of localgen.py (LOCALGEN1): the prompt record's shape and the short prompt for 77-token text encoders, with
# no model and no OpenVINO (the token count is the tool's estimate, or the model's own CLIP tokenizer files when they
# are there), plus the --dry-run smoke. Run from anywhere with the plain Python (Pillow only):
#   python tools/portraits/test_localgen.py
import contextlib
import io
import json
import pathlib
import shutil
import sys
import tempfile
import unittest

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
            code, out = run("prompt", "wren", "--prompts", tmp / "P.md", "--model", model)
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
        code, out = run("run", "wren", "--prompts", self.tmp / "P.md", "--n", "2", "--seed", "5", "--device", "gpu",
                        "--out", self.tmp / "out", "--dry-run")
        self.assertEqual(code, 0)
        recs = records(out)
        self.assertEqual([(r["n"], r["seed"], r["device"]) for r in recs], [(1, 5, "GPU"), (2, 6, "GPU")])
        self.assertTrue(recs[0]["prompt"].startswith(localgen.LEAD))
        self.assertFalse((self.tmp / "out").exists())

    def test_vary_dry_names_each_strength(self):
        code, out = run("vary", "wren", "--from", self.tmp / "wren.png", "--prompts", self.tmp / "P.md",
                        "--change", "hurt, wincing", "--strength", "0.3", "--strength", "0.5", "--dry-run")
        self.assertEqual(code, 0)
        recs = records(out)
        self.assertEqual([r["strength"] for r in recs], [0.3, 0.5])
        self.assertTrue(all(r["prompt"].startswith("hurt, wincing, ") for r in recs))

    def test_prompt_lists_the_cut(self):
        code, out = run("prompt", "wren", "--prompts", self.tmp / "P.md")
        self.assertEqual(code, 0)
        self.assertIn("cut: name: Wren Ash", out)


if __name__ == "__main__":
    unittest.main()

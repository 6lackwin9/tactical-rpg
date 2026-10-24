# Lorebook (2026-10-25): renders a game's design library (games/<id>/docs/lore/) as one Worldbook page, the story
# guide's view for review on any device: the story, belief against truth, the acts, the cast by group (who's who and
# the cast registry), places, story elements, the reveal timeline, the glossary and the open story gaps. Every entry
# carries a note box; the published page keeps notes in its `db` collection "notes" (one document per entry key, such
# as `character~kaelen`), which the planner reads with ArtifactData. Game-agnostic: it reads only the library's files.
# Usage: python tools/lorebook.py --game emberwake --out <file.html>   (from the repo root)
import argparse, html, pathlib, re

ROOT = pathlib.Path(__file__).resolve().parents[1]


def esc(s):
    return html.escape(s, quote=True)


def inline(s):
    """Markdown inline to HTML: links become their text, `code`, **bold**, *em*."""
    s = esc(s)
    s = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", s)
    s = re.sub(r"`([^`]+)`", r"<code>\1</code>", s)
    s = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", s)
    s = re.sub(r"(?<![*\w])\*([^*]+)\*(?!\w)", r"<em>\1</em>", s)
    return s


def slug(s):
    return re.sub(r"[^a-z0-9]+", "_", re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", s).lower()).strip("_")[:48] or "x"


def cells(line):
    return [c.strip() for c in line.strip().strip("|").split(" | ")]


def sections(text, level=2):
    """{heading: body} for one heading level, in order."""
    out, head, buf = {}, None, []
    mark = "#" * level + " "
    for line in text.split("\n"):
        if line.startswith(mark):
            if head is not None:
                out[head] = "\n".join(buf).strip()
            head, buf = line[len(mark):].strip(), []
        elif head is not None:
            buf.append(line)
    if head is not None:
        out[head] = "\n".join(buf).strip()
    return out


def find(secs, *words):
    for k, v in secs.items():
        if all(w.lower() in k.lower() for w in words):
            return k, v
    return None, ""


def table(body):
    """(header, rows) of the first table in a body."""
    lines = [l for l in body.split("\n") if l.startswith("|")]
    if len(lines) < 2:
        return [], []
    return cells(lines[0]), [cells(l) for l in lines[2:]]


def blocks(body):
    """Markdown blocks to HTML: paragraphs, bullet and numbered lists (with indented continuations), tables, h3/h4."""
    out, para, items, kind = [], [], [], None
    lines = body.split("\n")

    def flush():
        nonlocal para, items, kind
        if para:
            out.append("<p>" + inline(" ".join(para)) + "</p>")
        if items:
            tag = "ol" if kind == "ol" else "ul"
            out.append(f"<{tag}>" + "".join("<li>" + inline(i) + "</li>" for i in items) + f"</{tag}>")
        para, items, kind = [], [], None

    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("|"):
            flush()
            tl = []
            while i < len(lines) and lines[i].startswith("|"):
                tl.append(lines[i]); i += 1
            head, rows = cells(tl[0]), [cells(l) for l in tl[2:]]
            out.append('<div class="tablewrap"><table><thead><tr>' + "".join(f"<th>{inline(h)}</th>" for h in head)
                       + "</tr></thead><tbody>" + "".join("<tr>" + "".join(f"<td>{inline(c)}</td>" for c in r) + "</tr>" for r in rows)
                       + "</tbody></table></div>")
            continue
        m_b, m_n = re.match(r"^- (.*)", line), re.match(r"^\d+\. (.*)", line)
        if line.startswith("### ") or line.startswith("#### "):
            flush(); out.append("<h4>" + inline(line.lstrip("#").strip()) + "</h4>")
        elif m_b or m_n:
            k = "ul" if m_b else "ol"
            if kind and kind != k:
                flush()
            if para:
                flush()
            kind = k; items.append((m_b or m_n).group(1))
        elif items and line.startswith("  ") and line.strip():
            items[-1] += " " + line.strip()
        elif not line.strip() or line.startswith("```"):
            flush()
        else:
            if items:
                flush()
            para.append(line.strip())
        i += 1
    flush()
    return "\n".join(out)


def note(key, label="Note"):
    return (f'<div class="slot" data-key="{esc(key)}"><textarea class="note" rows="1" placeholder="{esc(label)}: a change you want" '
            f'aria-label="{esc(label)} for {esc(key)}"></textarea><span class="saved"></span></div>')


def bullets_with_titles(body):
    """Top-level bullets '- **Title.** text' as (title, text)."""
    out = []
    for item in re.findall(r"^- (.*(?:\n  .*)*)", body, flags=re.M):
        item = " ".join(l.strip() for l in item.split("\n"))
        m = re.match(r"\*\*(.+?)\*\*\s*(.*)", item)
        out.append((m.group(1).rstrip("."), m.group(2)) if m else ("", item))
    return out


SIGIL = {"founder": "--scarlet", "hub": "--amber", "regime": "--rust", "guild": "--leather", "neutral": "--violet", "other": "--steel"}


def group_kind(title):
    t = title.lower()
    for k, w in (("founder", "founder"), ("hub", "hub"), ("regime", "regime"), ("guild", "guild"), ("neutral", "neutral")):
        if w in t:
            return k
    return "other"


def build(game):
    lore = ROOT / "games" / game / "docs" / "lore"
    read = lambda p: (lore / p).read_text(encoding="utf-8") if (lore / p).exists() else ""
    guide, who, reg, canon = read("STORY_GUIDE.md"), read("background/who_is_who.md"), read("background/cast_registry.md"), read("CANON.md")
    title = re.search(r"^# (.*)", guide, flags=re.M).group(1)
    title = re.sub(r"^Story guide:\s*", "", title)
    g = sections(guide)
    status = re.search(r"^Status: (.*?)(?:\.|;)", guide, flags=re.M)
    status = status.group(1) if status else ""

    # registry rows by first name: age and background
    regrow = {}
    for k, body in sections(reg).items():
        head, rows = table(body)
        if "Age" in head:
            for r in rows:
                d = dict(zip(head, r))
                regrow[slug(d["Name"].split(" (")[0])] = d

    parts = []
    nav = []

    def section(sid, name, inner, eyebrow=""):
        nav.append((sid, name))
        eb = f'<div class="eyebrow">{esc(eyebrow)}</div>' if eyebrow else ""
        parts.append(f'<section class="world" id="{sid}"><div class="world-head">{eb}<h2>{esc(name)}</h2></div>{inner}</section>')

    # Belief against truth
    k, body = find(g, "believes")
    if k:
        section("truth", "Belief and truth", f'<div class="prose spoiler">{blocks(body)}{note("section~truth")}</div>', "Spoilers")

    # Acts
    k, body = find(g, "acts")
    if k:
        cards = "".join(f'<article class="act"><h3>{inline(t)}</h3><p>{inline(x)}</p>{note("act~" + slug(t))}</article>'
                        for t, x in bullets_with_titles(body))
        section("acts", "The acts", f'<div class="acts">{cards}</div>', "Prologue to the Dreaming Core")

    # Cast
    who_secs = sections(who)
    trace_k, trace = find(sections(guide, 3), "traced")
    cast = []
    for gtitle, body in who_secs.items():
        head, rows = table(body)
        if not rows:
            continue
        kind = group_kind(gtitle)
        cards = []
        for r in rows:
            d = dict(zip(head, r))
            name = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", d.get("Name", ""))
            key = slug(name.split(" (")[0])
            first = key.split("_")[-1] if key not in regrow else key
            rg = regrow.get(key) or next((v for kk, v in regrow.items() if kk.endswith(first) or first.endswith(kk)), {})
            facts = " · ".join(x for x in [("age " + rg["Age"]) if rg.get("Age") and rg["Age"] != "-" else "", d.get("First seen", "")] if x and x != "-")
            acts = "".join(f"<dt>{esc(c)}</dt><dd>{inline(d[c])}</dd>" for c in head if c.startswith(("Prologue", "Act")) and d.get(c, "-") not in ("-", ""))
            initial = esc(name.strip("'\" ")[:1] or "?")
            extra = f'<div class="trace">{blocks(trace)}</div>' if trace and name.split()[-1:] == ["Nemeth"] else ""
            cards.append(
                f'<article class="char" id="c-{key}"><div class="sigil" style="--c:var({SIGIL[kind]})" aria-hidden="true">{initial}</div>'
                f'<div class="body"><h4>{inline(name)}</h4><p class="title">{inline(d.get("Faction and role", ""))}</p>'
                + (f'<p class="facts">{inline(facts)}</p>' if facts else "")
                + (f'<p class="bg">{inline(rg["Background (one line)"])}</p>' if rg.get("Background (one line)") else "")
                + (f'<p class="pers"><strong>Wants:</strong> {inline(d["Wants"])}</p>' if d.get("Wants", "-") != "-" else "")
                + (f"<dl>{acts}</dl>" if acts else "")
                + f'<p class="fate"><strong>Fate:</strong> {inline(d.get("Fate or open", "-"))}</p>{extra}{note("character~" + key)}</div></article>')
        cast.append(f'<h3>{inline(re.sub(r"^[0-9.]+ ", "", gtitle))} <span>{len(rows)}</span></h3><div class="chars">{"".join(cards)}</div>')
    if cast:
        section("cast", "The cast", "".join(cast), "Who's who by act")

    # Places
    k, body = find(sections(reg), "places")
    head, rows = table(body)
    if rows:
        cards = "".join(f'<article class="mini"><h4>{inline(r[0])}</h4><p>{inline(r[1])}</p>'
                        + (f'<p class="facts">Formerly: {inline(r[2])}</p>' if len(r) > 2 and r[2] != "-" else "")
                        + note("place~" + slug(r[0])) + "</article>" for r in rows)
        section("places", "Places", f'<div class="minis">{cards}</div>', "Cinderreach and below")

    # Story elements
    k, body = find(g, "elements")
    head, rows = table(body)
    if rows:
        cards = []
        for r in rows:
            d = dict(zip(head, r))
            dl = "".join(f"<dt>{esc(c)}</dt><dd>{inline(d[c])}</dd>" for c in head[2:] if d.get(c, "-") != "-")
            cards.append(f'<article class="mini"><h4>{inline(r[0])}</h4><p>{inline(r[1])}</p><dl>{dl}</dl>{note("element~" + slug(r[0]))}</article>')
        section("elements", "Story elements", f'<div class="minis">{"".join(cards)}</div>', "Set up and paid off")

    # Reveal timeline
    k, body = find(g, "reveal")
    head, rows = table(body)
    if rows:
        items = "".join(f'<li><span class="when">{inline(r[0])}</span><span>{inline(r[1])}</span></li>' for r in rows)
        section("reveals", "Reveal timeline", f'<ol class="timeline">{items}</ol>{note("section~reveals")}', "What the player learns, and when")

    # Glossary (the canon sheet)
    k, body = find(sections(canon), "glossary")
    head, rows = table(body)
    if rows:
        items = "".join(f"<div><dt>{inline(r[0])}</dt><dd>{inline(r[1])}</dd></div>" for r in rows)
        section("glossary", "Glossary", f'<dl class="gloss">{items}</dl>', "One name per thing")

    # Gaps
    k, body = find(g, "gaps")
    head, rows = table(body)
    open_gaps = []
    if rows:
        out = []
        for r in rows:
            d = dict(zip(head, r))
            text = " ".join(r)
            done = "Resolved" in text or "resolved" in d.get("Rec.", "")
            if not done:
                open_gaps.append(d)
            opts = d.get("Options (new only)", "-")
            out.append(f'<li class="gap{" done" if done else ""}" id="g-{slug(r[0])}"><div class="ghead"><span class="gid">{esc(r[0])}</span>'
                       f'<span class="chip">{"resolved" if done else esc(d.get("Open Q or new", ""))}</span></div><p>{inline(r[1])}</p>'
                       + (f'<p class="facts">Options: {inline(opts)}</p>' if opts not in ("-", "") else "")
                       + (f'<p class="facts">Rec.: {inline(d.get("Rec.", ""))}</p>' if d.get("Rec.", "-") not in ("-", "") else "")
                       + note("gap~" + slug(r[0]), "Answer") + "</li>")
        section("gaps", "Open threads", f'<p class="lede">{len(open_gaps)} open of {len(rows)}. Answer here or in chat; the planner turns answers into canon.</p><ol class="gaps">{"".join(out)}</ol>', "Story gaps")

    # Story (first) and its side panel
    k, story = find(g, "one page")
    threads = "".join(f'<li><a href="#g-{slug(d["Id"])}">{esc(d["Id"])}</a> {inline(d["Gap"])}</li>' for d in open_gaps[:10])
    story_html = (f'<div class="story"><div class="col"><h2>The story</h2>{blocks(story)}{note("section~story")}</div>'
                  f'<aside class="threads"><h2>Open threads</h2><ul>{threads}</ul><p class="facts"><a href="#gaps">All {len(open_gaps)} open threads</a></p></aside></div>')
    navhtml = "".join(f'<a href="#{s}">{esc(n)}</a>' for s, n in nav)
    return TEMPLATE.replace("{{TITLE}}", esc(title)).replace("{{GAME}}", esc(game)).replace("{{STATUS}}", inline(status)) \
        .replace("{{STORY}}", story_html).replace("{{NAV}}", navhtml).replace("{{SECTIONS}}", "\n".join(parts))


TEMPLATE = (pathlib.Path(__file__).with_name("lorebook_template.html")).read_text(encoding="utf-8") if pathlib.Path(__file__).with_name("lorebook_template.html").exists() else ""

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--game", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    pathlib.Path(a.out).write_text(build(a.game), encoding="utf-8")
    print("wrote", a.out)

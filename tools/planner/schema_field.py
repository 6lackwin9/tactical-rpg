#!/usr/bin/env python3
"""TOOLS-HELP1 (ENGINE-15): `tools/fc.sh schema-field <table> <field> <json> [--after <key>] [--replace] [--file F]`.

Adds a field to a table of the kit's field tables, the one-line `kits/rpg/schema.json`
({"content": {kind: [field]}, "world": {kind: [field]}}), and writes the file back in its own form byte for byte
(compact, no trailing newline), so the diff is the new field alone. <table> is `content.<kind>` or `world.<kind>`, or a
bare kind when only one section has it. <json> is the field object (`{"label": ..., "type": ...}`); its `key` is
<field> (given or not). The field goes last, or right after the field `--after <key>`; an existing <field> is refused
unless `--replace` (it then keeps its place). Prints what it did; changes nothing on any refusal.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = ROOT / "kits" / "rpg" / "schema.json"


def dump(d):
    """The schema file's own form (serde_json's compact output: `,` and `:`, UTF-8, no newline)."""
    return json.dumps(d, separators=(",", ":"), ensure_ascii=False)


def table_of(d, name):
    if "." in name:
        sec, kind = name.split(".", 1)
        if sec in d and kind in d[sec]:
            return sec, kind
        raise ValueError("no table %s" % name)
    hits = [sec for sec in d if isinstance(d[sec], dict) and name in d[sec]]
    if len(hits) == 1:
        return hits[0], name
    if not hits:
        raise ValueError("no table %s (content: %s; world: %s)" % (name, ", ".join(d.get("content", {})), ", ".join(d.get("world", {}))))
    raise ValueError("%s is in %s: name it as %s" % (name, " and ".join(hits), " or ".join("%s.%s" % (h, name) for h in hits)))


def add_field(text, table, key, field, after=None, replace=False):
    """The new file text and a line on what changed."""
    d = json.loads(text)
    if dump(d) != text:
        raise ValueError("the schema file is not in its one-line form; refusing to rewrite it")
    sec, kind = table_of(d, table)
    fields = d[sec][kind]
    field = dict(field)
    if field.get("key", key) != key:
        raise ValueError("the json's key %r is not the field %r" % (field["key"], key))
    field = {"key": key, **{k: v for k, v in field.items() if k != "key"}}
    at = next((i for i, f in enumerate(fields) if f.get("key") == key), None)
    if at is not None:
        if not replace:
            raise ValueError("%s.%s already has a field %s (--replace to change it)" % (sec, kind, key))
        fields[at] = field
        how = "replaced"
    else:
        if after is None:
            fields.append(field)
        else:
            i = next((i for i, f in enumerate(fields) if f.get("key") == after), None)
            if i is None:
                raise ValueError("%s.%s has no field %s to go after" % (sec, kind, after))
            fields.insert(i + 1, field)
        how = "added"
    return dump(d), "%s field %s in %s.%s (%d fields)" % (how, key, sec, kind, len(fields))


def main(argv):
    args, after, replace, path = [], None, False, SCHEMA
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--after":
            after, i = argv[i + 1], i + 2
        elif a == "--file":
            path, i = Path(argv[i + 1]), i + 2
        elif a == "--replace":
            replace, i = True, i + 1
        else:
            args.append(a)
            i += 1
    if len(args) != 3:
        print(__doc__)
        return 2
    try:
        field = json.loads(args[2])
        if not isinstance(field, dict):
            raise ValueError("the field is a JSON object")
        text, line = add_field(path.read_text(encoding="utf-8"), args[0], args[1], field, after, replace)
    except ValueError as e:
        print("schema-field: %s" % e)
        return 1
    path.write_text(text, encoding="utf-8")
    print("schema-field: %s; %s" % (line, path.relative_to(ROOT) if path.is_relative_to(ROOT) else path))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

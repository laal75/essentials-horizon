#!/usr/bin/env python3
"""Build the Essentials install package (essentials.hnmod) from the bundle.

A .hnmod is a zip holding the manifest (horizon-native.json) and the bundle
(mixins/<id>/...). This reads the id, version and Horizon pin from the bundle's
horizon-mixin.toml, writes the manifest the launcher expects, and zips the two
together. No dependencies beyond the Python 3 standard library.

    python3 tools/package.py [-o OUT]
"""

import argparse
import json
import os
import re
import sys
import zipfile

# The launcher's constants (launcher/src/native_mods.rs): the default mixins
# revision and the manifest's schema. The game and the native-hook API version
# match the Horizon commit the bundle pins.
GAME = "26.3"
MIXINS_REV = "a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab"
HOOKS = 80
SCHEMA = 1


def read_field(toml_text, section, key):
    """A small line-based reader for the two [mod] and [compatibility] keys we
    need — no TOML dependency. `key = "value"` under the named section."""
    lines = toml_text.splitlines()
    in_section = False
    for line in lines:
        s = line.strip()
        if s.startswith("[") and s.endswith("]"):
            in_section = s == "[%s]" % section
            continue
        if in_section:
            m = re.match(r'%s\s*=\s*"([^"]*)"' % re.escape(key), s)
            if m:
                return m.group(1)
    return None


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "-o", "--out", default=None, help="output .hnmod path (default: essentials.hnmod)"
    )
    args = ap.parse_args()

    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    bundle_dir = os.path.join(root, "mixins")
    toml_path = os.path.join(bundle_dir, "essentials", "horizon-mixin.toml")

    with open(toml_path, "r", encoding="utf-8") as f:
        toml_text = f.read()

    mod_id = read_field(toml_text, "mod", "id")
    version = read_field(toml_text, "mod", "version")
    horizon = read_field(toml_text, "compatibility", "horizon_revision")

    problems = []
    if not mod_id:
        problems.append("no [mod] id in %s" % toml_path)
    if not version:
        problems.append("no [mod] version in %s" % toml_path)
    if not horizon:
        problems.append("no [compatibility] horizon_revision in %s" % toml_path)
    if problems:
        for p in problems:
            print("error:", p, file=sys.stderr)
        return 1

    def is_rev(v):
        return v is not None and len(v) == 40 and all(c in "0123456789abcdef" for c in v)

    if not is_rev(horizon):
        print("error: horizon_revision is not a 40-hex commit: %r" % horizon, file=sys.stderr)
        return 1
    if not is_rev(MIXINS_REV):
        print("error: MIXINS_REV is not a 40-hex revision", file=sys.stderr)
        return 1

    manifest = {
        "schema": SCHEMA,
        "id": mod_id,
        "version": version,
        "game": GAME,
        "horizon": horizon,
        "mixins": MIXINS_REV,
        "hooks": HOOKS,
    }

    # The bundle directory that goes into the package: mixins/<id>/
    bundle_src = os.path.join(bundle_dir, mod_id)
    if not os.path.isdir(bundle_src):
        print("error: bundle dir not found: %s" % bundle_src, file=sys.stderr)
        return 1

    out = args.out or os.path.join(root, "essentials.hnmod")

    # Collect the bundle files (deterministic order).
    entries = []
    for dirpath, dirnames, filenames in os.walk(bundle_src):
        dirnames.sort()
        for name in sorted(filenames):
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full, bundle_src).replace(os.sep, "/")
            entries.append((full, "mixins/%s/%s" % (mod_id, rel)))

    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("horizon-native.json", json.dumps(manifest, indent=2) + "\n")
        for full, arcname in entries:
            z.write(full, arcname)

    print("wrote %s" % out)
    print("  manifest:", json.dumps(manifest))
    print("  files:   %d bundle file(s) under mixins/%s/" % (len(entries), mod_id))
    return 0


if __name__ == "__main__":
    sys.exit(main())


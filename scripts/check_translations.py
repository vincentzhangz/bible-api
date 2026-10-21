#!/usr/bin/env python3
"""Check that every book in a translation file can be placed in the canon.

The API matches books by name. A translation may call a book something else —
"I Samuel" for "1 Samuel", "Kejadian" for "Genesis" — and it may carry
apocryphal works that are not part of the canon at all. This script reports
both, so a new translation can be checked before it is published.

Run it from the repository root:

    python3 scripts/check_translations.py
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRANSLATION_DIR = ROOT / "data" / "translations"
CANONICAL_FILE = ROOT / "src" / "canonical.rs"

# Accented Latin letters, so that "Génesis" and "Genesis" land on the same key.
FOLDING = {
    "à": "a", "á": "a", "â": "a", "ã": "a", "ä": "a", "å": "a", "ā": "a",
    "ă": "a", "ą": "a", "ç": "c", "ć": "c", "č": "c", "ď": "d", "đ": "d",
    "è": "e", "é": "e", "ê": "e", "ë": "e", "ē": "e", "ĕ": "e", "ė": "e",
    "ę": "e", "ě": "e", "ğ": "g", "ĝ": "g", "ġ": "g", "ģ": "g", "ĥ": "h",
    "ħ": "h", "ì": "i", "í": "i", "î": "i", "ï": "i", "ĩ": "i", "ī": "i",
    "ĭ": "i", "į": "i", "ı": "i", "ĵ": "j", "ķ": "k", "ĺ": "l", "ļ": "l",
    "ľ": "l", "ł": "l", "ñ": "n", "ń": "n", "ņ": "n", "ň": "n", "ò": "o",
    "ó": "o", "ô": "o", "õ": "o", "ö": "o", "ø": "o", "ō": "o", "ŏ": "o",
    "ő": "o", "ŕ": "r", "ŗ": "r", "ř": "r", "ś": "s", "ŝ": "s", "ş": "s",
    "š": "s", "ș": "s", "ß": "ss", "ţ": "t", "ť": "t", "ŧ": "t", "ț": "t",
    "ù": "u", "ú": "u", "û": "u", "ü": "u", "ũ": "u", "ū": "u", "ŭ": "u",
    "ů": "u", "ű": "u", "ų": "u", "ŵ": "w", "ý": "y", "ÿ": "y", "ŷ": "y",
    "ź": "z", "ż": "z", "ž": "z",
}


def normalize(name: str) -> str:
    folded = [FOLDING.get(c, c) for c in name.lower()]
    return "".join(c for c in folded if c.isalnum())


def canonical_entries() -> dict[str, str]:
    """Every name and alias the Rust source knows about."""
    source = CANONICAL_FILE.read_text(encoding="utf-8")
    entries: dict[str, str] = {}

    for block in re.findall(r"CanonicalBook \{(.*?)\n    \}", source, re.S):
        name = re.search(r'name: "([^"]+)"', block)
        if not name:
            continue

        alias_block = block.split("aliases: &[", 1)[-1].split("]", 1)[0]
        for value in [name.group(1), *re.findall(r'"([^"]+)"', alias_block)]:
            entries[normalize(value)] = name.group(1)

    return entries


def main() -> int:
    entries = canonical_entries()
    print(f"{len(entries)} names and aliases are known\n")

    problems = 0

    for path in sorted(TRANSLATION_DIR.glob("*.json")):
        data = json.loads(path.read_text(encoding="utf-8"))
        matched: dict[str, int] = {}
        unknown: list[str] = []

        for book in data.get("books", []):
            name = book.get("name", "")
            key = normalize(name)
            canonical_name = entries.get(key)

            if canonical_name is None:
                unknown.append(name)
                continue

            if canonical_name in matched:
                print(
                    f"{path.name}: {name} and "
                    f"{matched[canonical_name]} are both {canonical_name}"
                )
                problems += 1
            matched[canonical_name] = name

        missing = 66 - len(matched)
        status = "ok" if missing == 0 else f"{missing} canonical books missing"

        print(f"{path.name:28s} {len(matched):2d} books placed, {status}")

        if unknown:
            print(f"{'':28s} not in the canon: {', '.join(unknown)}")

    return 1 if problems else 0


if __name__ == "__main__":
    raise SystemExit(main())

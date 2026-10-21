"""Report which book names in the translation files cannot be matched."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRANSLATION_DIR = ROOT / "data" / "translations"
CANONICAL_FILE = ROOT / "src" / "canonical.rs"

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


def canonical_entries():
    source = CANONICAL_FILE.read_text(encoding="utf-8")
    entries = {}
    for block in re.findall(r"CanonicalBook \{(.*?)\n    \}", source, re.S):
        name = re.search(r'name: "([^"]+)"', block)
        if not name:
            continue
        aliases = re.findall(r'"([^"]+)"', block.split("aliases: &[", 1)[-1])
        for value in [name.group(1), *aliases]:
            entries[normalize(value)] = name.group(1)
    return entries


entries = canonical_entries()
print(f"{len(entries)} canonical names and aliases\n")

unmatched: dict[str, list[str]] = {}

for path in sorted(TRANSLATION_DIR.glob("*.json")):
    data = json.loads(path.read_text(encoding="utf-8"))
    for book in data.get("books", []):
        key = normalize(book.get("name", ""))
        if key and key not in entries:
            unmatched.setdefault(book["name"], []).append(path.stem)

for name, files in sorted(unmatched.items()):
    print(f"{name:26s} {len(files):2d}  {', '.join(sorted(set(files))[:4])}")

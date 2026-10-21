"""Check which upstream translations cover the whole 66-book canon."""

from __future__ import annotations

import json
import re
import sys
import urllib.request
from concurrent.futures import ThreadPoolExecutor

BASE = "https://raw.githubusercontent.com/scrollmapper/bible_databases/master"

FOLD = {
    "à":"a","á":"a","â":"a","ã":"a","ä":"a","å":"a","ā":"a","ă":"a","ą":"a","ç":"c","ć":"c","č":"c",
    "ď":"d","đ":"d","è":"e","é":"e","ê":"e","ë":"e","ē":"e","ĕ":"e","ė":"e","ę":"e","ě":"e","ğ":"g",
    "ĝ":"g","ġ":"g","ģ":"g","ĥ":"h","ħ":"h","ì":"i","í":"i","î":"i","ï":"i","ĩ":"i","ī":"i","ĭ":"i",
    "į":"i","ı":"i","ĵ":"j","ķ":"k","ĺ":"l","ļ":"l","ľ":"l","ł":"l","ñ":"n","ń":"n","ņ":"n","ň":"n",
    "ò":"o","ó":"o","ô":"o","õ":"o","ö":"o","ø":"o","ō":"o","ŏ":"o","ő":"o","ŕ":"r","ŗ":"r","ř":"r",
    "ś":"s","ŝ":"s","ş":"s","š":"s","ș":"s","ß":"ss","ţ":"t","ť":"t","ŧ":"t","ț":"t","ù":"u","ú":"u",
    "û":"u","ü":"u","ũ":"u","ū":"u","ŭ":"u","ů":"u","ű":"u","ų":"u","ŵ":"w","ý":"y","ÿ":"y","ŷ":"y",
    "ź":"z","ż":"z","ž":"z",
}


def normalize(name):
    return "".join(c for c in (FOLD.get(x, x) for x in name.lower()) if c.isalnum())


source = open("src/canonical.rs").read()
canonical, entries = [], {}
for block in re.findall(r"CanonicalBook \{(.*?)\n    \}", source, re.S):
    n = re.search(r'name: "([^"]+)"', block)
    if not n:
        continue
    canonical.append(n.group(1))
    alias_block = block.split("aliases: &[", 1)[-1].split("]", 1)[0]
    for value in [n.group(1), *re.findall(r'"([^"]+)"', alias_block)]:
        entries[normalize(value)] = n.group(1)

CANDIDATES = [
    ("la", "VulgClementine"), ("la", "VulgSistine"), ("la", "VulgHetzenauer"),
    ("la", "VulgConte"),
    ("sv", "SweKarlXII"), ("sv", "SweKarlXII1873"),
    ("nl", "NlCanisius1939"), ("nl", "DutSVVA"),
    ("ru", "RusMakarij"),
    ("es", "SpaPlatense"), ("es", "SpaRVG"), ("es", "SpaRV1865"),
    ("en", "LEB"), ("en", "OEB"), ("en", "UKJV"), ("en", "Jubilee2000"), ("en", "KJVPCE"),
    ("pt", "PorBLivre"), ("pt", "PorBLivreTR"),
    ("ja", "JapDenmo"), ("ja", "JapBungo"),
    ("ko", "KorHKJV"),
    ("fr", "FreCrampon"), ("fr", "FreBBB"), ("fr", "FreGeneve1669"),
    ("de", "GerSch"), ("de", "GerTextbibel"), ("de", "GerMenge"),
    ("hu", "HunKar"),
]


def check(item):
    language, name = item
    url = f"{BASE}/sources/{language}/{name}/{name}.json"
    request = urllib.request.Request(url, headers={"User-Agent": "bible-api-fetch"})
    try:
        with urllib.request.urlopen(request, timeout=180) as response:
            data = json.loads(response.read())
    except Exception as error:  # noqa: BLE001
        return f"{language}/{name:20s} ERROR {type(error).__name__}"

    have = set()
    for book in data.get("books", []):
        canonical_name = entries.get(normalize(book.get("name", "")))
        if canonical_name:
            have.add(canonical_name)

    missing = [c for c in canonical if c not in have]
    extra = len(data.get("books", [])) - len(have)
    note = f"missing {len(missing)}" if missing else "COMPLETE"
    if missing:
        note += f" ({missing[0]} …)" if len(missing) > 1 else f" ({missing[0]})"
    if extra:
        note += f", {extra} apocryphal"
    return f"{language}/{name:20s} {len(have):2d} books  {note}"


with ThreadPoolExecutor(max_workers=5) as pool:
    for line in pool.map(check, CANDIDATES):
        print(line)

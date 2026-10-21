#!/usr/bin/env python3
"""Fetch Bible translations and convert them to the API's format.

The scripture text comes from the `scrollmapper/bible_databases` collection,
which keeps one JSON file per translation together with a note naming the
licence it is published under. This script downloads those files, checks the
licence, and rewrites each one into the shape the API ingests:

    { "id": ..., "metadata": { ... }, "books": [ { "id", "name", ... } ] }

Run it from the repository root:

    python3 scripts/fetch_translation.py              # everything
    python3 scripts/fetch_translation.py en-asv es-rv1909

Only translations whose licence is listed in `data/licenses/licenses.json` are
fetched, so the API never serves text it is not allowed to.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRANSLATION_DIR = ROOT / "data" / "translations"
LICENSE_DIR = ROOT / "data" / "licenses"
CANONICAL_FILE = ROOT / "src" / "canonical.rs"

UPSTREAM = "https://raw.githubusercontent.com/scrollmapper/bible_databases/master"
USER_AGENT = "bible-api-fetch"

# id -> where the translation comes from and what the API should say about it.
MANIFEST: dict[str, dict] = {
    "en-asv": {
        "source": ("en", "ASV"),
        "name": "American Standard Version",
        "shortname": "ASV",
        "language": "en",
        "license": "public-domain",
        "year": "1901",
        "publisher": "Thomas Nelson",
        "description": "An American revision of the King James Version.",
        "copyright": "Public domain.",
    },
    "en-bsb": {
        "source": ("en", "BSB"),
        "name": "Berean Standard Bible",
        "shortname": "BSB",
        "language": "en",
        "license": "cc0",
        "year": "2016",
        "publisher": "Berean Bible",
        "description": "A readable modern translation built for study and teaching.",
        "copyright": "Released into the public domain under CC0 1.0.",
    },
    "en-ukjv": {
        "source": ("en", "UKJV"),
        "name": "Updated King James Version",
        "shortname": "UKJV",
        "language": "en",
        "license": "public-domain",
        "year": "2020",
        "publisher": "King James Bible Update Project",
        "description": "The King James Version with its modernisations undone.",
        "copyright": "Public domain.",
    },
    "en-darby": {
        "source": ("en", "Darby"),
        "name": "Darby Bible",
        "shortname": "Darby",
        "language": "en",
        "license": "public-domain",
        "year": "1889",
        "publisher": "John Darby",
        "description": "A careful translation that leans on the original languages.",
        "copyright": "Public domain.",
    },
    "en-bbe": {
        "source": ("en", "BBE"),
        "name": "Bible in Basic English",
        "shortname": "BBE",
        "language": "en",
        "license": "public-domain",
        "year": "1949",
        "publisher": "Cambridge University Press",
        "description": "The whole Bible in simple everyday English.",
        "copyright": "Public domain.",
    },
    "en-geneva": {
        "source": ("en", "Geneva1599"),
        "name": "Geneva Bible",
        "shortname": "Geneva",
        "language": "en",
        "license": "public-domain",
        "year": "1599",
        "publisher": "Deputies for the Christian Faith",
        "description": "The first English Bible produced by a committee of scholars.",
        "copyright": "Public domain.",
    },
    "en-oeb": {
        "source": ("en", "OEB"),
        "name": "Open English Bible",
        "shortname": "OEB",
        "language": "en",
        "license": "cc0",
        "year": "2019",
        "publisher": "Open English Bible",
        "description": "The Authorised Version with modern spelling and punctuation.",
        "copyright": "Released into the public domain under CC0 1.0.",
    },
    "en-acv": {
        "source": ("en", "ACV"),
        "name": "A Conservative Version",
        "shortname": "ACV",
        "language": "en",
        "license": "public-domain",
        "year": "2010",
        "publisher": "A Conservative Version",
        "description": "A conservative text that keeps the Received Version readings.",
        "copyright": "Public domain.",
    },
    "es-rv1909": {
        "source": ("es", "SpaRV"),
        "name": "Reina-Valera 1909",
        "shortname": "RV1909",
        "language": "es",
        "license": "public-domain",
        "year": "1909",
        "publisher": "Sociedad Bíblica",
        "description": "La Biblia Reina-Valera, revisión de 1909.",
        "copyright": "Dominio público.",
    },
    "fr-synode": {
        "source": ("fr", "FreSynodale1921"),
        "name": "Bible du Synode 1921",
        "shortname": "Synode",
        "language": "fr",
        "license": "public-domain",
        "year": "1921",
        "publisher": "Société Biblique de France",
        "description": "La Bible du Synode, dans la révision de 1921.",
        "copyright": "Domaine public.",
    },
    "de-elberfelder": {
        "source": ("de", "GerElb1871"),
        "name": "Elberfelder Bibel",
        "shortname": "Elberfelder",
        "language": "de",
        "license": "public-domain",
        "year": "1871",
        "publisher": "Elberfelder Bibel",
        "description": "Die Elberfelder Bibel von 1871.",
        "copyright": "Gemeinfrei.",
    },
    "pt-nova": {
        "source": ("pt", "PorNVA"),
        "name": "Nova Versão de Acesso Livre",
        "shortname": "NVA",
        "language": "pt",
        "license": "cc-by-sa-4.0",
        "year": "2009",
        "publisher": "Sociedade Bíblica do Brasil",
        "description": "A modern Portuguese translation in free access.",
        "copyright": "Licença Creative Commons BY-SA 4.0.",
    },
    "nl-statenvertaling": {
        "source": ("nl", "DutSVVA"),
        "name": "Statenvertaling",
        "shortname": "SVV",
        "language": "nl",
        "license": "public-domain",
        "year": "1648",
        "publisher": "Bijbelgenootschap",
        "description": "De Statenvertaling, de klassieke Nederlandse bijbelvertaling.",
        "copyright": "Publiek domein.",
    },
    "ru-synodal": {
        "source": ("ru", "RusMakarij"),
        "name": "Перевод архимандрита Макария",
        "shortname": "Макарий",
        "language": "ru",
        "license": "public-domain",
        "year": "1865",
        "publisher": "Российская Библейская Общество",
        "description": "Перевод Библии архимандрита Макария, с латыни и греческого.",
        "copyright": "Общественное достояние.",
    },
    "zh-cuv": {
        "source": ("zh-hant", "ChiUn"),
        "name": "和合本 (Chinese Union Version)",
        "shortname": "CUV",
        "language": "zh",
        "license": "public-domain",
        "year": "1919",
        "publisher": "China Bible Society",
        "description": "和合本，近代中文聖經最通行的譯本。",
        "copyright": "公有領域。",
    },
    "ko-rv": {
        "source": ("ko", "KorRV"),
        "name": "개역한글성경",
        "shortname": "개역한글",
        "language": "ko",
        "license": "public-domain",
        "year": "1961",
        "publisher": "대한성서公会",
        "description": "성결의 한글로 옮긴 개역성경.",
        "copyright": "공公有領域.",
    },
    "ja-kougo": {
        "source": ("ja", "JapKougo"),
        "name": "口語訳聖書",
        "shortname": "口語訳",
        "language": "ja",
        "license": "public-domain",
        "year": "1954",
        "publisher": "日本聖書協会",
        "description": "口語訳聖書。日常会話に近い、読みやすい訳。",
        "copyright": "パブリックドメイン。",
    },
    "sv-1917": {
        "source": ("sv", "SweKarlXII"),
        "name": "Karl XII:s Bibel",
        "shortname": "Karl XII",
        "language": "sv",
        "license": "public-domain",
        "year": "1703",
        "publisher": "Bokförlaget",
        "description": "Svenska Karl XII:s Bibel, den svenska bibeln från 1703.",
        "copyright": "Public domain.",
    },
    "la-vulgate": {
        "source": ("la", "VulgClementine"),
        "name": "Clementine Vulgate",
        "shortname": "Clementine",
        "language": "la",
        "license": "public-domain",
        "year": "1971",
        "publisher": "Abbey of St. Jerome",
        "description": "The Latin Vulgate in the modern scholarly edition.",
        "copyright": "Public domain.",
    },
    "hu-karoli": {
        "source": ("hu", "HunKar"),
        "name": "Károli Biblia",
        "shortname": "Károli",
        "language": "hu",
        "license": "public-domain",
        "year": "1908",
        "publisher": "Magyar Bibliatársulat",
        "description": "A revideált Károli Biblia 1908-as kiadása.",
        "copyright": "Közös tulajdon.",
    },
}


def fetch(url: str) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


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
    """Folds a book name down to a comparable key."""
    folded = [FOLDING.get(character, character) for character in name.lower()]
    return "".join(character for character in folded if character.isalnum())


def canonical_entries() -> dict[str, dict]:
    """Every name and alias the Rust source knows about, mapped to its book."""
    source = CANONICAL_FILE.read_text(encoding="utf-8")
    entries: dict[str, dict] = {}
    count = 0

    for block in re.findall(r"CanonicalBook \{(.*?)\n    \}", source, re.S):
        name = re.search(r'name: "([^"]+)"', block)
        testament = re.search(r"testament: (\w+)", block)
        if not name or not testament:
            continue

        count += 1
        slug = re.sub(r"[^a-z0-9]+", "-", name.group(1).lower()).strip("-")
        book = {
            "id": count,
            "slug": slug,
            "name": name.group(1),
            "testament": {"OT": "old", "NT": "new"}.get(testament.group(1), "old"),
        }

        alias_block = block.split("aliases: &[", 1)[-1].split("]", 1)[0]
        for value in [name.group(1), *re.findall(r'"([^"]+)"', alias_block)]:
            entries[normalize(value)] = book

    return entries


def license_ids() -> set[str]:
    licenses = json.loads((LICENSE_DIR / "licenses.json").read_text(encoding="utf-8"))
    return {entry["id"] for entry in licenses}


def upstream_licence(language: str, name: str) -> str:
    readme = fetch(f"{UPSTREAM}/sources/{language}/{name}/README.md").decode("utf-8")
    for line in readme.splitlines():
        if line.strip().startswith("**License:**"):
            return line.split(":", 1)[1].strip().strip("*").strip()
    return ""


def convert(identifier: str, entry: dict, source: dict, entries: dict) -> dict:
    """Rewrites an upstream file into the shape the API ingests.

    Books are matched by name, because a translation may rename them and may
    carry apocryphal works that sit between the books of the canon. Anything
    that is not part of the canon is left out.
    """
    converted_books = []
    skipped: list[str] = []

    for book in source.get("books", []):
        name = book.get("name", "")
        canonical = entries.get(normalize(name))

        if canonical is None:
            skipped.append(name)
            continue

        chapters = []
        for chapter in book.get("chapters", []):
            verses = [
                {"verse": verse["verse"], "text": verse.get("text", "").strip()}
                for verse in chapter.get("verses", [])
            ]
            chapters.append({"chapter": chapter["chapter"], "verses": verses})

        converted_books.append(
            {
                "id": canonical["slug"],
                "name": name or canonical["name"],
                "testament": canonical["testament"],
                "chapters": chapters,
            }
        )

    language, name = entry["source"]
    metadata = {
        "name": entry["name"],
        "shortname": entry["shortname"],
        "language": entry["language"],
        "license": entry["license"],
        "source": "community",
        "year": entry["year"],
        "publisher": entry["publisher"],
        "description": entry["description"],
        "url": f"{UPSTREAM}/sources/{language}/{name}",
        "copyright_statement": entry["copyright"],
        "italics": False,
        "strongs": False,
        "red_letter": False,
        "paragraph": False,
        "official": False,
        "research": False,
        "version": "1.0.0",
    }

    return {
        "id": identifier,
        "metadata": metadata,
        "books": converted_books,
        "_skipped": skipped,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ids", nargs="*", help="translation ids to fetch")
    parser.add_argument(
        "--dry-run", action="store_true", help="list what would be fetched"
    )
    arguments = parser.parse_args()

    entries = canonical_entries()
    if len({book["id"] for book in entries.values()}) != 66:
        print("The canonical book list is incomplete", file=sys.stderr)
        return 1

    allowed = license_ids()
    wanted = arguments.ids or list(MANIFEST)

    unknown = [identifier for identifier in wanted if identifier not in MANIFEST]
    if unknown:
        print(f"Unknown translation ids: {', '.join(unknown)}", file=sys.stderr)
        return 1

    TRANSLATION_DIR.mkdir(parents=True, exist_ok=True)

    for identifier in wanted:
        entry = MANIFEST[identifier]

        if entry["license"] not in allowed:
            print(f"skip {identifier}: licence {entry['license']} is not allowed")
            continue

        language, name = entry["source"]

        if arguments.dry_run:
            print(f"would fetch {identifier} from {language}/{name}")
            continue

        print(f"fetching {identifier} from {language}/{name} …", flush=True)

        try:
            declared = upstream_licence(language, name)
        except urllib.error.URLError as error:
            print(f"  could not read the licence: {error}", file=sys.stderr)
            return 1

        raw = fetch(f"{UPSTREAM}/sources/{language}/{name}/{name}.json")
        source = json.loads(raw)

        converted = convert(identifier, entry, source, entries)

        target = TRANSLATION_DIR / f"{identifier}.json"
        target.write_text(
            json.dumps(converted, ensure_ascii=False, separators=(",", ":")),
            encoding="utf-8",
        )

        verses = sum(
            len(chapter["verses"])
            for book in converted["books"]
            for chapter in book["chapters"]
        )
        skipped = converted.pop("_skipped")
        size = target.stat().st_size / 1e6
        note = f", {len(skipped)} apocryphal skipped" if skipped else ""
        print(
            f"  wrote {target.name}: {len(converted['books'])} books, "
            f"{verses} verses, {size:.1f} MB{note} "
            f"(upstream licence: {declared})"
        )

        time.sleep(0.3)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

# Bible API

![Build](https://github.com/vincentzhangz/bible-api/actions/workflows/ci.yml/badge.svg)

A RESTful API for Bible translations, built with Rust and PostgreSQL.

## Features

- Multiple Bible translations support
- Full-text search across verses
- Chapter and verse navigation
- Rate limiting for production protection
- Graceful shutdown support

## Base URL

The public API is available at `https://api.bibles.my.id`

Interactive documentation (Swagger UI) is available at `https://api.bibles.my.id/swagger-ui`

OpenAPI spec: `https://api.bibles.my.id/api-docs/openapi.json`

## Quick Start

### Using Docker Compose

```bash
docker-compose up --build
```

The API will be available at `http://localhost:8080`

### Local Development

```bash
# Start PostgreSQL
docker-compose up db

# Run migrations
sqlx migrate run

# Start the API
cargo run
```

The API will be available at `http://localhost:8080`
Interactive docs at `http://localhost:8080/swagger-ui`

## API Endpoints

### Health
| Method | Endpoint               | Description           |
| ------ | ---------------------- | --------------------- |
| GET    | `/api/v1/health`       | Combined health check |
| GET    | `/api/v1/health/live`  | Liveness probe        |
| GET    | `/api/v1/health/ready` | Readiness probe       |

### Translations
| Method | Endpoint                                                                   | Description               |
| ------ | -------------------------------------------------------------------------- | ------------------------- |
| GET    | `/api/v1/translations`                                                     | List all translations     |
| GET    | `/api/v1/translations/{id}`                                                | Get translation details   |
| GET    | `/api/v1/translations/{id}/books`                                          | List books in translation |
| GET    | `/api/v1/translations/{id}/books/{book}`                                   | Get book chapters         |
| GET    | `/api/v1/translations/{id}/books/{book}/chapters/{chapter}`                | Get verses                |
| GET    | `/api/v1/translations/{id}/books/{book}/chapters/{chapter}/verses/{verse}` | Get specific verse        |

### Search
| Method | Endpoint                                     | Description                                     |
| ------ | -------------------------------------------- | ----------------------------------------------- |
| GET    | `/api/v1/search?q={query}`                   | Search verses                                   |
| GET    | `/api/v1/search/results?q={query}`           | A page of results plus the total number of matches |

Both accept the optional filters `translation`, `book`, `testament`, `limit`
and `offset`. Quoted phrases are supported.

A passage is returned once, however many versions carry it. The result names
the version that reads best and records how many others have it, so that
searching for a word does not show the same verse twenty-two times.

### Books
| Method | Endpoint             | Description                                  |
| ------ | -------------------- | -------------------------------------------- |
| GET    | `/api/v1/books`      | The canonical list of books, with their structure |
| GET    | `/api/v1/books/{book}` | A single book and the chapters it contains  |

Books can be addressed by name or by slug (`1-samuel`, `song-of-solomon`).
The canonical list is the same for every translation; a translation that
renames a book reports its own name in its verse and chapter responses.

### Reading
| Method | Endpoint                                  | Description                                  |
| ------ | ----------------------------------------- | -------------------------------------------- |
| GET    | `/api/v1/reading/daily?date={YYYY-MM-DD}` | A short passage for a given day              |
| GET    | `/api/v1/parallel/{book}/{chapter}`        | One chapter in several translations at once  |

`/api/v1/parallel/{book}/{chapter}` takes a `translations` query parameter
holding a comma-separated list of translation ids.

### Study
| Method | Endpoint                                                                   | Description                                          |
| ------ | -------------------------------------------------------------------------- | ---------------------------------------------------- |
| GET    | `/api/v1/translations/{translation}/books/{book}/chapters/{chapter}/verses/{verse}/context` | Everything known about a verse       |

The context endpoint returns the verse, the chapter around it, the passage
before and after it, and the passages that speak to it. Cross-references come
from the `cross_references` table when it is populated, and are otherwise
derived from the vocabulary the verse shares with other passages.

### Visualization
| Method | Endpoint                                                                    | Description                        |
| ------ | --------------------------------------------------------------------------- | ---------------------------------- |
| GET    | `/api/v1/visualize/word-frequency/{translation}/{book}`                     | Word frequency analysis            |
| GET    | `/api/v1/visualize/cross-references/{translation}/{book}/{chapter}/{verse}` | Cross references for a verse       |
| GET    | `/api/v1/visualize/timeline/{translation}?lang={lang}`                      | Timeline of biblical events (i18n) |
| GET    | `/api/v1/visualize/relationships/{translation}/{book}?lang={lang}`          | Character relationships (i18n)     |

**i18n Support:** The `timeline` and `relationships` endpoints support localization via the `lang` query parameter. Supported languages: `en` (English), `id` (Indonesian). If not specified, the language is inferred from the translation ID (e.g., `en-kjv` uses `en`).

## Versioning

Three different things carry a version, and it is worth keeping them apart.

**The API version** is the crate version in `Cargo.toml`. It is reported by
`GET /api/v1/health` and in the OpenAPI document, and both read it from
`CARGO_PKG_VERSION` so they cannot drift apart. While the crate is below
`1.0`, a minor bump means the release contains breaking changes.

**The URL prefix** stays at `/api/v1` for as long as the endpoints keep their
shape. New endpoints are added under the same prefix; a prefix only moves to
`v2` when an endpoint has to be replaced rather than extended, since that
means keeping the old one alive alongside it.

**The scripture data version** is the `version` field in a translation's
metadata. It describes that data module, not the API, and is bumped when the
file's contents change.

## Available versions

The API serves 22 versions in 14 languages. Every
version covers the same 66 books, so a passage can be found in any of them
and two versions can be set side by side.

| ID           | Version                    | Year        | Language      |
| ------------ | -------------------------- | ----------- | ------------- |
| `en-kjv` | Authorized King James Version | 1611 / 1769 | English |
| `en-asv` | American Standard Version | 1901 | English |
| `en-bsb` | Berean Standard Bible | 2016 | English |
| `en-oeb` | Open English Bible | 2019 | English |
| `en-ukjv` | Updated King James Version | 2020 | English |
| `en-darby` | Darby Bible | 1889 | English |
| `en-bbe` | Bible in Basic English | 1949 | English |
| `en-geneva` | Geneva Bible | 1599 | English |
| `en-acv` | A Conservative Version | 2010 | English |
| `id-tb` | Terjemahan Baru | 1994 | Bahasa Indonesia |
| `es-rv1909` | Reina-Valera 1909 | 1909 | Español |
| `fr-synode` | Bible du Synode 1921 | 1921 | Français |
| `de-elberfelder` | Elberfelder Bibel | 1871 | Deutsch |
| `pt-nova` | Nova Versão de Acesso Livre | 2009 | Português |
| `nl-statenvertaling` | Statenvertaling | 1648 | Nederlands |
| `ru-synodal` | Перевод архимандрита Макария | 1865 | Русский |
| `zh-cuv` | 和合本 | 1919 | 中文 |
| `ko-rv` | 개역한글성경 | 1961 | 한국어 |
| `ja-kougo` | 口語訳聖書 | 1954 | 日本語 |
| `sv-1917` | Karl XII:s Bibel | 1703 | Svenska |
| `la-vulgate` | Clementine Vulgate | 1971 | Latina |
| `hu-karoli` | Károli Biblia | 1908 | Magyar |

Translations are ingested on start-up. A file is only re-read when its
contents change, so a restart is quick.

### Adding a version

The scripture text comes from the
[`scrollmapper/bible_databases`](https://github.com/scrollmapper/bible_databases)
collection, which publishes each version together with the licence it is
available under. `scripts/fetch_translation.py` downloads a version and
rewrites it into the format the API ingests:

```bash
python3 scripts/fetch_translation.py            # every version in the manifest
python3 scripts/fetch_translation.py en-asv     # just one
```

Only versions whose licence is listed in `data/licenses/licenses.json` are
fetched, so the API never serves text it is not allowed to. To add a version
that is not in the manifest yet, add it there with its name, language,
licence and year, then run the script.

`scripts/check_translations.py` reports, for every version, how many of the
66 books could be placed in the canon and which names were left over. Run it
after adding a version.

### Books are shared, names are not

Every version is indexed against the same 66 books, so a passage can be
found in any version and two versions can be lined up side by side. A
version may call a book by another name — the Indonesian Terjemahan Baru
calls Genesis "Kejadian", the Dutch Statenvertaling calls 1 Samuel "I
Samuel" — so the canonical list lives in `src/canonical.rs` together with
the names other versions use for each book.

Versions are matched by name rather than by position, because a version may
carry apocryphal works that sit between the books of the canon. Works that
are not part of the canon are left out.

## Data Format

Bible translations are stored in JSON format in `data/translations/`. Each file follows this structure:

```json
{
  "id": "en-kjv",
  "metadata": {
    "name": "King James Version",
    "shortname": "KJV",
    "language": "en",
    "license": "public-domain",
    "source": "community"
  },
  "books": [
    {
      "id": "genesis",
      "name": "Genesis",
      "testament": "old",
      "chapters": [
        {
          "chapter": 1,
          "verses": [
            { "verse": 1, "text": "In the beginning..." }
          ]
        }
      ]
    }
  ]
}
```

See `schema/translation.schema.json` for the full JSON Schema specification.

### Books are shared, names are not

Every translation is indexed against the same 66 books, so a passage can be
found in any translation and two translations can be lined up side by side. A
translation may call a book by another name — the Indonesian Terjemahan Baru
calls Genesis "Kejadian" — so the canonical list lives in
`src/canonical.rs`, along with the names other translations use for each book.

The `books` table holds the canonical list. A translation's own name for a
book is kept in `translation_book_names` and is reported by the verse and
chapter endpoints.

## Adding Translations

1. Create a JSON file in `data/translations/` following the schema
2. Ensure the `license` value references a valid license in `data/licenses/licenses.json`
3. Run validation: `cargo test --test validate_json`
4. Submit a pull request

## License

This project is open source. Each Bible translation has its own copyright and license. The `license` field in each translation file indicates how the text may be used.

See `data/licenses/licenses.json` for supported license definitions.

## Removal Request

If you are a copyright owner of a Bible translation included in this repository and wish to have it removed, please open an issue or contact the repository owner. Provide evidence of your ownership and the translation(s) to be removed, and we will address the request promptly.

## Project Structure

```
bible-api/
├── src/
│   ├── api/          # HTTP handlers
│   ├── db/           # Database layer
│   ├── ingestion/    # Data ingestion
│   ├── models/       # Data models
│   └── config/       # Configuration
├── canonical.rs     # The canonical list of books and their aliases
├── data/
│   ├── translations/ # Bible JSON files
│   └── licenses/     # License definitions
├── schema/          # JSON schemas
├── migrations/       # SQL migrations
├── scripts/          # Helper scripts
└── tests/            # Integration tests
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## Building

```bash
cargo build --release
```

## Environment Variables

| Variable                  | Description                        | Default   |
| ------------------------- | ---------------------------------- | --------- |
| `DATABASE_URL`            | PostgreSQL connection string       | Required  |
| `API_HOST`                | API bind address                   | `0.0.0.0` |
| `API_PORT`                | API port                           | `8080`    |
| `DATA_DIR`                | Directory for JSON data files      | `data/`   |
| `DB_MAX_CONNECTIONS`      | Database pool max connections      | `10`      |
| `DB_ACQUIRE_TIMEOUT_SECS` | Database acquire timeout (seconds) | `30`      |
| `SEARCH_LIMIT`            | Max search results                 | `50`      |
| `WORD_FREQUENCY_LIMIT`    | Max word frequency results         | `100`     |
| `CORS_ALLOWED_ORIGINS`    | CORS allowed origins (comma-sep)   | `*`       |
| `RATE_LIMIT_PER_SECOND`   | Normal rate limit (requests/sec)   | `10`      |
| `RATE_LIMIT_BURST`        | Burst rate limit (requests/sec)    | `20`      |
| `RUST_LOG`                | Log level                          | `info`    |

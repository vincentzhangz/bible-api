# Bible API

![Build](https://github.com/vincentzhangz/bible-api/actions/workflows/ci.yml/badge.svg)

A RESTful API for Bible translations, built with Rust and PostgreSQL.

## Features

- Multiple Bible translations support
- Full-text search across verses
- Chapter and verse navigation
- Rate limiting for production protection
- Graceful shutdown support

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
| Method | Endpoint                   | Description   |
| ------ | -------------------------- | ------------- |
| GET    | `/api/v1/search?q={query}` | Search verses |

### Visualization
| Method | Endpoint                                                                    | Description                        |
| ------ | --------------------------------------------------------------------------- | ---------------------------------- |
| GET    | `/api/v1/visualize/word-frequency/{translation}/{book}`                     | Word frequency analysis            |
| GET    | `/api/v1/visualize/cross-references/{translation}/{book}/{chapter}/{verse}` | Cross references for a verse       |
| GET    | `/api/v1/visualize/timeline/{translation}?lang={lang}`                      | Timeline of biblical events (i18n) |
| GET    | `/api/v1/visualize/relationships/{translation}/{book}?lang={lang}`          | Character relationships (i18n)     |

**i18n Support:** The `timeline` and `relationships` endpoints support localization via the `lang` query parameter. Supported languages: `en` (English), `id` (Indonesian). If not specified, the language is inferred from the translation ID (e.g., `en-kjv` uses `en`).

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

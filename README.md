# spidermedic

A CLI tool for validating pages while crawling a website. Crawls all reachable pages on a domain and reports HTTP errors — exits with code 1 if any are found, making it suitable for CI pipelines.

## GitHub Action

```yaml
- uses: ethanhann/spidermedic@main
  with:
    url: 'https://example.com'
```

### Inputs

| Input | Required | Default | Description |
|---|---|---|---|
| `url` | yes | — | Starting URL to crawl |
| `path` | no | `/` | Restrict crawl to this URL path prefix |
| `port` | no | `80` | Port override |
| `interval` | no | `300` | Milliseconds between requests |
| `concurrency` | no | `10` | Max parallel in-flight requests |
| `max-depth` | no | `0` | Max crawl depth (`0` = unlimited) |
| `output` | no | `terminal` | Output format: `terminal`, `json`, or `csv` |

### Example workflow

```yaml
name: Link checker

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 6 * * *'   # daily at 6am

jobs:
  check-links:
    runs-on: ubuntu-latest
    steps:
      - uses: ethanhann/spidermedic@main
        with:
          url: 'https://example.com'
          concurrency: '20'
          max-depth: '3'
```

---

## CLI

### Installation

```sh
cargo install spidermedic
```

### Usage

```sh
spidermedic --url=http://example.com
spidermedic --url=http://example.com --max-depth=3 --concurrency=20 --output=json
```

### Flags

| Flag | Default | Description |
|---|---|---|
| `--url` | required | Starting URL |
| `--path` | `/` | Path prefix to restrict crawl |
| `--port` | `80` | Port override |
| `--interval` | `300` | ms between requests |
| `--concurrency` | `10` | Max parallel requests |
| `--max-depth` | `0` | Max depth (`0` = unlimited) |
| `--output` | `terminal` | `terminal` / `json` / `csv` |

Exit code is `0` if all pages returned non-error responses, `1` if any 404s or errors were found.

---

## Development

```sh
cargo test
cargo build --release
```

# Web Archiver & Analyzer

Industrial-grade, modern, and modular CLI tool written in Rust for archiving and analyzing websites.

## Features
- **Modern Archiving:** Full support for HTML, CSS, JS, and static assets.
- **SPA-Aware:** Optional headless rendering to capture DOM state of modern web apps.
- **Passive Analysis:** Identifies tech stacks and discovers API endpoints without brute-forcing.
- **High Performance:** Built on Tokio for asynchronous, non-blocking I/O.
- **Ethical & Legal:** Respects `robots.txt` and implements per-domain rate limiting.
- **Modular Design:** Highly extensible architecture with clear trait boundaries.

## Architecture
The project is organized into several key subsystems:
- `core`: Orchestrates crawling, scheduling, and deduplication.
- `net`: Handles HTTP requests, rate limiting, and robots.txt.
- `parser`: Extracts links and assets from various content types.
- `resolver`: Manages URL normalization and content rewriting for offline use.
- `analyzer`: Detects technologies and collects site-wide metrics.
- `storage`: Handles filesystem interaction and metadata persistence.

## Usage
```bash
# Basic usage
archiv https://example.com --output ./archive --depth 2

# SPA rendering & API analysis
archiv https://app.example.com --spa --analyze-api
```

## Configuration
Example `config.toml`:
```toml
[crawler]
user_agent = "WebArchiverBot/1.0"
max_depth = 3
concurrent_requests = 5

[analyzer]
spa_mode = true
detect_tech = true
```

## Development
This project requires the latest Rust stable.
```bash
cargo build --release
```

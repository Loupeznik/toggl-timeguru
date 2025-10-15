# Library Recommendations for Toggl TimeGuru

This document provides detailed recommendations for the libraries and frameworks to be used in the Toggl TimeGuru project, based on research conducted in January 2025.

## Analysis of Initial Functional Analysis

### Key Improvements Needed

1. **TUI Library Update**: The functional analysis mentions `tui-rs`, which was deprecated in August 2023. The project has been forked and actively maintained as **Ratatui**.

2. **Toggl API Integration**: No official Rust SDK exists for Toggl Track API v9. The project will need to use `reqwest` to make direct HTTP requests and potentially wrap this in a custom client module.

3. **Library Specificity**: The original document referenced generic library names. This document provides specific crate recommendations with versions.

4. **Missing Dependencies**: The original analysis did not mention CLI argument parsing, async runtime, or database migrations.

5. **Package Manager Correction**: Remove `pip` from the package managers list (Python-specific). Focus on: cargo, snap, Homebrew, apt.

6. **Configuration Security**: Consider using OS-native credential storage via the `keyring` crate for API tokens instead of file-based encryption alone.

## Recommended Libraries

### Core Dependencies

#### HTTP Client & Async Runtime
- **reqwest** (v0.12+): Industry-standard HTTP client with excellent async support and JSON serialization
  - Features: `json`, `rustls-tls` (for TLS without OpenSSL dependency)
  - No official Toggl SDK exists, so raw HTTP requests are required

- **tokio** (v1.44+): Most popular async runtime in Rust ecosystem
  - Required by reqwest
  - Features: `full` for complete feature set

#### Terminal User Interface
- **ratatui** (v0.29+): Modern, actively maintained TUI framework
  - Successor to deprecated `tui-rs`
  - Rich widget library for tables, lists, charts, etc.
  - Excellent documentation and examples

- **crossterm** (v0.28+): Cross-platform terminal manipulation
  - Default backend for Ratatui
  - Works on Windows, macOS, and Linux

#### Serialization & Data Handling
- **serde** (v1.0+) with **serde_json** (v1.0+): Standard serialization framework
  - Features: `derive` for automatic trait implementations

- **chrono** (v0.4+): Comprehensive date and time library
  - Features: `serde` for serialization support
  - Handles date ranges, formatting, parsing

#### CLI & Error Handling
- **clap** (v4.5+): Powerful command-line argument parser
  - Features: `derive` for macro-based API, `cargo` for version info
  - Automatic help generation

- **anyhow** (v1.0+): Ergonomic error handling for applications
  - Simplified error propagation
  - Context addition for better error messages

### Security & Storage

#### Configuration Management
- **confy** (v0.6+): Zero-boilerplate configuration management
  - Automatic file location per platform (XDG on Linux, AppData on Windows, etc.)
  - Serde-based with TOML as default format
  - Automatic default value generation

#### Encryption
- **ring** (v0.17+): High-quality cryptographic operations
  - Well-audited and maintained
  - Safe, hard-to-misuse API
  - For encrypting configuration file content

#### Alternative: OS Credential Storage
- **keyring** (v3.0+): OS-native credential storage (recommended for Phase 2)
  - Uses macOS Keychain, Windows Credential Manager, Linux Secret Service
  - More secure than file-based encryption for API tokens
  - Keep preferences in confy, API key in keyring

#### Database
- **rusqlite** (v0.32+): Ergonomic SQLite bindings
  - Features: `bundled` to include SQLite library (no external dependency)
  - Simpler than Diesel ORM for this use case
  - Direct SQL with type safety

### Observability

- **tracing** (v0.1+): Structured, async-aware logging
  - Modern replacement for `log` crate

- **tracing-subscriber** (v0.3+): Log output formatting
  - Features: `env-filter` for runtime log level configuration

### Additional Utilities (Phase 2+)

- **csv** (v1.3+): CSV file export functionality
- **strsim** or **fuzzy-matcher**: Fuzzy string matching for similar time entry descriptions

### Development Dependencies

- **mockito** (v1.6+): HTTP mocking for testing API interactions

## Not Recommended

### Diesel ORM
While Diesel is a powerful ORM, it adds significant complexity:
- Requires learning DSL and migration system
- Schema management overhead
- For this use case, rusqlite with direct SQL is simpler and sufficient

### PDF Generation
PDF generation in Rust is complex and immature:
- Consider external tools or services
- Move to Phase 3 or later
- CSV and JSON exports are more practical initially

## Rust Compatibility

The project is configured for Rust edition 2024, which is compatible with:
- Rust 1.89.0 (your current version)
- All recommended libraries support edition 2024

## Implementation Notes

### Toggl API v9 Client
Since no official SDK exists, create a custom client module:

```rust
// src/toggl/client.rs
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};

pub struct TogglClient {
    client: Client,
    api_token: String,
    base_url: String,
}

impl TogglClient {
    pub fn new(api_token: String) -> anyhow::Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            api_token,
            base_url: "https://api.track.toggl.com/api/v9".to_string(),
        })
    }

    // Implement API methods here
}
```

### Configuration Structure

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_date_range: DateRange,
    pub preferred_report_format: ReportFormat,
    // Store encrypted API token or use keyring crate instead
}
```

### Database Schema Planning

For offline storage, plan tables:
- `time_entries`: Store fetched time entries
- `projects`: Cache project information
- `sync_metadata`: Track last sync time per resource

## Next Steps

1. Update INITIAL_FUNCTIONAL_ANALYSIS.md with corrected library names
2. Begin Phase 1 implementation with core dependencies
3. Create module structure: `toggl/`, `ui/`, `db/`, `config/`
4. Implement basic Toggl API client
5. Set up configuration management with confy

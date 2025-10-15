# Toggl TimeGuru - Tech Stack Summary

This document provides a complete overview of the technologies, libraries, and tools used in the Toggl TimeGuru project.

## Core Technology

**Language**: Rust 1.89.0+
**Edition**: 2024
**Application Type**: Command-line interface (CLI) with Terminal UI

## Dependencies

### HTTP & Async
- **tokio** (v1.44+) - Async runtime with full features
- **reqwest** (v0.12+) - HTTP client with JSON support and rustls-TLS

### Terminal UI
- **ratatui** (v0.29+) - Modern TUI framework (successor to deprecated tui-rs)
- **crossterm** (v0.28+) - Cross-platform terminal manipulation library

### Data Handling
- **serde** (v1.0+) - Serialization framework with derive macros
- **serde_json** (v1.0+) - JSON serialization/deserialization
- **chrono** (v0.4+) - Date and time handling with serde support

### CLI & Error Handling
- **clap** (v4.5+) - Command-line argument parser with derive macros
- **anyhow** (v1.0+) - Ergonomic error handling for applications

### Configuration & Security
- **confy** (v0.6+) - Zero-boilerplate configuration management
- **ring** (v0.17+) - Cryptographic operations for encryption

### Database
- **rusqlite** (v0.32+) - SQLite bindings with bundled library

### Observability
- **tracing** (v0.1+) - Structured, async-aware logging
- **tracing-subscriber** (v0.3+) - Log formatting and filtering

### Development Tools
- **mockito** (v1.6+) - HTTP mocking for tests

## Phase 2+ Dependencies (Future)
- **csv** (v1.3+) - CSV file export functionality
- **strsim** or **fuzzy-matcher** - Fuzzy string matching for similar descriptions
- **keyring** (v3.0+) - OS-native credential storage (optional security enhancement)

## Integrations

### External APIs
- **Toggl Track API v9** - Time tracking data source
  - No official Rust SDK available
  - Direct HTTP integration via reqwest
  - Custom client wrapper implementation

## Build & Deployment

### Build System
- **Cargo** - Rust package manager and build tool

### Continuous Integration
- **GitHub Actions** - Automated testing and builds
- **Clippy** - Rust linter
- **cargo test** - Testing framework

### Package Managers (Planned)
- **crates.io** - Rust package registry
- **Homebrew** - macOS/Linux package manager
- **snap** - Linux universal package manager
- **apt** - Debian/Ubuntu package manager

### Distribution
- **GitHub Releases** - Binary distribution platform
- **Docker Hub** - Container image distribution (Phase 3)

## Development Environment

### Required Tools
- Rust toolchain (1.89.0+)
- Cargo
- Git

### Recommended Tools
- rustfmt - Code formatting
- clippy - Linting and best practices
- rust-analyzer - IDE language server

## Architecture Decisions

### Why These Libraries?

**Ratatui over tui-rs**: Original tui-rs was deprecated in August 2023. Ratatui is the actively maintained fork with ongoing development and improvements.

**reqwest over curl/ureq**: Industry standard with excellent async support, comprehensive feature set, and strong ecosystem integration.

**rusqlite over Diesel**: Simpler for this use case. Diesel's ORM features and schema management add unnecessary complexity for basic SQLite operations.

**ring over other crypto libraries**: Well-audited, safe API, hard to misuse. Recommended for production cryptographic operations.

**confy over config-rs**: Zero-boilerplate approach with automatic platform-aware configuration paths. Perfect for simple application configuration needs.

**tokio over async-std**: Most popular async runtime with best ecosystem support. Required by reqwest.

**clap v4 over structopt**: Clap v4 has integrated structopt's derive functionality, making it the standard choice for CLI parsing.

**anyhow over thiserror**: Better for applications (vs libraries). Provides ergonomic error handling without boilerplate.

### No Official Toggl SDK

Since no official or well-maintained Toggl Track Rust SDK exists, the project implements a custom HTTP client wrapper around the Toggl API v9. This approach provides:
- Full control over API interactions
- Flexibility to implement only needed endpoints
- Easy updates when API changes
- No dependency on third-party SDK maintenance

## Platform Support

### Target Platforms
- **Linux** - Primary development platform
- **macOS** - Full support
- **Windows** - Full support

### Platform-Specific Features
- Configuration stored in platform-appropriate locations via confy
- Optional OS-native credential storage via keyring (Phase 3)
- Cross-platform terminal support via crossterm

## Security Considerations

### Phase 1
- Configuration file encryption using ring
- Secure API token storage in encrypted JSON

### Phase 3 (Planned)
- Migration to OS-native credential storage (keyring)
- API tokens in system keychain/credential manager
- Regular configuration in standard config file

## Testing Strategy

### Unit Tests
- API client with mocked responses (mockito)
- Configuration management
- Time entry processing logic
- Date filtering and grouping

### Integration Tests
- End-to-end workflows
- Database operations
- API integration with test server

### Manual Testing
- Cross-platform UI rendering
- User interaction flows
- Performance with large datasets

## License

MIT License - See LICENSE file for details

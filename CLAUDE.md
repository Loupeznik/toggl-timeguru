# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Toggl TimeGuru is a Rust CLI application for managing and analyzing Toggl Track time entries. It provides an interactive TUI, local SQLite caching, time entry grouping/filtering, and offline support.

**Current Status**: Phase 1 MVP complete. Phase 2 (enhanced functionality) is next.

## Essential Commands

### Development
```bash
# Build (debug)
cargo build

# Build (release) - may fail on low-memory systems
cargo build --release

# Run application
cargo run -- [SUBCOMMAND] [ARGS]

# Example: Run with config subcommand
cargo run -- config --show
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_group_by_description

# Run tests in specific module
cargo test processor::tests
```

### Code Quality
```bash
# Format code (uses rustfmt.toml config)
cargo fmt

# Lint with clippy (must pass with -D warnings)
cargo clippy -- -D warnings

# Check compilation without building
cargo check
```

## Architecture Overview

### Module Structure

The codebase follows a modular architecture with clear separation of concerns:

```
src/
├── main.rs           # Entry point, command routing, async handlers
├── cli.rs            # Clap-based CLI definitions and argument parsing
├── config/           # Configuration management using confy
├── db/               # SQLite database layer
│   ├── connection.rs # Database operations (CRUD, queries)
│   └── schema.rs     # Table definitions and migrations
├── processor.rs      # Time entry processing (grouping, filtering, calculations)
├── toggl/            # Toggl API integration
│   ├── client.rs     # HTTP client with Basic auth
│   └── models.rs     # Serde data models for API responses
└── ui/               # Ratatui-based TUI
    ├── app.rs        # Main TUI app loop and event handling
    └── components.rs # Reusable UI components
```

### Key Architectural Patterns

#### 1. **Custom API Client (src/toggl/)**
No official Toggl SDK exists for Rust, so we implement a custom HTTP client:
- Uses `reqwest` with rustls (no OpenSSL dependency)
- Basic authentication via base64-encoded `{token}:api_token`
- Custom error handling with `anyhow::Context`
- API v9 endpoints: `/me/time_entries`, `/workspaces`, `/projects`

**Adding new API endpoints**: Follow the pattern in `client.rs`:
1. Add async method to `TogglClient`
2. Construct URL with query params
3. Add auth header via `self.auth_header()`
4. Parse response with typed serde model
5. Add error context with `.context()`

#### 2. **Data Flow: API → Database → Processing → UI**

```
Toggl API (reqwest)
    ↓
TimeEntry models (serde)
    ↓
SQLite (rusqlite) - Local cache
    ↓
processor.rs - Grouping/filtering
    ↓
UI/CLI output (ratatui/stdout)
```

**Sync mechanism**:
- `sync` command fetches from API, saves to DB, updates sync metadata
- `list` and `tui` commands load from DB (with optional `--offline` flag to skip API)
- DB acts as cache + offline storage

#### 3. **State Management in TUI**

The `App` struct (src/ui/app.rs) maintains UI state:
- `time_entries: Vec<TimeEntry>` - Raw entries from DB
- `grouped_entries: Vec<GroupedTimeEntry>` - Grouped view (computed in main.rs)
- `list_state: ListState` - Ratatui list selection state
- `show_grouped: bool` - Toggle between grouped/individual views (default: false)
- `show_rounded: bool` - Toggle rounding on/off (default: true)
- `round_minutes: Option<i64>` - Rounding interval from config

**Rounding behavior**:
- **Grouped view**: Rounding is applied when `show_rounded` is true and `round_minutes` is configured
- **Non-grouped view**: Always shows real times (no rounding regardless of toggle)
- User can press 'r' to toggle rounding on/off in real-time
- Default is rounding ON for grouped view

**Event loop**:
1. Render UI (`ui()` method)
2. Wait for keyboard event (`crossterm::event::read()`)
3. Handle event (`handle_key_event()`)
4. Update state (e.g., `next_item()`, `toggle_grouping()`, `toggle_rounding()`)
5. Check `should_quit` flag

#### 4. **Configuration Management**

Uses `confy` for platform-aware config storage:
- Config struct in `src/config/mod.rs` with `Default` trait
- `Config::load()` reads from OS-specific path (e.g., `~/.config/toggl-timeguru/`)
- API token currently stored as `Vec<u8>` (Phase 3 will migrate to OS keyring)

**Adding config fields**:
1. Add field to `Config` struct
2. Update `Default` impl
3. Add CLI flags in `cli.rs` Commands::Config
4. Handle in `handle_config()` in main.rs

#### 5. **Time Entry Grouping Algorithm**

Located in `processor.rs`:
- Groups entries by `(description, project_id)` tuple
- Uses `HashMap<(Option<String>, Option<i64>), Vec<TimeEntry>>`
- Sums durations per group
- Sorts by total duration (descending)

**Duration rounding**: `GroupedTimeEntry::rounded_duration(round_to_minutes)` rounds **up** to the next N-minute interval for cleaner reporting. For example, with 15-minute rounding:
- 22.2 minutes (0.37h) → 30 minutes (0.5h)
- 69.6 minutes (1.16h) → 75 minutes (1.25h)

This uses `ceil()` to always round up, never down.

### Database Schema

SQLite tables in `src/db/schema.rs`:
- **time_entries**: Stores time entries with JSON-serialized tags
- **projects**: Caches project metadata
- **sync_metadata**: Tracks last sync timestamp per resource type

**Important**: Time entries use `INSERT OR REPLACE` to handle updates during sync.

### Error Handling

- Application-level errors use `anyhow::Result<T>`
- Add context to errors: `.context("Description of what failed")?`
- API client errors include status codes and response bodies
- Database errors include SQL context

### Testing Approach

- Unit tests in modules (e.g., `processor::tests`, `toggl::client::tests`)
- Use `mockito` for HTTP mocking (dev-dependency)
- Tests focus on business logic (grouping, filtering, calculations)
- No integration tests yet (planned for Phase 2)

### Async Runtime

- Uses `tokio` with `#[tokio::main]` in main.rs
- All API calls are async
- Database and UI operations are sync (wrapped in async handlers)

## Important Implementation Details

### Date Parsing
`Cli::parse_date()` accepts two formats:
- ISO 8601: `2025-01-15T10:30:00Z`
- Simple: `2025-01-15` (converted to local midnight in UTC)

### API Authentication
Toggl uses HTTP Basic auth with username=API_TOKEN, password="api_token":
```rust
let credentials = format!("{}:api_token", self.api_token);
let encoded = base64::encode(credentials);
header: "Basic {encoded}"
```

### Dead Code Annotations
Several functions/methods have `#[allow(dead_code)]` because they're planned for Phase 2:
- `calculate_billable_duration()`, `calculate_non_billable_duration()`
- `TogglClient::get_workspaces()`, `get_projects()`, `get_current_user()`
- `Database::save_projects()`
- UI helper functions in `components.rs`

**Do not remove these** - they're part of the roadmap.

### TUI Footer Bug
The footer shows inverted text: when grouped view is ON, it says "Toggle grouping (OFF)". This is intentional in the code but reads awkwardly - may need UX improvement in Phase 2.

### Missing Feature: Project Assignment
Currently, the application can only **read** time entries and their associated projects. There is no functionality to **assign** or **reassign** a time entry (or group of entries) to a different project. This would require:
- Adding a PUT endpoint to `TogglClient` for updating time entries
- UI/CLI commands to select entries and assign them to a project
- Potentially batch update support for grouped entries

This is a candidate for Phase 2 or Phase 3 depending on user needs.

## Development Workflow

1. Make changes to code
2. Run `cargo fmt` to format
3. Run `cargo clippy -- -D warnings` to check for issues
4. Run `cargo test` to verify tests pass
5. Run `cargo build` to compile
6. Test manually with `cargo run -- [command]`
7. Commit changes (commits should be concise, present tense, imperative mood)

## Phase Development

**Phase 1 (Complete)**: Core functionality - API client, DB, TUI, grouping, config
**Phase 2 (Next)**: Advanced filtering, reports, CSV export, fuzzy matching, incremental sync
**Phase 3 (Future)**: PDF export, packaging, CI/CD, Docker, OS keyring

See `docs/PROGRESS.md` for detailed task tracking.

## Platform Support

- **Linux/macOS/Windows** all supported
- Config paths handled by `confy` (platform-aware)
- DB paths handled by `dirs` crate (`dirs::data_dir()`)
- Terminal via `crossterm` (cross-platform)

## Troubleshooting

### Release builds fail with SIGKILL
Known issue on low-memory systems. Use debug builds for development:
```bash
cargo build  # Not cargo build --release
```

### Tests fail with "No such table"
Database schema not initialized. Tests should create in-memory DB with `init_database()`.

### TUI doesn't render colors
Terminal might not support true color. Use standard terminal with 256-color support.

## Documentation Files

- `README.md` - User-facing documentation and quick start
- `docs/TECH_STACK.md` - Library choices and rationale
- `docs/PROGRESS.md` - Phase checklist (may be outdated, check git log)
- `docs/INITIAL_FUNCTIONAL_ANALYSIS.md` - Original requirements
- `docs/LIBRARY_RECOMMENDATIONS.md` - Research on Rust crates

## Configuration Files

- `Cargo.toml` - Dependencies and package metadata
- `rustfmt.toml` - Code formatting rules (edition 2024, max_width 100)
- `.clippy.toml` - Clippy configuration (cognitive-complexity-threshold 30)
- `.gitignore` - Standard Rust ignore patterns

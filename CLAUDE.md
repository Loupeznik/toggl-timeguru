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

## Pre-Commit Checklist

**IMPORTANT**: Before committing ANY changes, you MUST run these commands in order:

```bash
# 1. Format code
cargo fmt

# 2. Check for linting issues (MUST pass with no warnings)
cargo clippy -- -D warnings

# 3. Run all tests
cargo test

# 4. Build to verify compilation
cargo build
```

**All four steps must pass before committing.** If clippy fails, fix the warnings before proceeding. This ensures CI/CD checks will pass.

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

**Database Location**:
- The SQLite database is stored at `{data_dir}/toggl-timeguru/timeguru.db`
- `data_dir` is platform-specific:
  - **macOS**: `~/Library/Application Support/toggl-timeguru/timeguru.db`
  - **Linux**: `~/.local/share/toggl-timeguru/timeguru.db`
  - **Windows**: `%APPDATA%\toggl-timeguru\timeguru.db`

**Manual Database Deletion**:
To manually delete the database (e.g., when switching accounts):
```bash
# macOS
rm -rf ~/Library/Application\ Support/toggl-timeguru/

# Linux
rm -rf ~/.local/share/toggl-timeguru/

# Windows (PowerShell)
Remove-Item -Recurse -Force "$env:APPDATA\toggl-timeguru"
```

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

#### **CRITICAL: Calling Async Code from Sync TUI Context**

**Issue**: Cannot use `Handle::block_on()` from within an async runtime context. This causes a panic:
```
Cannot start a runtime from within a runtime. This happens because a function (like `block_on`)
attempted to block the current thread while the thread is being used to drive asynchronous tasks.
```

**Why it happens**:
1. `main()` is marked with `#[tokio::main]`, creating a tokio runtime
2. All command handlers (e.g., `handle_tui()`) are async functions running on that runtime
3. The TUI event loop runs synchronously but on a thread owned by the tokio runtime
4. Calling `Handle::block_on()` tries to block that thread, creating a runtime conflict

**Solution**: Use `Handle::spawn()` + sync channels instead of `block_on()`:

```rust
// ❌ WRONG - This will panic!
let handle = tokio::runtime::Handle::current();
let result = handle.block_on(client.some_async_method());

// ✅ CORRECT - Spawn async task and wait via channel
let (tx, rx) = std::sync::mpsc::channel();
let client_clone = client.clone();

handle.spawn(async move {
    let result = client_clone.some_async_method().await;
    let _ = tx.send(result);
});

match rx.recv() {
    Ok(Ok(value)) => { /* success */ }
    Ok(Err(e)) => { /* API error */ }
    Err(e) => { /* channel error */ }
}
```

**When this pattern is needed**:
- Calling async API methods from synchronous TUI event handlers
- Any sync code that needs to execute async operations while running on a tokio runtime
- Bridge between sync UI code and async backend operations

**Reference Implementation**: See `src/ui/app.rs::assign_project_to_entry()` for the complete pattern (lines 672-714 and 761-801).

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

### Project Assignment Feature (v1.1.1)
The TUI now supports project assignment for both individual and grouped time entries:
- Press 'p' to open the project selector panel
- Navigate with arrow keys or vim keys (j/k)
- Search projects by typing '/' followed by search term
- Press Enter to assign the selected project to the current time entry
- **Individual view**: Assigns project to the selected single entry
- **Grouped view**: Batch assigns project to ALL entries in the selected group
- Uses `Arc<TogglClient>` with `tokio::runtime::Handle::spawn()` + sync channels to call async API from sync TUI
- See "CRITICAL: Calling Async Code from Sync TUI Context" section for implementation details

## v1.1.1 Implemented Features

### Multi-Account Support ✅ IMPLEMENTED
The application now supports multiple Toggl accounts with automatic detection:
- User ID filtering in database queries (with index for performance)
- Visual indicator in TUI showing current account email
- Auto-detect account switching with helpful messages
- Database data is automatically filtered by user_id

### Data Management CLI ✅ IMPLEMENTED
The `clean` command allows safe deletion of application data:
```bash
toggl-timeguru clean --all          # Delete database + config
toggl-timeguru clean --data         # Delete only database
toggl-timeguru clean --config       # Delete only config
toggl-timeguru clean --confirm      # Skip confirmation prompt
```

### CSV Export with Day-based Grouping ✅ IMPLEMENTED
CSV export now supports multiple grouping options:
```bash
toggl-timeguru export --output file.csv --group           # Group by description
toggl-timeguru export --output file.csv --group-by-day    # Group by description within each day
```

## Known Issues and Limitations

No major known issues at this time. Please report any bugs via GitHub issues.

## Development Workflow

1. Make changes to code
2. **Run the Pre-Commit Checklist** (see above section):
   - `cargo fmt` - Format code
   - `cargo clippy -- -D warnings` - Check for linting issues (must pass)
   - `cargo test` - Verify tests pass
   - `cargo build` - Verify compilation
3. Test manually with `cargo run -- [command]`
4. Commit changes (commits should be concise, present tense, imperative mood)
5. **IMPORTANT**: When completing features, update progress tracking in BOTH:
   - `docs/PROGRESS.md` - Mark tasks as completed with [x]
   - `docs/VERSION_TIMELINE.md` - Mark version sections as completed with checkmarks

**Note**: The Pre-Commit Checklist is mandatory. All checks must pass before pushing to remote.

### Feature Request Documentation

**CRITICAL**: All new feature requests or enhancements MUST be documented in the implementation docs:

1. **For new features**: Add to `docs/PROGRESS.md` AND `docs/VERSION_TIMELINE.md` under the appropriate phase or version
2. **Version assignment**: Assign features to specific versions (e.g., v1.1.1, v1.2.0) based on priority and scope
3. **Track status**: Mark features as `PLANNED`, `IN PROGRESS`, or `✅ COMPLETED`
4. **Include details**: Add sub-tasks with checkboxes for tracking granular progress
5. **Update both files**: Always keep both PROGRESS.md and VERSION_TIMELINE.md in sync

**Example**: When a user requests "multi-account support", add it to:
- `docs/PROGRESS.md` under appropriate Phase 2 section with detailed sub-tasks
- `docs/VERSION_TIMELINE.md` under the target version (e.g., v1.1.1)

This ensures all feature requests are tracked and prioritized properly.

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

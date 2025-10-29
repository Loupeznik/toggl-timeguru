# Toggl TimeGuru

A powerful CLI tool for managing and analyzing Toggl Track time entries, built with Rust.

## Features

- **Time tracking** from command line - start and stop time entries directly
- **Sync time entries** from Toggl Track to local SQLite database
- **Interactive TUI** for browsing time entries with vim-style navigation
- **Project assignment** directly from TUI with search and batch operations
- **Time entry editing** in TUI for updating descriptions with batch support
- **Group entries** by description or by description within each day
- **Filter entries** by date range (project and tag filtering via CLI only)
- **CSV export** with grouping options for external reporting
- **Duration rounding** (rounds up to next interval) for easy time reporting
- **Multi-account support** with automatic account detection and switching
- **Data management** CLI commands for cleaning database and config
- **Offline support** via local caching
- **Fast and efficient** native Rust performance

## Installation

### From Source

```bash
cargo build --release
sudo cp target/release/toggl-timeguru /usr/local/bin/
```

## Quick Start

1. Get your Toggl API token from [Toggl Track Profile Settings](https://track.toggl.com/profile)

2. Configure the application:
```bash
toggl-timeguru config --set-token YOUR_API_TOKEN
```

3. Sync your time entries:
```bash
toggl-timeguru sync
```

4. Launch the interactive TUI:
```bash
toggl-timeguru tui
```

## Usage

### Commands

#### `config` - Configure the application

```bash
# Set API token
toggl-timeguru config --set-token YOUR_TOKEN

# Set default date range (in days)
toggl-timeguru config --set-date-range 7

# Set duration rounding (in minutes, rounds UP to next interval)
# Example: 15 rounds to quarter hours (0.25h, 0.5h, 0.75h, 1.0h, etc.)
toggl-timeguru config --set-round-minutes 15

# Show current configuration
toggl-timeguru config --show
```

#### `sync` - Sync time entries from Toggl

```bash
# Sync last 90 days (default)
toggl-timeguru sync

# Sync specific date range
toggl-timeguru sync --start 2025-01-01 --end 2025-01-31
```

#### `list` - List time entries

```bash
# List entries for the last 7 days (default)
toggl-timeguru list

# List with grouping by description
toggl-timeguru list --group

# Filter by project ID
toggl-timeguru list --project 12345

# Filter by tag
toggl-timeguru list --tag "client-work"

# Use offline/cached data
toggl-timeguru list --offline

# Custom date range
toggl-timeguru list --start 2025-01-01 --end 2025-01-31
```

#### `tui` - Interactive terminal UI

```bash
# Launch TUI with default date range
toggl-timeguru tui

# Launch TUI with custom date range
toggl-timeguru tui --start 2025-01-01 --end 2025-01-31
```

**TUI Keyboard Shortcuts:**
- `↑`/`k` - Move up
- `↓`/`j` - Move down
- `PageUp`/`PageDown` - Jump by page
- `Home`/`End` - Jump to first/last entry
- `g` - Toggle grouping by description
- `d` - Toggle day-based grouping (groups by description within each day)
- `s` - Toggle date sorting (ascending/descending)
- `r` - Toggle rounding on/off (default: ON in grouped view)
- `p` - Open project selector to assign project (works on individual or grouped entries)
- `e` - Edit description (works on individual or grouped entries, batch edit supported)
- `y` - Copy selected entry description to clipboard
- `q`/`Esc` - Quit

#### `export` - Export time entries to CSV

```bash
# Export entries to CSV (individual entries)
toggl-timeguru export --start 2025-01-01 --end 2025-01-31 --output report.csv

# Export with grouping by description
toggl-timeguru export --output report.csv --group

# Export with day-based grouping (groups by description within each day)
toggl-timeguru export --output report.csv --group-by-day

# Include metadata header (date range, user email, entry count)
toggl-timeguru export --output report.csv --include-metadata
```

#### `clean` - Delete application data

```bash
# Delete all data (database + config)
toggl-timeguru clean --all

# Delete only database (keeps config)
toggl-timeguru clean --data

# Delete only configuration (keeps database)
toggl-timeguru clean --config

# Skip confirmation prompt (useful for automation)
toggl-timeguru clean --all --confirm
```

#### `track` - Start and stop time tracking

```bash
# Start a new time entry with description
toggl-timeguru track start --message "Working on feature X"

# Start a new time entry without description
toggl-timeguru track start

# Stop the currently running time entry
toggl-timeguru track stop
```

**Note:** The track command works directly with the Toggl API and requires an active internet connection.

### Global Options

```bash
# Use custom API token for single command
toggl-timeguru --api-token TOKEN sync

# Enable verbose logging
toggl-timeguru -v tui
```

## Configuration

Configuration is stored in platform-specific locations:
- **Linux**: `~/.config/toggl-timeguru/config.toml`
- **macOS**: `~/Library/Application Support/toggl-timeguru/config.toml`
- **Windows**: `%APPDATA%\toggl-timeguru\config.toml`

The SQLite database is stored in:
- **Linux**: `~/.local/share/toggl-timeguru/timeguru.db`
- **macOS**: `~/Library/Application Support/toggl-timeguru/timeguru.db`
- **Windows**: `%APPDATA%\toggl-timeguru\timeguru.db`

## Troubleshooting

### Deleting Application Data

To manually delete the application database (useful when switching Toggl accounts):

**macOS:**
```bash
rm -rf ~/Library/Application\ Support/toggl-timeguru/
```

**Linux:**
```bash
rm -rf ~/.local/share/toggl-timeguru/
```

**Windows (PowerShell):**
```powershell
Remove-Item -Recurse -Force "$env:APPDATA\toggl-timeguru"
```

**Note**: To also delete the configuration file, remove the config directory:
- **macOS**: `rm -rf ~/Library/Application\ Support/toggl-timeguru/config.toml`
- **Linux**: `rm -rf ~/.config/toggl-timeguru/`
- **Windows**: `Remove-Item -Recurse -Force "$env:APPDATA\toggl-timeguru\config.toml"`

### Multi-Account Support

The application automatically detects when you switch between Toggl API tokens (different accounts):
- Database entries are automatically filtered by user_id
- The TUI displays your current account email in the header
- When switching accounts, you'll see a helpful message with cleanup instructions
- Use `toggl-timeguru clean --data` to remove old account data if needed

## Development

### Requirements

- Rust 1.89.0 or newer
- Cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy -- -D warnings
```

### Project Structure

```
src/
├── cli.rs          # Command-line interface definitions
├── config/         # Configuration management
├── db/             # SQLite database operations
│   ├── connection.rs
│   └── schema.rs
├── processor.rs    # Time entry processing logic
├── toggl/          # Toggl API client
│   ├── client.rs
│   └── models.rs
├── ui/             # Terminal user interface
│   ├── app.rs
│   └── components.rs
└── main.rs         # Application entry point
```

## Roadmap

See [docs/PROGRESS.md](docs/PROGRESS.md) for detailed development progress.

### Phase 1: MVP (Completed)
- ✅ Core Toggl API integration
- ✅ Local SQLite caching
- ✅ Basic TUI with time entry display
- ✅ Time entry grouping and filtering
- ✅ Configuration management

### Phase 2: Enhanced Functionality (In Progress)
- ✅ Advanced filtering (CLI only - project, tags via list command)
- ✅ CSV export with grouping options
- ✅ Enhanced UI with better navigation
- ✅ Project assignment in TUI
- ✅ Time entry editing in TUI
- ✅ Multi-account support
- ✅ Data management CLI
- ✅ Time tracking CLI (start/stop commands)
- ✅ CI/CD & Build Automation
- Interactive TUI filtering (project, tags, client)
- Report generation (daily, weekly, monthly)
- Fuzzy description matching
- Incremental sync

### Phase 3: Additional Features
- PDF export
- In-app help system
- Cross-platform packaging
- CI/CD pipeline
- Docker support

## Tech Stack

- **Language**: Rust (edition 2024)
- **TUI Framework**: Ratatui + Crossterm
- **HTTP Client**: Reqwest (with rustls)
- **Database**: SQLite (via rusqlite)
- **CLI Parsing**: Clap
- **Configuration**: Confy
- **Serialization**: Serde
- **Date/Time**: Chrono
- **Error Handling**: Anyhow
- **Logging**: Tracing

See [docs/TECH_STACK.md](docs/TECH_STACK.md) for complete details.

## Documentation

- [Initial Functional Analysis](docs/INITIAL_FUNCTIONAL_ANALYSIS.md)
- [Library Recommendations](docs/LIBRARY_RECOMMENDATIONS.md)
- [Tech Stack Details](docs/TECH_STACK.md)
- [Development Progress](docs/PROGRESS.md)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Dominik Zarsky <dzarsky@dzarsky.eu>

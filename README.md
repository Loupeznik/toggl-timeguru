# Toggl TimeGuru

A powerful CLI tool for managing and analyzing Toggl Track time entries, built with Rust.

## Features

- **Sync time entries** from Toggl Track to local SQLite database
- **Interactive TUI** for browsing time entries with vim-style navigation
- **Group entries** by description with automatic duration summation
- **Filter entries** by date range, project, and tags
- **Duration rounding** (rounds up to next interval) for easy time reporting
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
- `g` - Toggle grouping on/off
- `r` - Toggle rounding on/off (default: ON in grouped view)
- `q`/`Esc` - Quit

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

### Known Limitations

1. **Multiple Accounts**: The application currently does not support multiple Toggl accounts. When switching API tokens, manually delete the database first (see above) to avoid mixing data from different accounts.

2. **Data Management**: There is currently no CLI command to reset or clean application data. This feature is planned for a future release.

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

### Phase 2: Enhanced Functionality
- Advanced filtering (project, tags, client)
- Report generation (daily, weekly, monthly)
- CSV export
- Enhanced UI with better navigation
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

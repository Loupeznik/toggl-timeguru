# Toggl TimeGuru - Development Progress

This document tracks the development progress across all phases of the Toggl TimeGuru project.

## Phase 1: MVP - Core Functionality ✅ COMPLETED

### Setup & Foundation
- [x] Initialize Rust project with Cargo
- [x] Configure dependencies in Cargo.toml
- [x] Create project documentation structure
- [x] Set up project module structure (src/toggl/, src/ui/, src/db/, src/config/)
- [x] Configure tracing for structured logging
- [x] Set up rustfmt for code formatting
- [x] Configure clippy for linting
- [x] Set up pre-commit hooks or development scripts
- [x] Add formatting and linting checks to development workflow

### Toggl API Integration
- [x] Implement Toggl API client module
- [x] Add authentication via API token
- [x] Implement fetch time entries endpoint
- [x] Add basic error handling for API responses
- [x] Create data models for time entries using serde

### Configuration Management
- [x] Design configuration structure
- [x] Implement config loading/saving with confy
- [x] Add API token storage with encryption using ring
- [x] Store user preferences (default date range, report format)
- [x] Create default configuration generator

### Data Storage
- [x] Design SQLite database schema
- [x] Implement database connection module
- [x] Create time_entries table
- [x] Add basic CRUD operations for time entries

### Terminal UI
- [x] Set up ratatui with crossterm backend
- [x] Create main application loop
- [x] Implement basic time entries list view
- [x] Add simple navigation (up/down, quit)
- [x] Display time entry details (description, duration, project)
- [x] Add grouping toggle (g key)
- [x] Add day-based grouping toggle (d key)
- [x] Add date sorting toggle (s key)
- [x] Add rounding toggle (r key)

### Time Entry Processing
- [x] Implement date range filtering
- [x] Add time entry grouping by exact description match
- [x] Add day-based grouping (groups by description, project, and date)
- [x] Calculate total duration for grouped entries
- [x] Implement duration rounding functionality (rounds UP)
- [x] Add date sorting for day-grouped entries

### CLI Interface
- [x] Design command-line arguments structure with clap
- [x] Add date range parameters
- [x] Add configuration file path option
- [x] Implement help text and usage examples

### Testing & Polish
- [x] Add unit tests for API client
- [x] Test time entry grouping logic
- [x] Test rounding logic with comprehensive unit tests
- [x] Basic error handling improvements
- [x] Create initial README with setup instructions
- [x] Fix date format and timezone issues
- [x] All clippy warnings resolved

## Phase 2: Enhanced Functionality

### v1.1.0 Bugfixes & Quick Wins ✅ COMPLETED
- [x] Fix Windows TUI navigation bug where k/j keys skip two rows instead of one (filter KeyEventKind::Press)
- [x] Add clipboard copy functionality for time entry descriptions (hotkey 'y' using arboard crate)

### v1.1.1 Project Assignment ✅ COMPLETED
- [x] Implement TUI project selector panel with color-coded projects
- [x] Add project assignment for individual time entries (hotkey 'p')
- [x] Add project search functionality (hotkey '/' in selector)
- [x] Add navigation shortcuts (j/k, PageUp/PageDown, Home/End)
- [x] Batch assignment support for grouped entries
- [x] API support: update_time_entry_project() method added to TogglClient
- [x] Async/sync integration using Arc<TogglClient> with Handle::spawn()
- [x] Status messages for success/failure feedback

### v1.1.1 Data Management & Multi-Account Support ✅ COMPLETED
- [x] Add CLI command for data deletion (`toggl-timeguru clean`)
  - [x] `clean --all` - Delete both database and config
  - [x] `clean --data` - Delete only database
  - [x] `clean --config` - Delete only configuration
  - [x] `clean --confirm` - Skip confirmation prompt for automation
  - [x] Cross-platform support (macOS/Linux/Windows)
- [x] Implement multi-account support
  - [x] Add user_id filtering to database queries
  - [x] Add user_id index for query performance
  - [x] Single database with account filtering
  - [x] Add visual indicator in TUI showing current account (displays email)
  - [x] Auto-detect account switching with improved messages
- [x] Document database location and manual deletion procedures

### v1.1.1 CSV Export ✅ COMPLETED
- [x] Implement CSV export using csv crate
- [x] Add grouped export format
- [x] Add day-based grouped export format (--group-by-day)
- [x] Add individual entry export format
- [x] Include optional metadata (date range, user_id, entry count)
- [x] Support project name resolution in exports

### CI/CD & Build Automation ✅ COMPLETED (v1.1.2)
- [x] Set up GitHub Actions workflow (.github/workflows/ci.yml)
- [x] Configure multi-platform builds (Linux/macOS/Windows × amd64/arm64)
- [x] Add automated testing on push/PR (test, clippy, fmt jobs)
- [x] Implement automated GitHub Releases with binaries (.github/workflows/release.yml)
- [x] Add binary stripping and compression for smaller downloads
- [x] Set up clippy and rustfmt checks in CI
- [x] Add code coverage reporting (cargo-tarpaulin + Codecov)

### Advanced Filtering ✅ PARTIALLY COMPLETED (v1.1.0 - CLI Only)
- [x] Add project-based filtering (CLI `list --project` command)
- [x] Implement tag-based filtering (CLI `list --tag` command)
- [x] Add client-based filtering (backend TimeEntryFilter support)
- [x] Create filter combination logic
- [ ] Add interactive TUI filter panel UI (planned for v1.1.2)
- [ ] Add project filtering in TUI
- [ ] Add tag filtering in TUI
- [ ] Add client filtering in TUI

### v1.1.2 Data Persistence Improvements (PLANNED)
- [ ] Fix project assignment persistence issue
  - [ ] Update database record when project is assigned in TUI
  - [ ] Ensure changes persist across TUI sessions without manual sync
  - [ ] Use Database::update_time_entry_project() method

### v1.1.2 Time Tracking CLI (PLANNED)
- [ ] Create new `track` command in CLI
  - [ ] Add `track start` subcommand with `-m/--message` flag for description
  - [ ] Add `track stop` subcommand to end current running entry
  - [ ] Integrate with Toggl API start_time_entry() and stop_time_entry()
  - [ ] Add validation and error handling for API calls

### v1.1.2 TUI Time Entry Editing (PLANNED)
- [ ] Add time entry rename functionality in TUI
  - [ ] Assign hotkey for rename action (e.g., 'e' for edit)
  - [ ] Implement description input prompt/dialog
  - [ ] Update description via Toggl API
  - [ ] Save updated description to local database immediately
  - [ ] Support renaming in both individual and grouped views
  - [ ] Add visual feedback for success/failure

### Report Generation
- [ ] Implement daily summary report
- [ ] Add weekly summary report
- [ ] Create monthly summary report
- [ ] Add project-specific reports
- [ ] Calculate billable vs non-billable hours

### Data Export
- [ ] Implement CSV export using csv crate
- [ ] Add customizable CSV format options
- [ ] Include metadata in exports (date range, filters)

### Enhanced UI ✅ PARTIALLY COMPLETED (v1.1.0)
- [x] Improve navigation (page up/down, home/end)
- [x] Add filter UI panel (billable filter only, full filtering planned for v1.1.2)
- [ ] Create report selection interface
- [x] Implement status bar with help hints
- [ ] Add loading indicators for API calls

### Local Caching ✅ PARTIALLY COMPLETED (v1.1.0)
- [x] Implement sync mechanism for time entries
- [x] Add last sync timestamp tracking
- [x] Create offline mode support
- [ ] Implement incremental sync (only fetch new entries)
- [ ] Add cache invalidation logic

### Fuzzy Matching
- [ ] Integrate strsim or fuzzy-matcher crate
- [ ] Implement similar description matching
- [ ] Add similarity threshold configuration
- [ ] Create preview for fuzzy matches before grouping

### Testing & Error Handling ✅ PARTIALLY COMPLETED (v1.1.0)
- [x] Expand unit test coverage
- [ ] Add integration tests with mocked API
- [x] Improve error messages with context
- [x] Add retry logic for API failures
- [x] Implement rate limiting handling

### Logging ✅ COMPLETED (v1.1.0)
- [x] Add detailed debug logging
- [x] Configure log levels via environment variable
- [x] Log API requests/responses in debug mode
- [x] Add performance metrics logging

## Phase 3: Additional Features

### TUI Testing
- [ ] Research Ratatui TestBackend for TUI testing
- [ ] Add unit tests for keyboard event handlers
- [ ] Add integration tests for state transitions (grouping toggle, rounding toggle)
- [ ] Implement UI snapshot tests for rendering output
- [ ] Test navigation edge cases (empty lists, wrapping)
- [ ] Add tests for footer status display

### Security Enhancements
- [ ] Evaluate keyring crate for API token storage
- [ ] Implement OS-native credential storage
- [ ] Migrate from file-based encryption to keyring (optional)

### PDF Export
- [ ] Research PDF generation options in Rust
- [ ] Evaluate external tools vs native library
- [ ] Implement basic PDF report generation
- [ ] Add customizable PDF templates

### Documentation
- [ ] Create comprehensive user guide
- [ ] Add configuration examples
- [ ] Document all CLI commands and options
- [ ] Create troubleshooting guide
- [ ] Add API integration documentation

### Help System
- [ ] Implement in-app help viewer
- [ ] Add contextual help for each view
- [ ] Create keyboard shortcuts reference
- [ ] Add command palette or search

### Cross-Platform Testing
- [ ] Test on Linux (Ubuntu/Debian)
- [ ] Test on macOS
- [ ] Test on Windows
- [ ] Fix platform-specific issues
- [ ] Verify configuration paths on all platforms

### Advanced Preferences
- [ ] Add default filter presets
- [ ] Implement custom report format templates
- [ ] Add theme/color customization
- [ ] Create keyboard shortcut customization

### Packaging
- [ ] Create snap package
- [ ] Add Homebrew formula
- [ ] Create Debian package (.deb)
- [ ] Publish to crates.io

### Dockerization
- [ ] Create Dockerfile
- [ ] Optimize image size with multi-stage build
- [ ] Test Docker image on multiple platforms
- [ ] Publish to Docker Hub
- [ ] Document Docker usage

## Future Considerations

### Potential Phase 4+ Features
- [ ] Support for multiple Toggl workspaces
- [ ] Time entry editing via CLI
- [ ] Starting/stopping time entries from CLI
- [ ] Integration with other time tracking services
- [ ] Custom report plugins
- [ ] Web-based report viewer
- [ ] Automated report scheduling
- [ ] Team collaboration features
- [ ] Budget tracking and alerts

## Notes

- This document should be updated regularly as tasks are completed
- Each phase should be fully completed and tested before moving to the next
- Dependencies between tasks should be carefully managed
- User feedback should be incorporated between phases

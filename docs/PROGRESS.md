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

### v1.1.2 Data Persistence Improvements ✅ COMPLETED
- [x] Fix project assignment persistence issue
  - [x] Update database record when project is assigned in TUI
  - [x] Ensure changes persist across TUI sessions without manual sync
  - [x] Use Database::update_time_entry_project() method
  - [x] Made Database thread-safe with Mutex<Connection> for Arc sharing
  - [x] Added immediate database updates after successful API calls

### v1.1.2 TUI Error Display ✅ COMPLETED
- [x] Fix critical bug where error messages corrupt TUI display
  - [x] Implement error popup modal for displaying API errors
  - [x] Separate error_message field from status_message
  - [x] Add proper error wrapping for long messages
  - [x] Add keyboard controls (Enter/Esc) to dismiss error popups
  - [x] Prevent errors with newlines from breaking layout

### v1.1.2 TUI Time Entry Editing ✅ COMPLETED
- [x] Add time entry edit functionality in TUI
  - [x] Assign hotkey for edit action ('e' hotkey)
  - [x] Implement description input modal with text field
  - [x] Update description via Toggl API (update_time_entry_description method)
  - [x] Save updated description to local database immediately
  - [x] Support editing in both individual and grouped views
  - [x] Batch edit support for grouped entries (edits all entries in group)
  - [x] Add visual feedback for success/failure (status and error messages)
  - [x] Real-time text input handling (typing, backspace)
  - [x] Proper async/sync integration with Arc<TogglClient> and Handle::spawn()

### v1.1.2 Time Tracking CLI ✅ COMPLETED
- [x] Create new `track` command in CLI
  - [x] Add `track start` subcommand with `-m/--message` flag for description
  - [x] Add `track stop` subcommand to end current running entry
  - [x] Integrate with Toggl API start_time_entry() and stop_time_entry()
  - [x] Add get_current_time_entry() API method for checking running entries
  - [x] Add validation and error handling for API calls
  - [x] Handle edge cases (no running entry, starting without description)

### Logging ✅ COMPLETED (v1.1.0)
- [x] Add detailed debug logging
- [x] Configure log levels via environment variable
- [x] Log API requests/responses in debug mode
- [x] Add performance metrics logging

### Testing & Error Handling ✅ PARTIALLY COMPLETED (v1.1.0)
- [x] Expand unit test coverage
- [ ] Add integration tests with mocked API
- [x] Improve error messages with context
- [x] Add retry logic for API failures
- [x] Implement rate limiting handling

## Phase 2.5: v1.2.x - Reports, Filtering & Search

### v1.2.0 Report Generation (PLANNED)
- [ ] Implement daily summary report
  - [ ] Show total hours worked per day
  - [ ] Group by project with subtotals
  - [ ] Include billable vs non-billable breakdown
- [ ] Add weekly summary report
  - [ ] Weekly totals by project
  - [ ] Daily breakdown within week
- [ ] Create monthly summary report
  - [ ] Monthly totals by project
  - [ ] Weekly breakdown within month
- [ ] Add project-specific reports
  - [ ] Filter by single project
  - [ ] Show detailed breakdown
- [ ] Calculate billable vs non-billable hours
  - [ ] Add to all report types
  - [ ] Show percentages

### v1.2.0 Interactive TUI Filtering (PLANNED)
- [ ] Add project filtering UI to TUI filter panel
  - [ ] Multi-select project filter
  - [ ] Visual indication of active filters
- [ ] Add tag filtering UI to TUI filter panel
  - [ ] Multi-select tag filter
  - [ ] Show tag counts
- [ ] Add client filtering UI to TUI filter panel
  - [ ] Single-select client filter
- [ ] Add filter persistence across TUI sessions
  - [ ] Save active filters to config
  - [ ] Restore on next launch
- [ ] Add visual indicators for active filters in entry list
  - [ ] Badge showing filter count
  - [ ] Highlight filtered entries

### v1.2.0 Project Selector Enhancements (PLANNED)
- [ ] Sort projects by usage in last month
  - [ ] Count time entries per project in last 30 days
  - [ ] Sort by entry count (most used first)
  - [ ] Show usage count in selector
- [ ] Show usage statistics per project
  - [ ] Display percentage of total time
  - [ ] Show entry count
- [ ] Add configuration option to toggle sort method
  - [ ] Sort by name (default/existing)
  - [ ] Sort by usage (new option)

### v1.2.1 Instant Project Search (PLANNED)
- [ ] Type-to-filter in project selector (no '/' needed)
  - [ ] Start filtering on any character input
  - [ ] Real-time filtering as user types
  - [ ] Show "Type to search..." hint
- [ ] Clear search query with Esc
  - [ ] Reset to full project list
  - [ ] Clear search input
- [ ] Preserve existing '/' search for compatibility
  - [ ] Keep old search method working
  - [ ] Allow both methods

### v1.2.1 Fuzzy Matching (PLANNED)
- [ ] Integrate strsim or fuzzy-matcher crate
  - [ ] Evaluate both libraries
  - [ ] Choose based on performance
- [ ] Implement similar description matching for grouping
  - [ ] Calculate similarity scores
  - [ ] Group similar entries together
- [ ] Add similarity threshold configuration
  - [ ] Configurable threshold (0.0-1.0)
  - [ ] Default to 0.8
- [ ] Create preview for fuzzy matches before grouping
  - [ ] Show suggested groups
  - [ ] Allow user to confirm/reject

### v1.2.1 Report Selection Interface (PLANNED)
- [ ] Create report selection interface in TUI
  - [ ] Add hotkey to open report menu (e.g., 'r')
  - [ ] List available report types
- [ ] Allow selecting report type and date range
  - [ ] Daily/Weekly/Monthly selector
  - [ ] Custom date range picker
- [ ] Display report in TUI or export
  - [ ] Show in popup or new view
  - [ ] Option to export to CSV/PDF

### v1.2.2 Incremental Sync (PLANNED)
- [ ] Implement incremental sync
  - [ ] Track last sync timestamp per resource
  - [ ] Only fetch entries modified since last sync
  - [ ] Use Toggl API since parameter
- [ ] Add cache invalidation logic
  - [ ] Invalidate on manual sync request
  - [ ] Invalidate on date range change
- [ ] Optimize sync performance for large date ranges
  - [ ] Batch API requests
  - [ ] Progress indicator

### v1.2.2 Performance Optimizations (PLANNED)
- [ ] Profile TUI rendering performance
  - [ ] Identify slow rendering paths
  - [ ] Optimize hot paths
- [ ] Optimize database queries
  - [ ] Add missing indexes
  - [ ] Optimize complex queries
- [ ] Add loading indicators for API calls
  - [ ] Show spinner during sync
  - [ ] Show progress for long operations

### Enhanced UI ✅ PARTIALLY COMPLETED (v1.1.0)
- [x] Improve navigation (page up/down, home/end)
- [x] Add filter UI panel (billable filter only, full filtering planned for v1.2.0)
- [ ] Create report selection interface (moved to v1.2.1)
- [x] Implement status bar with help hints
- [ ] Add loading indicators for API calls (moved to v1.2.2)

### Local Caching ✅ PARTIALLY COMPLETED (v1.1.0)
- [x] Implement sync mechanism for time entries
- [x] Add last sync timestamp tracking
- [x] Create offline mode support
- [ ] Implement incremental sync (moved to v1.2.2)
- [ ] Add cache invalidation logic (moved to v1.2.2)

### Data Export ✅ COMPLETED (v1.1.1)
- [x] Implement CSV export using csv crate
- [x] Add customizable CSV format options
- [x] Include metadata in exports (date range, filters)

## Phase 3: v1.3.x - Quality & Testing

### v1.3.0 TUI Testing (PLANNED)
- [ ] Research Ratatui TestBackend for TUI testing
- [ ] Add unit tests for keyboard event handlers
- [ ] Add integration tests for state transitions (grouping toggle, rounding toggle)
- [ ] Implement UI snapshot tests for rendering output
- [ ] Test navigation edge cases (empty lists, wrapping)
- [ ] Add tests for footer status display

### v1.3.0 Cross-Platform Testing (PLANNED)
- [ ] Test on Linux (Ubuntu/Debian)
- [ ] Test on macOS
- [ ] Test on Windows
- [ ] Fix platform-specific issues
- [ ] Verify configuration paths on all platforms

### v1.3.1 Security Enhancements (PLANNED)
- [ ] Evaluate keyring crate for API token storage
- [ ] Implement OS-native credential storage
- [ ] Migrate from file-based encryption to keyring (optional)

### v1.3.1 Documentation (PLANNED)
- [ ] Create comprehensive user guide
- [ ] Add configuration examples
- [ ] Document all CLI commands and options
- [ ] Create troubleshooting guide
- [ ] Add API integration documentation

### v1.3.1 Help System (PLANNED)
- [ ] Implement in-app help viewer
- [ ] Add contextual help for each view
- [ ] Create keyboard shortcuts reference
- [ ] Add command palette or search

## Phase 4: v1.4.x - Export & Customization

### v1.4.0 PDF Export (PLANNED)
- [ ] Research PDF generation options in Rust
- [ ] Evaluate external tools vs native library
- [ ] Implement basic PDF report generation
- [ ] Add customizable PDF templates

### v1.4.0 Advanced Preferences (PLANNED)
- [ ] Add default filter presets
- [ ] Implement custom report format templates
- [ ] Add theme/color customization
- [ ] Create keyboard shortcut customization

## Phase 5: v1.5.x - Distribution

### v1.5.0 Packaging (PLANNED)
- [ ] Create snap package
- [ ] Add Homebrew formula
- [ ] Create Debian package (.deb)
- [ ] Publish to crates.io

### v1.5.0 Dockerization (PLANNED)
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

# Version Timeline

## v1.0.0 - Initial Release ✅ COMPLETED
- Core Toggl API integration
- Local SQLite caching
- Interactive TUI with grouping/rounding
- Configuration management
- Basic CLI commands

## v1.1.0 - Advanced Filtering & Enhanced UI ✅ COMPLETED
### Advanced Filtering (CLI Only)
- [x] Add project-based filtering (via `list --project` command)
- [x] Implement tag-based filtering (via `list --tag` command)
- [x] Add client-based filtering (backend support in TimeEntryFilter)
- [x] Create filter combination logic (TimeEntryFilter builder)
- Note: TUI filtering UI not implemented yet (planned for v1.1.2)

### Enhanced UI
- [x] Improve navigation (page up/down, home/end)
- [x] Implement enhanced status bar with help hints (two-line footer)
- [x] Add entry counter and date range display
- [x] Add interactive filter panel in TUI
- [x] Display project names with color coding in entry rows
- [x] Sync projects to SQLite database

### Grouping & Sorting
- [x] Add day-based grouping toggle (hotkey 'd')
- [x] Implement grouping by (description, project, date) tuple
- [x] Add date sorting toggle for both grouped and individual entries (hotkey 's')
- [x] Grouping preserves input order (sorted entries result in sorted groups)

### Improved Logging
- [x] Add detailed debug logging throughout codebase
- [x] Configure log levels via RUST_LOG environment variable
- [x] Log API requests/responses in debug mode
- [x] Add structured logging with tracing subscriber

### Testing & Error Handling
- [x] Expand unit test coverage (21 tests total)
- [x] Add tests for filtering (project, tag, client, combined, billable)
- [x] Add tests for day-based grouping (3 tests)
- [x] Add tests for date sorting (3 tests)
- [x] Improve error messages with context using anyhow
- [x] Add retry logic for API failures (exponential backoff, 3 retries)
- [x] Implement rate limiting handling (429 status code)

## v1.1.0 - Bugfixes & Quick Wins ✅ COMPLETED
### Bugfixes
- [x] Fix Windows TUI navigation bug where k/j keys skip two rows instead of one

### Quick Win Features
- [x] Add clipboard copy functionality for time entry descriptions (hotkey 'y')

## v1.1.1 - Project Assignment & Data Management ✅ COMPLETED
### Project Assignment Feature ✅ COMPLETED
- [x] Implement TUI project selector panel
- [x] Add project assignment for individual time entries (hotkey 'p')
- [x] Add project search functionality (hotkey '/')
- [x] Add navigation shortcuts (j/k, PageUp/PageDown, Home/End)
- [x] Batch assignment support for grouped entries
- [x] API support: update_time_entry_project() method added
- [x] Async/sync integration: Using Arc<TogglClient> with Handle::spawn()

### Data Management & Multi-Account Support ✅ COMPLETED
- [x] Add CLI command for data deletion (`toggl-timeguru clean`)
  - [x] `clean --all` - Delete database + config
  - [x] `clean --data` - Delete only database
  - [x] `clean --config` - Delete only config
  - [x] `clean --confirm` - Skip confirmation prompt
- [x] Implement multi-account support
  - [x] Store user_id with database entries
  - [x] Add user_id index for better query performance
  - [x] Filter database queries by user_id
  - [x] Visual indicator in TUI showing current account (displays email)
  - [x] Auto-detect account switching with improved messages

### Data Export ✅ COMPLETED
- [x] Implement CSV export using csv crate
- [x] Add customizable CSV format options (grouped vs individual)
- [x] Add day-based grouped export format (--group-by-day)
- [x] Include metadata in exports (date range, user_id, entry count)

## v1.1.2 - TUI Enhancements & Time Tracking (IN PROGRESS)
### Data Persistence Improvements ✅ COMPLETED
- [x] Fix project assignment persistence (save to local DB immediately on assignment)
- [x] Update database record when project is assigned to time entry/group
- [x] Ensure TUI changes persist across sessions without manual sync

### TUI Error Display ✅ COMPLETED
- [x] Fix error message display corruption in TUI
- [x] Implement error popup modal for displaying API errors
- [x] Prevent long error messages from breaking TUI layout
- [x] Add keyboard controls (Enter/Esc) to close error popups

### Time Tracking CLI
- [ ] Create new `track` command for starting/stopping time entries
- [ ] Add `track start` subcommand with `-m/--message` flag for description
- [ ] Add `track stop` subcommand to end current time entry
- [ ] Integrate with Toggl API for real-time tracking

### TUI Time Entry Editing
- [ ] Add time entry rename functionality in TUI (new hotkey)
- [ ] Update description in both Toggl API and local database
- [ ] Add visual feedback for rename success/failure
- [ ] Support renaming in both individual and grouped views

### Interactive TUI Filtering
- [ ] Add project filtering UI to TUI filter panel
- [ ] Add tag filtering UI to TUI filter panel
- [ ] Add client filtering UI to TUI filter panel
- [ ] Add filter persistence across TUI sessions
- [ ] Add visual indicators for active filters in entry list

### CI/CD & Build Automation ✅ COMPLETED
- [x] Set up GitHub Actions workflow
- [x] Configure multi-platform builds (Linux/macOS/Windows × amd64/arm64)
- [x] Add automated testing on push/PR
- [x] Implement automated GitHub Releases with binaries
- [x] Add binary stripping and compression for smaller downloads
- [x] Set up clippy and rustfmt checks in CI
- [x] Add code coverage reporting

### Report Generation
- [ ] Implement daily summary report
- [ ] Add weekly summary report
- [ ] Create monthly summary report
- [ ] Add project-specific reports
- [ ] Calculate billable vs non-billable hours

## v1.1.3 - Fuzzy Matching & Incremental Sync (PLANNED)
### Fuzzy Matching
- [ ] Integrate strsim or fuzzy-matcher crate
- [ ] Implement similar description matching
- [ ] Add similarity threshold configuration
- [ ] Create preview for fuzzy matches before grouping

### Incremental Sync
- [ ] Implement incremental sync (only fetch new entries since last sync)
- [ ] Add cache invalidation logic
- [ ] Optimize sync performance for large date ranges

## v1.1.4+ - Remaining Phase 2 Features (PLANNED)
### Report Selection Interface
- [ ] Create report selection interface in TUI

### Additional Enhancements
- Any remaining Phase 2 tasks not yet completed
- Bug fixes and refinements based on user feedback
- Performance optimizations
- Documentation improvements

## Phase 3 - v1.2.0+ (FUTURE)
- TUI Testing
- Security Enhancements (keyring)
- PDF Export
- Help System
- Cross-Platform Testing
- Advanced Preferences
- Packaging (Homebrew, snap, .deb)
- Dockerization

## Notes
- Each version will be tested, formatted, linted, and committed before push
- Version bumps will follow semantic versioning
- Breaking changes will trigger major version bump
- User feedback will influence priority of remaining features

# Version Timeline

## v1.0.0 - Initial Release ✅ COMPLETED
- Core Toggl API integration
- Local SQLite caching
- Interactive TUI with grouping/rounding
- Configuration management
- Basic CLI commands

## v1.1.0 - Advanced Filtering & Enhanced UI ✅ COMPLETED
### Advanced Filtering
- [x] Add project-based filtering
- [x] Implement tag-based filtering
- [x] Add client-based filtering
- [x] Create filter combination logic (TimeEntryFilter builder)

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

## v1.1.1 - Project Assignment & Data Management (IN PROGRESS)
### Project Assignment Feature ✅ COMPLETED
- [x] Implement TUI project selector panel
- [x] Add project assignment for individual time entries (hotkey 'p')
- [x] Add project search functionality (hotkey '/')
- [x] Add navigation shortcuts (j/k, PageUp/PageDown, Home/End)
- [x] Batch assignment note: Shows message to toggle to individual view
- [x] API support: update_time_entry_project() method added
- [x] Async/sync integration: Using Arc<TogglClient> with Handle::block_on()

### Data Management & Multi-Account Support (PLANNED)
- [ ] Add CLI command for data deletion (`toggl-timeguru clean`)
  - [ ] `clean --all` - Delete database + config
  - [ ] `clean --data` - Delete only database
  - [ ] `clean --config` - Delete only config
  - [ ] `clean --confirm` - Skip confirmation prompt
- [ ] Implement multi-account support
  - [ ] Store user_id with database entries
  - [ ] Add account switching mechanism
  - [ ] Separate databases per account OR add account filtering
  - [ ] Visual indicator in TUI showing current account

### Data Export (PLANNED)
- [ ] Implement CSV export using csv crate
- [ ] Add customizable CSV format options
- [ ] Include metadata in exports (date range, filters)

## v1.1.2 - CI/CD & Build Automation (PLANNED)
### CI/CD & Build Automation
- [ ] Set up GitHub Actions workflow
- [ ] Configure multi-platform builds (Linux/macOS/Windows × amd64/arm64)
- [ ] Add automated testing on push/PR
- [ ] Implement automated GitHub Releases with binaries
- [ ] Add binary stripping and compression for smaller downloads
- [ ] Set up clippy and rustfmt checks in CI
- [ ] Add code coverage reporting

### Report Generation
- [ ] Implement daily summary report
- [ ] Add weekly summary report
- [ ] Create monthly summary report
- [ ] Add project-specific reports
- [ ] Calculate billable vs non-billable hours

## v1.1.2 - Fuzzy Matching & Caching (PLANNED)
### Fuzzy Matching
- [ ] Integrate strsim or fuzzy-matcher crate
- [ ] Implement similar description matching
- [ ] Add similarity threshold configuration
- [ ] Create preview for fuzzy matches before grouping

### Local Caching
- [ ] Implement sync mechanism for time entries
- [ ] Add last sync timestamp tracking
- [ ] Create offline mode support
- [ ] Implement incremental sync (only fetch new entries)
- [ ] Add cache invalidation logic

## v1.1.3-v1.1.5 - Remaining Features (PLANNED)
### Report Selection Interface (v1.1.3)
- [ ] Create report selection interface in TUI

### Additional Enhancements (v1.1.4-v1.1.5)
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

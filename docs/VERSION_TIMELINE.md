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

## v1.1.2 - Time Tracking & Entry Editing ✅ COMPLETED
### Data Persistence Improvements ✅ COMPLETED
- [x] Fix project assignment persistence (save to local DB immediately on assignment)
- [x] Update database record when project is assigned to time entry/group
- [x] Ensure TUI changes persist across sessions without manual sync

### TUI Error Display ✅ COMPLETED
- [x] Fix error message display corruption in TUI
- [x] Implement error popup modal for displaying API errors
- [x] Prevent long error messages from breaking TUI layout
- [x] Add keyboard controls (Enter/Esc) to close error popups

### CI/CD & Build Automation ✅ COMPLETED
- [x] Set up GitHub Actions workflow
- [x] Configure multi-platform builds (Linux/macOS/Windows × amd64/arm64)
- [x] Add automated testing on push/PR
- [x] Implement automated GitHub Releases with binaries
- [x] Add binary stripping and compression for smaller downloads
- [x] Set up clippy and rustfmt checks in CI
- [x] Add code coverage reporting

### TUI Time Entry Editing ✅ COMPLETED
- [x] Add time entry edit functionality in TUI (hotkey 'e')
- [x] Implement description input modal with text field
- [x] Update description in both Toggl API and local database
- [x] Add visual feedback for edit success/failure
- [x] Support editing in both individual and grouped views
- [x] Batch edit support for grouped entries

### Time Tracking CLI ✅ COMPLETED
- [x] Create new `track` command for starting/stopping time entries
- [x] Add `track start` subcommand with `-m/--message` flag for description
- [x] Add `track stop` subcommand to end current time entry
- [x] Integrate with Toggl API for real-time tracking

## v1.2.0 - API Optimization (CRITICAL - ✅ COMPLETED)
**Priority:** CRITICAL - Must be completed before adding more batch features
**Issue:** Current implementation makes N sequential API calls for batch operations, exhausting rate limits

### Bulk Update Endpoint Implementation ✅ COMPLETED
- [x] Add bulk update structs (BulkUpdateOperation, BulkUpdateResponse, BulkUpdateFailure)
- [x] Implement bulk_update_time_entries() method (max 100 entries per request)
- [x] Add bulk_assign_project() convenience method
- [x] Add bulk_update_descriptions() convenience method
- [x] Handle batches > 100 entries (split into chunks)
- [x] Update TUI to use bulk operations for project assignment
- [x] Update TUI to use bulk operations for description editing
- [x] Handle partial failures and provide detailed status messages

### Rate Limit Monitoring ✅ COMPLETED
- [x] Extract X-Toggl-Quota-Remaining and X-Toggl-Quota-Resets-In headers
- [x] Add RateLimitInfo struct to track quota state
- [x] Log warnings when quota is low (< 10 requests)
- [x] Update all API methods to extract headers

### Proactive Rate Limit Handling ✅ COMPLETED
- [x] Implement check_rate_limit_before_request() with throttling
- [x] Handle HTTP 402 Payment Required status code
- [x] Add wait and retry logic for quota exhaustion
- [x] Display rate limit info in TUI footer

### Testing & Documentation (PARTIALLY COMPLETED)
- [x] Unit tests for bulk operations and rate limiting
- [x] Integration tests with mocked rate limits
- [x] Update README and AGENTS.md with API optimization details
- [x] Create API_OPTIMIZATION_ANALYSIS.md document

**Expected Impact:** ✅ ACHIEVED
- 99% reduction in API calls for batch operations
- Free tier: Batch operations become usable
- Starter tier: 20x more operations per hour
- Premium tier: 12x more operations per hour

## v1.2.1 - Reports & Advanced Filtering (COMPLETED FOR RELEASE SCOPE)
### Release Scope Summary
- [x] Daily, weekly, and monthly report generation
- [x] Project-specific reports via `--project <id>`
- [x] Billable vs non-billable report totals and percentages
- [x] Report rounding options via `--round`, `--round-minutes`, and `--round-mode total|entry`
- [x] Project and tag multi-select filtering in the TUI
- [x] Persisted filter state across TUI sessions
- [x] Active filter indicators in the entry list and filter panel
- [x] Project selector usage sorting, usage stats, and `config --set-project-sort name|usage`
- [x] First-letter project selector jumps with repeated-key cycling
- [x] Main-list `c` hotkey to clear active filters

### Required Non-Deferred v1.2.1 Features
These are the planned v1.2.1 features that were not deferred and therefore should be included in the v1.2.1 release.

- [x] Report generation for daily, weekly, and monthly periods
- [x] Project-specific report filtering
- [x] Billable/non-billable report breakdowns
- [x] Report rounding controls and rounding mode selection
- [x] Interactive project filtering in the TUI
- [x] Interactive tag filtering in the TUI
- [x] TUI filter persistence and active-filter indicators
- [x] Project selector usage-based sorting and usage display
- [x] Project selector first-letter jump/cycle behavior
- [x] Clear-filters shortcut from the main list

### Still Deferred After v1.2.1
- [ ] Client filtering UI (deferred until client names/models are available)
- [ ] Tag counts in the filter panel (deferred to a later polish pass)
- [ ] Real API rate-limit integration tests (deferred to v1.2.2; requires staging account)

### Completed v1.2.1 Follow-up Work
- [x] Live rate-limit quota in the TUI footer
- [x] Mocked rate-limit integration tests and expanded API optimization docs

### Report Generation
- [x] Implement daily summary report
- [x] Add weekly summary report
- [x] Create monthly summary report
- [x] Add project-specific reports (via `--project <id>`)
- [x] Calculate billable vs non-billable hours
- [x] Add rounding options for report output

### Interactive TUI Filtering
- [x] Add project filtering UI to TUI filter panel (multi-select)
- [x] Add tag filtering UI to TUI filter panel (multi-select)
- [ ] Add client filtering UI to TUI filter panel (deferred — no Client model yet)
- [x] Add filter persistence across TUI sessions
- [x] Add visual indicators for active filters in entry list
- [x] Add shortcuts to open filters and clear active filters

### Project Selector Enhancements
- [x] Sort projects by usage in last month (most used first)
- [x] Show usage count/percentage per project in selector
- [x] Add configuration option to toggle sort method
- [x] Jump to project by first-letter key in selector
- [x] Cycle matching projects on repeated first-letter key presses
- [x] `c` hotkey in main entry list clears all active filters

## v1.2.2 - Smart Search & Fuzzy Matching (PLANNED)
### Instant Project Search
- [ ] Type-to-filter in project selector (no '/' needed)
- [ ] Real-time filtering as user types
- [ ] Clear search query with Esc
- [ ] Preserve existing '/' search for compatibility

### Fuzzy Matching for Grouping
- [ ] Integrate strsim or fuzzy-matcher crate
- [ ] Implement similar description matching
- [ ] Add similarity threshold configuration
- [ ] Create preview for fuzzy matches before grouping

### Report Selection Interface
- [ ] Create report selection interface in TUI
- [ ] Add hotkey to open report menu
- [ ] Allow selecting report type and date range

## v1.2.3 - Sync & Performance (PLANNED)
### Incremental Sync
- [ ] Implement incremental sync (only fetch new entries since last sync)
- [ ] Add cache invalidation logic
- [ ] Optimize sync performance for large date ranges

### Performance Optimizations
- [ ] Profile and optimize TUI rendering
- [ ] Optimize database queries with proper indexing
- [ ] Add connection pooling if needed

### Additional Enhancements
- Bug fixes and refinements based on user feedback
- Documentation improvements

## v1.3.0 - Testing & Quality (PLANNED)
### TUI Testing
- [ ] Research Ratatui TestBackend for TUI testing
- [ ] Add unit tests for keyboard event handlers
- [ ] Add integration tests for state transitions
- [ ] Implement UI snapshot tests for rendering output
- [ ] Test navigation edge cases

### Cross-Platform Testing
- [ ] Test on Linux (Ubuntu/Debian)
- [ ] Test on macOS
- [ ] Test on Windows
- [ ] Fix platform-specific issues
- [ ] Verify configuration paths on all platforms

## v1.3.1 - Security & Documentation (PLANNED)
### Security Enhancements
- [ ] Evaluate keyring crate for API token storage
- [ ] Implement OS-native credential storage
- [ ] Migrate from file-based encryption to keyring

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

## v1.4.0 - Export & Customization (PLANNED)
### PDF Export
- [ ] Research PDF generation options in Rust
- [ ] Evaluate external tools vs native library
- [ ] Implement basic PDF report generation
- [ ] Add customizable PDF templates

### Advanced Preferences
- [ ] Add default filter presets
- [ ] Implement custom report format templates
- [ ] Add theme/color customization
- [ ] Create keyboard shortcut customization

## v1.5.0 - Distribution (PLANNED)
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

## Notes
- Each version will be tested, formatted, linted, and committed before push
- Version bumps will follow semantic versioning
- Breaking changes will trigger major version bump
- User feedback will influence priority of remaining features

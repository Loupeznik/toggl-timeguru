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

### Improved Logging
- [x] Add detailed debug logging throughout codebase
- [x] Configure log levels via RUST_LOG environment variable
- [x] Log API requests/responses in debug mode
- [x] Add structured logging with tracing subscriber

### Testing & Error Handling
- [x] Expand unit test coverage (15 tests total)
- [x] Add tests for filtering (project, tag, client, combined, billable)
- [x] Improve error messages with context using anyhow
- [x] Add retry logic for API failures (exponential backoff, 3 retries)
- [x] Implement rate limiting handling (429 status code)

## v1.1.1 - CSV Export & CI/CD (PLANNED)
### Data Export
- [ ] Implement CSV export using csv crate
- [ ] Add customizable CSV format options
- [ ] Include metadata in exports (date range, filters)

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

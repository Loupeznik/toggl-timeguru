# Toggl TimeGuru - Development Progress

This document tracks the development progress across all phases of the Toggl TimeGuru project.

## Phase 1: MVP - Core Functionality

### Setup & Foundation
- [x] Initialize Rust project with Cargo
- [x] Configure dependencies in Cargo.toml
- [x] Create project documentation structure
- [ ] Set up project module structure (src/toggl/, src/ui/, src/db/, src/config/)
- [ ] Configure tracing for structured logging
- [ ] Set up rustfmt for code formatting
- [ ] Configure clippy for linting
- [ ] Set up pre-commit hooks or development scripts
- [ ] Add formatting and linting checks to development workflow

### Toggl API Integration
- [ ] Implement Toggl API client module
- [ ] Add authentication via API token
- [ ] Implement fetch time entries endpoint
- [ ] Add basic error handling for API responses
- [ ] Create data models for time entries using serde

### Configuration Management
- [ ] Design configuration structure
- [ ] Implement config loading/saving with confy
- [ ] Add API token storage with encryption using ring
- [ ] Store user preferences (default date range, report format)
- [ ] Create default configuration generator

### Data Storage
- [ ] Design SQLite database schema
- [ ] Implement database connection module
- [ ] Create time_entries table
- [ ] Add basic CRUD operations for time entries

### Terminal UI
- [ ] Set up ratatui with crossterm backend
- [ ] Create main application loop
- [ ] Implement basic time entries list view
- [ ] Add simple navigation (up/down, quit)
- [ ] Display time entry details (description, duration, project)

### Time Entry Processing
- [ ] Implement date range filtering
- [ ] Add time entry grouping by exact description match
- [ ] Calculate total duration for grouped entries
- [ ] Implement duration rounding functionality

### CLI Interface
- [ ] Design command-line arguments structure with clap
- [ ] Add date range parameters
- [ ] Add configuration file path option
- [ ] Implement help text and usage examples

### Testing & Polish
- [ ] Add unit tests for API client (using mockito)
- [ ] Test configuration management
- [ ] Test time entry grouping logic
- [ ] Basic error handling improvements
- [ ] Create initial README with setup instructions

## Phase 2: Enhanced Functionality

### Advanced Filtering
- [ ] Add project-based filtering
- [ ] Implement tag-based filtering
- [ ] Add client-based filtering
- [ ] Create filter combination logic

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

### Enhanced UI
- [ ] Improve navigation (page up/down, home/end)
- [ ] Add filter UI panel
- [ ] Create report selection interface
- [ ] Implement status bar with help hints
- [ ] Add loading indicators for API calls

### Local Caching
- [ ] Implement sync mechanism for time entries
- [ ] Add last sync timestamp tracking
- [ ] Create offline mode support
- [ ] Implement incremental sync (only fetch new entries)
- [ ] Add cache invalidation logic

### Fuzzy Matching
- [ ] Integrate strsim or fuzzy-matcher crate
- [ ] Implement similar description matching
- [ ] Add similarity threshold configuration
- [ ] Create preview for fuzzy matches before grouping

### Testing & Error Handling
- [ ] Expand unit test coverage
- [ ] Add integration tests with mocked API
- [ ] Improve error messages with context
- [ ] Add retry logic for API failures
- [ ] Implement rate limiting handling

### Logging
- [ ] Add detailed debug logging
- [ ] Configure log levels via environment variable
- [ ] Log API requests/responses in debug mode
- [ ] Add performance metrics logging

## Phase 3: Additional Features

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
- [ ] Set up GitHub Releases with binaries

### CI/CD
- [ ] Set up GitHub Actions workflow
- [ ] Add automated testing on push
- [ ] Configure cross-platform builds
- [ ] Implement automated releases
- [ ] Add code coverage reporting
- [ ] Set up linting (clippy)

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

# Toggl TimeGuru - Initial Functional Analysis

This document outlines the initial functional analysis for the Toggl TimeGuru project. It provides a high-level overview of the key features, user roles, and system requirements to guide the development process.

## Functional requirements

- Application is able to connect to Toggl Track API
- Application is able to fetch time entries from Toggl Track
- Application is able to display time entries in a user-friendly terminal interface
- Application is able to filter time entries by date range, project, and tags
- Application is able to generate reports based on time entries
    - Daily, weekly, and monthly summaries
    - Project-specific reports
- Application is able to summarize time entries by various criteria (e.g., total hours worked, billable vs non-billable)
- Application has functionality to group time entries with the same or similar descriptions and sum their duration with optional rounding
- Application is able to export reports to common formats (e.g., CSV, PDF)
- Application is able to handle user authentication securely
- Application is able to handle errors gracefully (e.g., API errors, network issues)
- Application is able to store user preferences (e.g., default date range, preferred report format) in an encrypted JSON configuration file. The user authentication defaults are also stored in this file.
    - The private key for encrypting this file is generated and stored on user's machine, not in the repository.
- Application is able to provide help and documentation within the terminal interface
- Application is able to run on multiple platforms (e.g., Windows, macOS, Linux)
- Application is able to be installed via package managers (e.g., pip, Homebrew)
- Application is able to be updated easily (e.g., via package managers)
- Application is able to store previous time entries locally in a sqlite database to allow offline access, faster loading times and reduce the number of API calls to Toggl Track.

## Tech stack

- Programming language: Rust
- Application type: Command-line interface (CLI)
- Integrations:
    - Toggl Track API
- Libraries/Frameworks:
    - HTTP client library for API requests (e.g., reqwest)
    - Terminal UI library for displaying data (e.g., tui-rs)
    - Serialization library for handling JSON data (e.g., serde)
    - Date and time library for handling date ranges (e.g., chrono)
    - Encryption library for securing user preferences (e.g., rust-crypto)
    - Configuration management library for handling user preferences (e.g., config)
    - Error handling library for managing errors (e.g., anyhow)
    - Testing framework for unit and integration tests (e.g., cargo test)
- Build and deployment:
    - Build tool: Cargo
    - Continuous integration: GitHub Actions
    - Package managers: snap, Homebrew, apt, cargo
    - Distribution platforms: GitHub Releases

## Development phases

1. **Phase 1: MVP - Core Functionality**
    - Connect to Toggl Track API via API key
    - Fetch and display time entries
    - Basic filtering by date range
    - Storing user preferences and toggl API key in an encrypted JSON configuration file
    - Basic error handling
    - Basic terminal UI
    - Time entry grouping by exact description match and summing their duration with rounding

2. **Phase 2: Enhanced Functionality**
    - Advanced filtering (project, tags)
    - Report generation (daily, weekly, monthly summaries)
    - Unit tests with mocked API responses
    - Improved terminal UI with better navigation and display options
    - Export reports to CSV format
    - More robust error handling and logging
    - Storing previous time entries locally in a sqlite database to allow offline access, faster loading
    - Time entry grouping by similar description match and summing their duration with rounding

3. **Phase 3: Additional Features**
    - Export reports to PDF format
    - Terminal app documentation and help system
    - Cross-platform support testing
    - Packaging for multiple platforms
    - Continuous integration setup
    - Dockerization
    - More advanced user preferences (e.g., default filters, report formats)

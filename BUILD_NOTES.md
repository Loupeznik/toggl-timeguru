# Build Notes

## Current Build Status

### Successfully Built ✅
- **macOS ARM64** (aarch64-apple-darwin) - Native build
  - Binary: `target/releases/toggl-timeguru-macos-arm64` (8.8 MB)
  - Built natively on macOS ARM64 system

- **macOS x86_64** (x86_64-apple-darwin) - Cross-compiled
  - Binary: `target/releases/toggl-timeguru-macos-x86_64` (8.9 MB)
  - Built using rustup-managed toolchain

- **Linux x86_64** (x86_64-unknown-linux-gnu) - Cross-compiled
  - Binary: `target/releases/toggl-timeguru-linux-x86_64` (10 MB)
  - Built using `cross` tool with Docker

- **Linux ARM64** (aarch64-unknown-linux-gnu) - Cross-compiled
  - Binary: `target/releases/toggl-timeguru-linux-arm64` (9.9 MB)
  - Built using `cross` tool with Docker

### Build Failures ❌
- **Windows x86_64** (x86_64-pc-windows-msvc) - Cannot build on macOS
  - Requires Visual Studio build tools (MSVC)
  - `cross` does not provide Docker image for Windows MSVC targets
  - Needs Windows runner in CI/CD

- **Windows ARM64** (aarch64-pc-windows-msvc) - Cannot build on macOS
  - Same limitations as Windows x86_64

## Build Method

After switching from Homebrew Rust to rustup:

```bash
# Install rustup targets
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc

# Install cross for Linux builds
cargo install cross --git https://github.com/cross-rs/cross

# Build for macOS targets (using cargo)
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Build for Linux targets (using cross with Docker)
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
```

## Recommended Solutions

### Option 1: GitHub Actions CI/CD (Recommended)
Use GitHub Actions to build for all platforms in isolated environments:
- Linux builds on `ubuntu-latest` runners
- macOS builds on `macos-latest` runners (both x86_64 and ARM64)
- Windows builds on `windows-latest` runners
- Automated release artifact creation

**Status**: Added to Phase 2 roadmap

### Option 2: Install rustup
Switch from Homebrew Rust to rustup-managed Rust:
```bash
# Remove Homebrew Rust
brew uninstall rust

# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add targets
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc
```

Then use `cross` for Linux/Windows builds and native cargo for macOS builds.

### Option 3: Docker-based Build Environment
Create a Dockerfile with rustup and all necessary targets for controlled cross-compilation.

## Current Release Binary

The only available release binary is:
- `target/releases/toggl-timeguru-macos-arm64` (Apple Silicon/M1/M2/M3)

For production releases, GitHub Actions (Option 1) is strongly recommended.

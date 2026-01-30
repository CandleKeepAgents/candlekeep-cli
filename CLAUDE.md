# CandleKeep CLI

Rust CLI for managing your CandleKeep document library.

## Quick Start

```bash
cargo build --release    # Build
cargo test               # Run tests
./target/release/ck      # Run locally
```

## Project Structure

```
src/
├── main.rs              # Entry point, CLI argument parsing
├── commands/            # Command implementations
│   ├── mod.rs
│   ├── auth.rs          # Login/logout flows
│   └── items.rs         # List, add, read, toc, remove
├── api.rs               # API client for CandleKeep server
└── output.rs            # Terminal output formatting
```

## Supported File Types

- **PDF** (.pdf) - `application/pdf`
- **Markdown** (.md, .markdown) - `text/markdown`

## Release Process

### Prerequisites

Install cargo-release for automated releases:
```bash
cargo install cargo-release
```

### Creating a Release

**Option A: Using cargo-release (Recommended)**
```bash
# Patch release (0.2.1 → 0.2.2)
cargo release patch --execute

# Minor release (0.2.1 → 0.3.0)
cargo release minor --execute

# Major release (0.2.1 → 1.0.0)
cargo release major --execute
```

This automatically:
1. Bumps version in Cargo.toml
2. Creates commit
3. Creates matching git tag
4. Pushes commit and tag

**Option B: Manual Release**
```bash
# 1. Update version in Cargo.toml
vim Cargo.toml  # Change version = "X.Y.Z"

# 2. Commit the version bump
git add Cargo.toml
git commit -m "chore: Bump version to X.Y.Z"
git push

# 3. Create and push tag (MUST match Cargo.toml version!)
git tag vX.Y.Z
git push origin vX.Y.Z
```

### What Happens After Tagging

The release workflow automatically:
1. **Validates** - Fails if tag doesn't match Cargo.toml version
2. **Builds** - Creates binaries for macOS (Intel + ARM), Linux, Windows
3. **Releases** - Creates GitHub Release with checksums
4. **Publishes** - Uploads to crates.io
5. **Updates Homebrew** - Updates the tap formula

### Version Mismatch Protection

The CI will **fail** if you create a tag that doesn't match `Cargo.toml`:

```
ERROR: Version mismatch! Cargo.toml has 0.2.1 but tag is v0.3.0
Please update Cargo.toml version to 0.3.0 before tagging
```

This prevents releasing binaries with incorrect version numbers.

## Testing Against Local Server

1. Start local candlekeep-cloud:
   ```bash
   cd ../candlekeep-cloud && ./dev
   ```

2. Configure CLI for local:
   ```bash
   # Edit ~/.candlekeep/config.toml
   [api]
   url = "http://localhost:3000"
   ```

3. Re-authenticate:
   ```bash
   ck auth logout && ck auth login
   ```

4. Test:
   ```bash
   ck items add test.pdf
   ck items list
   ```

## Configuration

Config file: `~/.candlekeep/config.toml`

```toml
[auth]
api_key = "ck_..."

[api]
url = "https://www.getcandlekeep.com"  # or http://localhost:3000
```

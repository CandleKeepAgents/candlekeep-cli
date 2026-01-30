# CandleKeep CLI (`ck`)

Command-line interface for managing your CandleKeep document library.

## Installation

### Using Homebrew (macOS)

```bash
brew tap CandleKeepAgents/candlekeep
brew install candlekeep-cli
```

### Using cargo-binstall (Recommended)

The fastest way to install - downloads a pre-built binary:

```bash
cargo binstall candlekeep-cli
```

### Using cargo install

Compiles from source (requires Rust toolchain):

```bash
cargo install candlekeep-cli
```

### Download Binary

Download the latest release for your platform from [GitHub Releases](https://github.com/CandleKeepAgents/candlekeep-cli/releases).

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | `ck-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `ck-x86_64-apple-darwin.tar.gz` |
| Linux (x86_64) | `ck-x86_64-unknown-linux-gnu.tar.gz` |
| Windows (x86_64) | `ck-x86_64-pc-windows-msvc.zip` |

After downloading, extract and move to your PATH:

```bash
# macOS/Linux
tar -xzf ck-*.tar.gz
chmod +x ck
sudo mv ck /usr/local/bin/

# Windows - extract zip and add to PATH
```

## Quick Start

```bash
# Login to your CandleKeep account
ck auth login

# List your library
ck items list

# Read a book
ck items read <id>:1-10

# Upload a PDF
ck items add ./book.pdf
```

## Usage

### Authentication

```bash
# Login via browser (opens auth flow)
ck auth login

# Show current user info
ck auth whoami

# Logout
ck auth logout
```

### Managing Items

```bash
# List all items
ck items list

# Read content from items (every ID must specify a page range)
ck items read <id>:all              # All pages
ck items read <id>:1-5              # Pages 1-5
ck items read <id1>:1-5,<id2>:all   # Multiple items with ranges

# Show table of contents
ck items toc <id>
ck items toc <id1>,<id2>

# Upload a PDF
ck items add ./document.pdf

# Remove items
ck items remove <id>
ck items remove <id1>,<id2> --yes   # Skip confirmation
```

### Output Format

Add `--json` flag to any command for JSON output:

```bash
ck items list --json
ck auth whoami --json
```

## Configuration

Configuration is stored at `~/.candlekeep/config.toml`:

```toml
[auth]
api_key = "ck_xxxxxxxxxx"

[api]
url = "https://www.getcandlekeep.com"
```

## Development

```bash
# Clone the repository
git clone https://github.com/CandleKeepAgents/candlekeep-cli.git
cd candlekeep-cli

# Build
cargo build

# Run
cargo run -- auth whoami

# Test
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) for details.

# Release Process

Berry uses GitHub Actions for CI/CD with automated releases.

## Package Structure

| Crate | Binary | Description |
|-------|--------|-------------|
| `berry` | (library) | Shared library with types and utilities |
| `berry-cli` | `berry` | Command-line interface |
| `berry-server` | `berry-server` | HTTP API server |
| `berry-mcp` | `berry-mcp` | MCP server |

## Creating a Release

### 1. Make Your Changes

Develop your feature or fix on the `main` branch (or a feature branch that will be merged to main).

### 2. Update Version

Update the version in `Cargo.toml`:

```toml
[workspace.package]
version = "0.2.0"  # Bump appropriately
```

The workspace version is inherited by all crates.

### 3. Commit and Tag

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin main --tags
```

### 4. Automated Release

When you push a tag matching `v*`, GitHub Actions will:

1. Run tests on all platforms
2. Build release binaries for:
   - Linux (x86_64, aarch64)
   - macOS (x86_64, aarch64/Apple Silicon)
   - Windows (x86_64)
3. Create a GitHub Release with:
   - Binary tarballs/zips for each platform
   - Auto-generated changelog

## Version Flow

```
┌─────────────────┐
│  Make changes   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Update version  │  ← Cargo.toml
│ in workspace    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Commit & tag    │  ← git tag v0.2.0
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Push to main  │  ← git push --tags
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ GitHub Actions  │  ← Build & release
│ creates release │
└─────────────────┘
```

## Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (x.0.0): Breaking changes
- **MINOR** (0.x.0): New features, backwards compatible
- **PATCH** (0.0.x): Bug fixes, backwards compatible

### Examples

#### Patch Release (bug fix)

```bash
# Current: 0.1.0 → 0.1.1
git tag -a v0.1.1 -m "Fix memory leak in server connection handling"
```

#### Minor Release (new feature)

```bash
# Current: 0.1.1 → 0.2.0
git tag -a v0.2.0 -m "Add support for custom memory tags"
```

#### Major Release (breaking change)

```bash
# Current: 0.2.0 → 1.0.0
git tag -a v1.0.0 -m "Stable release with new configuration format"
```

## CI Configuration

### CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and PR:
- Code formatting check (`cargo fmt`)
- Linting (`cargo clippy`)
- Tests (`cargo test`)
- Build verification

### Release Workflow (`.github/workflows/release.yml`)

Runs on version tags (`v*`):
- Multi-platform builds
- Binary packaging
- GitHub Release creation
- Asset upload

## Installation Methods

After a release, users can install Berry via:

### Installation Script (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/berry-rs/berry/main/scripts/install.sh | bash
```

This downloads binaries from GitHub Releases.

### From Source

```bash
cargo install --git https://github.com/berry-rs/berry berry-cli
cargo install --git https://github.com/berry-rs/berry berry-server
cargo install --git https://github.com/berry-rs/berry berry-mcp
```

## Manual Release

If you need to build a release manually:

```bash
# Build release binaries
cargo build --release

# Binaries are in target/release/
ls -la target/release/berry*
```

### Cross-compilation

For cross-platform builds, you can use `cross`:

```bash
# Install cross
cargo install cross

# Build for Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu

# Build for Windows
cross build --release --target x86_64-pc-windows-gnu
```

## Troubleshooting

### Build fails on CI

Check the GitHub Actions logs for specific errors:
- Ensure all dependencies are available
- Check for platform-specific issues

### Tag not triggering release

Ensure the tag follows the `v*` pattern:
```bash
# Correct
git tag v1.0.0

# Wrong - won't trigger release
git tag 1.0.0
git tag release-1.0.0
```

### Release assets missing

Check that the build completed successfully for all platforms. Failed builds won't produce artifacts.

### Version mismatch

Ensure `Cargo.toml` version matches the git tag:
```toml
[workspace.package]
version = "0.2.0"  # Should match tag v0.2.0
```

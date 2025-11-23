# Deployment and Release Process

This guide covers how jiq is packaged, distributed, and released to users.

## Table of Contents

1. [Overview](#overview)
2. [Distribution Channels](#distribution-channels)
3. [Release Process](#release-process)
4. [Version Management](#version-management)
5. [cargo-dist Setup](#cargo-dist-setup)
6. [Manual Release](#manual-release)
7. [Post-Release](#post-release)

## Overview

jiq uses **cargo-dist** for automated cross-platform builds and distribution. This provides:

- Automated binary builds for multiple platforms
- GitHub Releases with assets
- Shell installer script (`curl | sh`)
- Homebrew formula generation
- Checksums and signatures

### Distribution Strategy

```
┌─────────────────────────────────────────────────┐
│           jiq Distribution Channels             │
├─────────────────────────────────────────────────┤
│                                                 │
│  1. GitHub Releases (primary)                   │
│     └─ Download binaries directly               │
│                                                 │
│  2. Shell Installer                             │
│     └─ curl -sSf URL | sh                       │
│                                                 │
│  3. Homebrew (macOS)                            │
│     └─ brew install bellicose100xp/tap/jiq      │
│                                                 │
│  4. Cargo Install                               │
│     └─ cargo install jiq                        │
│                                                 │
│  5. Binary Downloads                            │
│     └─ Direct download from releases            │
│                                                 │
└─────────────────────────────────────────────────┘
```

## Distribution Channels

### 1. GitHub Releases

**URL:** https://github.com/bellicose100xp/jiq/releases

**Platforms built:**
- Linux x86_64 (GNU)
- Linux ARM64 (GNU)
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)
- Windows x86_64 (MSVC)

**Assets per release:**
- Binary archives (.tar.gz, .zip)
- Checksums (SHA256)
- Installer scripts
- Source code

### 2. Shell Installer

**macOS/Linux installation:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh
```

**What it does:**
1. Detects OS and architecture
2. Downloads appropriate binary
3. Verifies checksum
4. Installs to `~/.cargo/bin` or `/usr/local/bin`
5. Updates PATH if needed

### 3. Homebrew

**Installation:**

```bash
brew install bellicose100xp/tap/jiq
```

**Tap repository:** https://github.com/bellicose100xp/homebrew-tap

**Formula:** Automatically updated by cargo-dist

### 4. Cargo Install

**From crates.io:**

```bash
cargo install jiq
```

**Builds from source:**
- Requires Rust toolchain
- Compiles on user's machine
- Works on any platform Rust supports

### 5. Binary Downloads

Users can directly download platform-specific binaries from:

https://github.com/bellicose100xp/jiq/releases/latest

## Release Process

### Automated Release (Recommended)

cargo-dist automates the entire release process:

```bash
# 1. Update version
vim Cargo.toml  # Update version field

# 2. Update CHANGELOG.md
vim CHANGELOG.md  # Add release notes

# 3. Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "release: v2.4.0"

# 4. Create and push tag
git tag v2.4.0
git push origin main
git push origin v2.4.0

# 5. Wait for CI
# GitHub Actions automatically:
# - Builds binaries for all platforms
# - Creates GitHub Release
# - Uploads assets
# - Generates installer scripts
# - Updates Homebrew formula
```

### Release Workflow

```
Developer                    GitHub Actions               Users
    │                              │                        │
    │  1. Create tag (v2.4.0)      │                        │
    ├──────────────────────────────>                        │
    │                              │                        │
    │                              │ 2. Detect tag push     │
    │                              │                        │
    │                              │ 3. Build binaries      │
    │                              │    (Linux, macOS, Win) │
    │                              │                        │
    │                              │ 4. Run tests           │
    │                              │                        │
    │                              │ 5. Create Release      │
    │                              │                        │
    │                              │ 6. Upload assets       │
    │                              │                        │
    │                              │ 7. Generate installer  │
    │                              │                        │
    │                              │ 8. Update Homebrew     │
    │                              ├────────────────────────>
    │                              │                        │
    │  9. Verify release           │                        │
    │<──────────────────────────────                        │
    │                              │                        │
    │  10. Announce release        │                        │
    │──────────────────────────────────────────────────────>
```

## Version Management

### Semantic Versioning

jiq follows [SemVer](https://semver.org/): `MAJOR.MINOR.PATCH`

**Version increments:**

| Type | Version Change | Example |
|------|---------------|---------|
| Breaking change | MAJOR | 2.0.0 → 3.0.0 |
| New feature | MINOR | 2.3.0 → 2.4.0 |
| Bug fix | PATCH | 2.3.1 → 2.3.2 |

**Examples:**

- `2.3.0 → 2.4.0` - Added fuzzy matching (new feature)
- `2.3.0 → 2.3.1` - Fixed autocomplete bug (bug fix)
- `2.0.0 → 3.0.0` - Changed CLI interface (breaking change)

### Version Update Locations

When releasing, update version in:

1. **Cargo.toml** (required)
   ```toml
   [package]
   version = "2.4.0"
   ```

2. **CHANGELOG.md** (required)
   ```markdown
   ## [2.4.0] - 2025-11-23

   ### Added
   - New feature description
   ```

3. **Documentation** (if version mentioned)

### Pre-releases

For beta/RC versions:

```bash
# Tag as pre-release
git tag v2.4.0-beta.1
git push origin v2.4.0-beta.1

# GitHub will mark it as pre-release
```

## cargo-dist Setup

### Initial Setup (already done)

jiq is already configured with cargo-dist. This section documents the setup for reference.

**Installation:**
```bash
cargo install cargo-dist
```

**Initialize:**
```bash
cargo dist init
```

**Configuration in Cargo.toml:**
```toml
[profile.dist]
inherits = "release"
lto = "thin"

# The profile that 'dist' will build with
[package.metadata.dist]
# CI backends to support
ci = ["github"]
# The installers to generate for each app
installers = ["shell", "homebrew"]
# Target platforms to build apps for
targets = ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu", "x86_64-apple-darwin", "aarch64-apple-darwin", "x86_64-pc-windows-msvc"]
```

### Testing Locally

Test the release process locally:

```bash
# Build release artifacts
cargo dist build

# See what would be released
cargo dist plan

# Build and check everything
cargo dist build --artifacts all --output-format json
```

### CI Configuration

The release workflow is in `.github/workflows/release.yml`.

**Triggers:**
- Push of a tag matching `v*.*.*`
- Manual trigger (workflow_dispatch)

**Jobs:**
1. `plan` - Determine what to build
2. `build-local-artifacts` - Build binaries
3. `upload-local-artifacts` - Upload to GitHub
4. `publish-release` - Create GitHub Release

## Manual Release

If you need to release manually (without cargo-dist):

### 1. Build Binaries

```bash
# Build for current platform
cargo build --release

# Cross-compile (requires cross)
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-apple-darwin
cross build --release --target aarch64-apple-darwin
```

### 2. Create Release

```bash
# Create tag
git tag v2.4.0
git push origin v2.4.0

# Create release on GitHub
gh release create v2.4.0 \
  --title "jiq v2.4.0" \
  --notes-file CHANGELOG.md \
  target/release/jiq-*
```

### 3. Update Homebrew Formula

**File:** `homebrew-tap/Formula/jiq.rb`

```ruby
class Jiq < Formula
  desc "Interactive JSON query tool with VIM keybindings"
  homepage "https://github.com/bellicose100xp/jiq"
  url "https://github.com/bellicose100xp/jiq/archive/v2.4.0.tar.gz"
  sha256 "..." # Calculate with: shasum -a 256 archive.tar.gz
  license "MIT OR Apache-2.0"

  depends_on "jq"
  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/jiq", "--version"
  end
end
```

Commit and push to homebrew-tap repository.

### 4. Publish to crates.io

```bash
# Login (one-time)
cargo login

# Publish
cargo publish

# Verify
cargo search jiq
```

## Post-Release

### 1. Verify Release

**Check GitHub Release:**
- [ ] Release created successfully
- [ ] All platform binaries uploaded
- [ ] Checksums present
- [ ] Installer script works

**Test installations:**

```bash
# Test shell installer
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh

# Test Homebrew (wait for tap update)
brew update
brew install bellicose100xp/tap/jiq

# Test binary download
wget https://github.com/bellicose100xp/jiq/releases/download/v2.4.0/jiq-x86_64-unknown-linux-gnu.tar.gz
tar xzf jiq-x86_64-unknown-linux-gnu.tar.gz
./jiq --version
```

### 2. Announce Release

**Channels:**
- GitHub Discussions (announcement post)
- Reddit (r/rust, if significant release)
- Twitter/X (if applicable)
- Project website/blog (if applicable)

**Template:**

```markdown
# jiq v2.4.0 Released!

We're excited to announce jiq v2.4.0 with the following improvements:

## Highlights
- 🎯 Fuzzy matching in autocomplete
- ⚡ 50% faster query execution
- 🐛 Fixed 5 bugs

## Installation

```bash
# Shell installer (macOS/Linux)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh

# Homebrew
brew install bellicose100xp/tap/jiq

# Cargo
cargo install jiq
```

## Full Changelog
[View on GitHub](https://github.com/bellicose100xp/jiq/releases/tag/v2.4.0)

Thank you to all contributors!
```

### 3. Update Documentation

- [ ] Update README.md if needed
- [ ] Update project website (if applicable)
- [ ] Close related issues/PRs
- [ ] Update milestones

### 4. Monitor for Issues

Watch for:
- Installation problems
- Platform-specific bugs
- Regression reports

**Quick hotfix process:**
```bash
# If critical bug found
git checkout main
git checkout -b fix/critical-bug
# Fix bug
git commit -m "fix: critical bug description"
git push

# After PR merged, release patch version
git tag v2.4.1
git push origin v2.4.1
```

## Release Checklist

Complete checklist for each release:

### Pre-Release
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Dependencies updated (if needed)
- [ ] Security audit clean (`cargo audit`)

### Release
- [ ] Create and push git tag
- [ ] CI builds complete successfully
- [ ] GitHub Release created
- [ ] All platform binaries uploaded
- [ ] Homebrew formula updated
- [ ] crates.io published (if applicable)

### Post-Release
- [ ] Test shell installer
- [ ] Test Homebrew install
- [ ] Test binary downloads
- [ ] Announce release
- [ ] Update documentation site
- [ ] Close milestone
- [ ] Thank contributors

## Troubleshooting

### CI Build Fails

**Check logs:**
1. Go to GitHub Actions tab
2. Click on failed workflow
3. Review error logs

**Common issues:**

**Build fails on specific platform:**
```bash
# Test locally with cross
cargo install cross
cross build --target aarch64-unknown-linux-gnu
```

**Checksum mismatch:**
```bash
# Regenerate checksums
shasum -a 256 target/release/jiq > checksums.txt
```

**cargo-dist errors:**
```bash
# Update cargo-dist
cargo install cargo-dist --force

# Re-run plan
cargo dist plan
```

### Homebrew Formula Issues

**Formula not updating:**
```bash
# Manually trigger Homebrew tap update
cd homebrew-tap
git pull origin main

# Update formula
vim Formula/jiq.rb
# Update version and sha256

git commit -am "chore: update to v2.4.0"
git push
```

**Test formula:**
```bash
brew install --build-from-source ./Formula/jiq.rb
brew test jiq
brew audit --strict jiq
```

### crates.io Publishing Issues

**Publish fails:**
```bash
# Check for issues
cargo publish --dry-run

# Common: missing required fields in Cargo.toml
# Add: license, description, repository, keywords
```

## Resources

- [cargo-dist Documentation](https://opensource.axo.dev/cargo-dist/)
- [SemVer Specification](https://semver.org/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)

---

**Questions about releases?** Ask in discussions: https://github.com/bellicose100xp/jiq/discussions

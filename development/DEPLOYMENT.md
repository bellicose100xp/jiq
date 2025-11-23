# Deployment

## Distribution Channels

| Channel | Method |
|---------|--------|
| GitHub Releases | Download binaries directly |
| Shell Installer | `curl -sSf <url>/jiq-installer.sh \| sh` |
| Homebrew | `brew install bellicose100xp/tap/jiq` |
| Cargo | `cargo install jiq` |

**Platforms:** Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Windows (x86_64)

## Release Process

Uses **cargo-dist** for automated builds.

```bash
# 1. Update version
vim Cargo.toml  # Update version field

# 2. Update CHANGELOG
vim CHANGELOG.md

# 3. Commit
git commit -am "release: v2.4.0"

# 4. Tag and push
git tag v2.4.0
git push origin main
git push origin v2.4.0

# 5. CI automatically:
#    - Builds all platform binaries
#    - Creates GitHub Release
#    - Updates Homebrew formula
```

## Version Management

**SemVer:** `MAJOR.MINOR.PATCH`

| Change Type | Increment | Example |
|-------------|-----------|---------|
| Breaking change | MAJOR | 2.0.0 → 3.0.0 |
| New feature | MINOR | 2.3.0 → 2.4.0 |
| Bug fix | PATCH | 2.3.1 → 2.3.2 |

**Update locations:**
1. `Cargo.toml` - version field
2. `CHANGELOG.md` - release notes

## cargo-dist Setup

Already configured in `Cargo.toml`:
```toml
[package.metadata.dist]
ci = ["github"]
installers = ["shell", "homebrew"]
targets = ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
           "x86_64-apple-darwin", "aarch64-apple-darwin",
           "x86_64-pc-windows-msvc"]
```

**Test locally:**
```bash
cargo dist build
cargo dist plan
```

## Post-Release

**Verify:**
- [ ] GitHub Release created
- [ ] All binaries uploaded
- [ ] Installer script works
- [ ] Homebrew formula updated

**Test installations:**
```bash
# Shell installer
curl --proto '=https' --tlsv1.2 -LsSf <url>/jiq-installer.sh | sh

# Homebrew
brew install bellicose100xp/tap/jiq

# Verify
jiq --version
```

## Release Checklist

**Pre-release:**
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] CHANGELOG updated
- [ ] Version bumped

**Release:**
- [ ] Tag pushed
- [ ] CI builds complete
- [ ] GitHub Release created

**Post-release:**
- [ ] Test installations
- [ ] Close milestone

## Reference

- [cargo-dist Documentation](https://opensource.axo.dev/cargo-dist/)
- [SemVer](https://semver.org/)

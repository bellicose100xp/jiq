# Distribution Strategy for jiq

## Goal
Make jiq easily installable via:
- `brew install jiq` (macOS)
- `curl -sSL <url> | sh` (Linux/macOS install script)
- GitHub Releases (download binaries directly)
- `cargo binstall jiq` (for Rust users)

---

## Recommended: cargo-dist

**cargo-dist** automates everything:
- ✅ Builds binaries for multiple platforms
- ✅ Creates GitHub Releases automatically
- ✅ Generates install scripts (`curl | sh`)
- ✅ Creates Homebrew formula
- ✅ Sets up CI/CD (GitHub Actions)

### Setup Steps

1. **Install cargo-dist:**
   ```bash
   cargo install cargo-dist
   ```

2. **Initialize in project:**
   ```bash
   cargo dist init
   ```

   This creates:
   - `.github/workflows/release.yml` - Automated releases
   - Updates `Cargo.toml` with dist metadata
   - Configures target platforms

3. **Configure platforms:**
   ```bash
   cargo dist init --ci=github --installer=shell,homebrew
   ```

   Targets:
   - `x86_64-unknown-linux-gnu` (Linux x64)
   - `aarch64-unknown-linux-gnu` (Linux ARM)
   - `x86_64-apple-darwin` (macOS Intel)
   - `aarch64-apple-darwin` (macOS Apple Silicon)

4. **Test locally:**
   ```bash
   cargo dist build
   cargo dist plan
   ```

5. **Release workflow:**
   ```bash
   git commit -am "release: 2.0.0"
   git tag v2.0.0
   git push
   git push --tags
   ```

   GitHub Actions automatically:
   - Builds binaries for all platforms
   - Creates GitHub Release
   - Uploads binaries
   - Generates install script
   - Publishes Homebrew formula

### Result

Users can install via:

```bash
# Install script (Linux/macOS)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh

# Homebrew (macOS)
brew install bellicose100xp/tap/jiq

# Download binary directly
# Visit: https://github.com/bellicose100xp/jiq/releases
```

---

## Alternative: Manual Approach

If you don't want to use cargo-dist:

### 1. GitHub Actions for Building

Create `.github/workflows/release.yml`:
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          name: jiq-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/jiq
```

### 2. Homebrew Tap (Manual)

Create repo: `homebrew-tap`

File: `Formula/jiq.rb`
```ruby
class Jiq < Formula
  desc "Interactive JSON query tool with VIM keybindings"
  homepage "https://github.com/bellicose100xp/jiq"
  url "https://github.com/bellicose100xp/jiq/archive/v2.0.0.tar.gz"
  sha256 "<computed-sha>"
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

Users install:
```bash
brew tap bellicose100xp/tap
brew install jiq
```

### 3. Install Script

Create `install.sh`:
```bash
#!/bin/sh
set -e

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Download URL
URL="https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-${ARCH}-${OS}"

# Download and install
echo "Downloading jiq for ${OS}-${ARCH}..."
curl -sSL "$URL" -o /tmp/jiq
chmod +x /tmp/jiq
sudo mv /tmp/jiq /usr/local/bin/jiq

echo "✅ jiq installed to /usr/local/bin/jiq"
jiq --version
```

Host this on GitHub Pages or in repo, then:
```bash
curl -sSL https://raw.githubusercontent.com/bellicose100xp/jiq/main/install.sh | sh
```

---

## Recommendation: Use cargo-dist

**Why cargo-dist:**
- ✅ **Automated** - One command setup, automatic releases
- ✅ **Best practices** - Industry-standard approach
- ✅ **Cross-platform** - Handles all platforms automatically
- ✅ **Maintained** - Tool is actively maintained by axo.dev
- ✅ **Multiple installers** - Shell script + Homebrew in one setup
- ✅ **CI/CD included** - GitHub Actions workflow generated
- ✅ **Zero maintenance** - Just tag and push

**Steps:**
1. Run `cargo dist init`
2. Commit the generated files
3. Tag a release: `git tag v2.0.0 && git push --tags`
4. Done! Binaries built, releases created, installers ready

---

## Additional: cargo-binstall Support

Add to Cargo.toml:
```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/jiq-{ target }{ archive-suffix }"
bin-dir = "jiq-{ target }/{ bin }{ binary-ext }"
pkg-fmt = "tgz"
```

Then users can:
```bash
cargo binstall jiq
```

---

## Next Steps

Would you like me to:
1. **Set up cargo-dist** (recommended) - Quick automated solution
2. **Create manual CI/CD** - More control but more work
3. **Research more options** - Other distribution methods

What's your preference?

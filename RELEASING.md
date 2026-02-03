# Releasing MCP Wallet

This document describes how to create a new release of MCP Wallet.

## Prerequisites

1. You have push access to the repository
2. The `main` branch is in a releasable state (all tests passing)
3. You've updated the version numbers (see below)

## Version Numbering

We use semantic versioning: `MAJOR.MINOR.PATCH[-PRERELEASE]`

- **MAJOR**: Breaking changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible
- **PRERELEASE**: `alpha`, `beta`, `rc1`, etc.

Example versions:
- `0.1.0-beta` - First beta release
- `0.1.0` - First stable release
- `0.1.1` - Bug fix release
- `0.2.0` - New feature release

## Updating Version Numbers

Before creating a release, update the version in these files:

1. **Workspace Cargo.toml** (`/Cargo.toml`):
   ```toml
   [workspace.package]
   version = "0.1.0"
   ```

2. **Tauri config** (`/crates/tauri-app/tauri.conf.json`):
   ```json
   {
     "version": "0.1.0"
   }
   ```

3. **UI package.json** (`/crates/tauri-app/ui/package.json`):
   ```json
   {
     "version": "0.1.0"
   }
   ```

## Creating a Release

### Option 1: Using Git Tags (Recommended)

1. **Update versions** (see above)

2. **Commit version changes**:
   ```bash
   git add .
   git commit -m "chore: bump version to v0.1.0-beta"
   ```

3. **Create and push tag**:
   ```bash
   git tag v0.1.0-beta
   git push origin main
   git push origin v0.1.0-beta
   ```

4. **Wait for builds** - GitHub Actions will automatically:
   - Create a draft release
   - Build for macOS (Apple Silicon + Intel)
   - Build for Windows (x64)
   - Build for Linux (x64)
   - Upload all artifacts

5. **Review and publish** - Go to GitHub Releases:
   - Review the draft release
   - Edit release notes if needed
   - Click "Publish release"

### Option 2: Manual Workflow Dispatch

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the version tag (e.g., `v0.1.0-beta`)
4. Click "Run workflow"

## Build Artifacts

Each release produces these artifacts:

### Desktop Application

| Platform | Architecture | File |
|----------|-------------|------|
| macOS | Apple Silicon (M1/M2/M3) | `MCP-Wallet_X.X.X_aarch64.dmg` |
| macOS | Intel | `MCP-Wallet_X.X.X_x64.dmg` |
| Windows | x64 | `MCP-Wallet_X.X.X_x64-setup.exe` |
| Linux | x64 | `MCP-Wallet_X.X.X_amd64.deb` |
| Linux | x64 | `MCP-Wallet_X.X.X_amd64.AppImage` |

### CLI Server (Headless)

| Platform | Architecture | File |
|----------|-------------|------|
| macOS | Apple Silicon | `mcp-wallet-server-aarch64-apple-darwin` |
| macOS | Intel | `mcp-wallet-server-x86_64-apple-darwin` |
| Windows | x64 | `mcp-wallet-server-x86_64-pc-windows-msvc.exe` |
| Linux | x64 | `mcp-wallet-server-x86_64-unknown-linux-gnu` |

## Code Signing (Optional)

For production releases, you may want to code sign the binaries:

### macOS

1. Get an Apple Developer certificate
2. Set these secrets in GitHub:
   - `APPLE_CERTIFICATE`: Base64-encoded .p12 file
   - `APPLE_CERTIFICATE_PASSWORD`: Password for the .p12
   - `APPLE_ID`: Your Apple ID email
   - `APPLE_PASSWORD`: App-specific password
   - `APPLE_TEAM_ID`: Your team ID

### Windows

1. Get a code signing certificate (e.g., from DigiCert)
2. Set these secrets:
   - `WINDOWS_CERTIFICATE`: Base64-encoded .pfx file
   - `WINDOWS_CERTIFICATE_PASSWORD`: Password for the .pfx

### Tauri Updater Signing

For auto-update functionality:

1. Generate a key pair:
   ```bash
   cargo tauri signer generate
   ```

2. Set these secrets:
   - `TAURI_SIGNING_PRIVATE_KEY`: The private key
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: The password

## Testing a Release

Before creating a public release:

1. **Test locally**:
   ```bash
   cargo build --release
   cd crates/tauri-app && cargo tauri build
   ```

2. **Test the built app**:
   - Install on your machine
   - Create a wallet
   - Add an integration
   - Verify MCP server works

3. **Test on other platforms** if possible

## Hotfix Releases

For urgent bug fixes:

1. Create a branch from the release tag:
   ```bash
   git checkout -b hotfix/v0.1.1 v0.1.0
   ```

2. Make the fix and commit

3. Tag and release:
   ```bash
   git tag v0.1.1
   git push origin hotfix/v0.1.1
   git push origin v0.1.1
   ```

4. Merge back to main:
   ```bash
   git checkout main
   git merge hotfix/v0.1.1
   git push origin main
   ```

## Troubleshooting

### Build fails on Linux

Make sure all dependencies are installed. The workflow installs them, but if building locally:

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libgtk-3-dev \
  libayatana-appindicator3-dev
```

### macOS build not code signed

Without code signing, users will see "app is damaged" warnings. They can bypass with:

```bash
xattr -cr /Applications/MCP\ Wallet.app
```

Or right-click → Open → Open anyway.

### Windows SmartScreen warning

Without an EV certificate, Windows will show SmartScreen warnings. Users can click "More info" → "Run anyway".

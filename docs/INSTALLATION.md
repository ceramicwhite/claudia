# Claudia Installation Guide

This guide provides detailed instructions for installing Claudia on various platforms.

## Prerequisites

Before installing Claudia, ensure you have the following:

### Required Software

1. **Claude Code CLI**
   - Download and install from [Claude's official site](https://claude.ai/code)
   - Ensure `claude` is available in your PATH
   - Verify installation: `claude --version`

### System Requirements

- **Operating System**: 
  - Windows 10/11
  - macOS 11+ (Big Sur or later)
  - Linux (Ubuntu 20.04+, Debian 11+, Fedora 34+)
- **RAM**: Minimum 4GB (8GB recommended)
- **Storage**: At least 1GB free space
- **Display**: 1280x720 minimum resolution

## Installation Methods

### Method 1: Download Pre-built Binaries (Recommended)

> **Note**: Pre-built binaries will be available soon. Check the [releases page](https://github.com/getAsterisk/claudia/releases) for updates.

#### Windows
1. Download the `.msi` installer from the releases page
2. Double-click the installer and follow the wizard
3. Launch Claudia from the Start Menu

#### macOS
1. Download the `.dmg` file from the releases page
2. Open the DMG and drag Claudia to Applications
3. Launch from Applications folder
4. If you see a security warning, go to System Preferences → Security & Privacy and click "Open Anyway"

#### Linux
Choose your preferred format:

**AppImage** (Universal):
```bash
# Download the AppImage
wget https://github.com/getAsterisk/claudia/releases/download/vX.X.X/claudia_X.X.X_amd64.AppImage
chmod +x claudia_X.X.X_amd64.AppImage
./claudia_X.X.X_amd64.AppImage
```

**Debian/Ubuntu (.deb)**:
```bash
# Download and install the .deb package
wget https://github.com/getAsterisk/claudia/releases/download/vX.X.X/claudia_X.X.X_amd64.deb
sudo dpkg -i claudia_X.X.X_amd64.deb
# Fix any dependency issues
sudo apt-get install -f
```

### Method 2: Build from Source

For developers or users who want the latest features, you can build Claudia from source.

#### Build Prerequisites

1. **Rust** (1.70.0 or later)
   ```bash
   # Install via rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Bun** (latest version)
   ```bash
   # Install bun
   curl -fsSL https://bun.sh/install | bash
   ```

3. **Git**
   - Usually pre-installed on most systems
   - If not: `sudo apt install git` (Linux) or `brew install git` (macOS)

#### Platform-Specific Dependencies

**Linux (Ubuntu/Debian)**:
```bash
# Update package list
sudo apt update

# Install required dependencies
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libxdo-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev
```

**Linux (Fedora/RHEL)**:
```bash
# Install required dependencies
sudo dnf install -y \
  webkit2gtk4.1-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  patchelf \
  gcc \
  gcc-c++ \
  openssl-devel \
  curl \
  wget \
  file
```

**macOS**:
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Optional: Install additional tools via Homebrew
brew install pkg-config
```

**Windows**:
1. Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Select "Desktop development with C++" workload
2. Install [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)
   - Usually pre-installed on Windows 11
   - For Windows 10, download from the link above

#### Build Steps

1. **Clone the Repository**
   ```bash
   git clone https://github.com/getAsterisk/claudia.git
   cd claudia
   ```

2. **Install Frontend Dependencies**
   ```bash
   bun install
   ```

3. **Build for Development**
   ```bash
   # Start development server with hot reload
   bun run tauri dev
   ```

4. **Build for Production**
   ```bash
   # Create optimized production build
   bun run tauri build
   ```

   The built artifacts will be in `src-tauri/target/release/bundle/`:
   - **Linux**: `.deb`, `.AppImage`, and binary
   - **macOS**: `.app` bundle and `.dmg`
   - **Windows**: `.msi` and `.exe` installers

#### Build Options

**Debug Build** (faster compilation, larger binary):
```bash
bun run tauri build --debug
```

**Build without Bundling** (creates just the executable):
```bash
bun run tauri build --no-bundle
```

**Universal Binary for macOS** (Intel + Apple Silicon):
```bash
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
bun run tauri build --target universal-apple-darwin
```

## Post-Installation Setup

### First Launch

1. **Launch Claudia**
   - Windows: Start Menu → Claudia
   - macOS: Applications → Claudia
   - Linux: Application menu or terminal

2. **Initial Configuration**
   - Claudia will automatically detect your `~/.claude` directory
   - If not found, you'll be prompted to set up Claude Code CLI

3. **Verify Claude Code Integration**
   - Go to Settings → System
   - Check that "Claude Code CLI" shows as "Available"
   - If not, ensure `claude` is in your PATH

### Data Storage Locations

Claudia stores data in platform-specific locations:

- **Windows**: `%APPDATA%\com.asterisk.claudia`
- **macOS**: `~/Library/Application Support/com.asterisk.claudia`
- **Linux**: `~/.config/com.asterisk.claudia`

Claude Code data remains in: `~/.claude/`

## Troubleshooting

### Common Installation Issues

#### "Claude command not found"
1. Ensure Claude Code CLI is installed
2. Add Claude to your PATH:
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="$PATH:/path/to/claude/bin"
   ```
3. Restart your terminal

#### Linux: "GLIBC version not found"
- Your system's glibc is too old
- Update your distribution or build from source

#### macOS: "Cannot be opened because the developer cannot be verified"
1. Go to System Preferences → Security & Privacy
2. Click "Open Anyway" next to the Claudia message
3. Or right-click the app and select "Open"

#### Windows: "WebView2 not found"
- Download and install WebView2 Runtime from Microsoft
- Restart your computer after installation

### Build Issues

#### "cargo not found"
```bash
# Ensure Rust is in your PATH
source $HOME/.cargo/env
# Or add to your shell profile
echo 'source $HOME/.cargo/env' >> ~/.bashrc
```

#### Linux: "webkit2gtk not found"
```bash
# For Ubuntu 22.04+
sudo apt install libwebkit2gtk-4.1-dev
# For older versions
sudo apt install libwebkit2gtk-4.0-dev
```

#### "Out of memory" during build
```bash
# Limit parallel jobs
CARGO_BUILD_JOBS=2 bun run tauri build
```

## Updating Claudia

### For Pre-built Binaries
- Check the app for update notifications
- Download the latest version from releases
- Install over the existing version

### For Source Builds
```bash
cd claudia
git pull origin main
bun install
bun run tauri build
```

## Uninstallation

### Windows
- Use "Add or Remove Programs" in Settings
- Or run the uninstaller from the installation directory

### macOS
- Drag Claudia from Applications to Trash
- Remove data: `rm -rf ~/Library/Application Support/com.asterisk.claudia`

### Linux
**If installed via .deb**:
```bash
sudo apt remove claudia
```

**If using AppImage**:
- Simply delete the AppImage file
- Remove data: `rm -rf ~/.config/com.asterisk.claudia`

## Getting Help

If you encounter issues:

1. Check the [GitHub Issues](https://github.com/getAsterisk/claudia/issues)
2. Join our community discussions
3. Review the [FAQ](FAQ.md)

## Next Steps

After installation:
- Read the [User Guide](../README.md#usage) to get started
- Explore the [Features](../README.md#features)
- Check out [DEVELOPMENT.md](DEVELOPMENT.md) if you want to contribute
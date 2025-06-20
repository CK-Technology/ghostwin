# GhostWin Installer Information

## Installation Scripts

GhostWin provides two installer scripts for different use cases:

### Main Installer (`install.ps1`)
**Recommended for most users**

- **Purpose**: Production-ready installer with essential dependencies
- **Features**:
  - Visual Studio Build Tools installation with C++ support
  - Rust toolchain installation and optimization
  - Windows ADK and PE add-on installation (winget + manual fallback)
  - Pre-built binary download option (faster)
  - Source compilation with optimized Cargo configuration
  - Robust error handling and user guidance
  - Clean installation directory management

**Usage:**
```powershell
# Standard installation
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex

# Pre-built binaries (faster)
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -PreBuilt

# Custom installation path
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/install.ps1 | iex -InstallPath "C:\Tools\GhostWin"
```

### Legacy Installer (`installerLegacy.sh`)
**For advanced users or troubleshooting**

- **Purpose**: Comprehensive installer with extended dependency management
- **Features**: 
  - All features from main installer
  - Extended dependency validation
  - Additional troubleshooting options
  - More verbose error reporting
  - Legacy compatibility options

**Usage:**
```powershell
# Standard installation with full dependency checks
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/installerLegacy.sh | iex

# With custom options
iwr -useb https://raw.githubusercontent.com/CK-Technology/ghostwin/main/installerLegacy.sh | iex -PreBuilt -InstallPath "C:\Tools\GhostWin"
```

## Dependencies Handled

Both installers automatically manage:

1. **Visual Studio Build Tools 2022**
   - Required for compiling Rust applications on Windows
   - Includes C++ toolchain and Windows 11 SDK
   - Auto-installation with fallback options

2. **Rust Toolchain**
   - Latest stable Rust via rustup
   - Optimized Cargo configuration for Windows
   - Environment variable optimization

3. **Windows Assessment and Deployment Kit (ADK)**
   - Windows ADK (base toolkit)
   - Windows PE add-on (for boot environments)
   - Required for Windows deployment and ISO creation
   - Installed via winget with manual download fallback

## Installation Options

| Option | Description | Use Case |
|--------|-------------|----------|
| `-PreBuilt` | Download pre-built binaries | Faster installation, no compilation |
| `-SkipRust` | Skip Rust installation | Already have Rust installed |
| `-SkipBuild` | Download source but skip build | Development or inspection |
| `-SkipEnvConfig` | Skip Cargo environment optimization | Custom Rust setup |
| `-FixCargo` | Reset Cargo configuration | Fix Cargo network/index issues |
| `-InstallPath` | Custom installation directory | Non-standard installation location |

## System Requirements

- **OS**: Windows 10/11 (x64)
- **Admin**: Recommended for optimal dependency installation
- **Internet**: Required for downloading dependencies and source
- **Disk Space**: ~2GB for full development installation
- **RAM**: 4GB recommended for compilation

## Troubleshooting

If you encounter issues:

1. **Build Tools Issues**: Try `-PreBuilt` flag to skip compilation
2. **Network Issues**: Use `-FixCargo` to reset Cargo configuration  
3. **Permission Issues**: Run PowerShell as Administrator
4. **Dependency Issues**: Use `installerLegacy.sh` for extended validation

For detailed troubleshooting, see `TROUBLESHOOTING.md`.

## Post-Installation

After successful installation:

1. **Launch GUI**: `ghostwin.exe gui`
2. **Command Help**: `ghostwin.exe --help`
3. **Validate**: `ghostwin.exe validate`
4. **Add to PATH**: Optional during installation

## Support

- **Issues**: [GitHub Issues](https://github.com/CK-Technology/ghostwin/issues)
- **Docs**: See `README.md`, `TROUBLESHOOTING.md`, `DOCS.md`
- **Commands**: See `COMMANDS.md` for usage examples

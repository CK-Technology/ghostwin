# Setup

## Windows Install

One-liner install:

```powershell
irm https://win.cktech.sh | iex
```

The installer handles:
- Rust toolchain installation
- Visual Studio Build Tools
- Windows ADK (10.1.26100.2454 for Windows 11 25H2)
- Windows PE Add-on
- Building GhostWin from source

## Manual ADK Installation

If you need a different ADK version (for older Windows) or prefer manual installation, see the [ADK/WinPE Guide](../guides/adk-winpe-guide.md).

The automatic installer targets Windows 11 25H2. For Windows 11 24H2 or earlier, the same ADK works, but you may want a version-matched ADK for specific scenarios.

## Development Setup (Linux/Cross-compile)

```bash
# Check Rust code
cargo check

# Run tests
cargo test

# Verify Windows cross-compile
cargo check --target x86_64-pc-windows-gnu
```

## Requirements

### Windows (Runtime)
- Windows 10/11 with Administrator privileges
- Windows ADK + WinPE Add-on
- 20GB+ free disk space for ISO building

### Linux (Development)
- Rust toolchain
- mingw-w64 for Windows cross-compilation
- Useful for code validation, not for actual ISO building

## Notes

- ISO building requires Windows (DISM, oscdimg are Windows-only)
- Linux development is useful for code changes and testing
- The GUI runs on both Linux and Windows

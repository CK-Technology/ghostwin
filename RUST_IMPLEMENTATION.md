# GhostWin Rust Implementation

This document outlines the Rust implementation of GhostWin, a modern replacement for the AutoIt-based Windows Setup Helper.

## ✅ Completed Features

### 🔧 Core CLI Framework
- **Multi-command CLI** with `clap` for argument parsing
- **Logging system** with `tracing` for structured logging
- **Configuration management** with TOML/JSON support
- **Error handling** with `anyhow` for better error messages

### 🏗️ Build System (`ghostwin build`)
- **ISO extraction** using 7-Zip integration
- **WIM mounting/unmounting** via DISM commands
- **File copying** with recursive directory support
- **WinPE package injection** for enhanced functionality
- **Registry fixes** (DPI scaling, etc.)
- **ISO creation** using oscdimg from Windows ADK

### 🛠️ Tool Detection System
- **Folder-based scanning** for Tools, PEAutoRun, Logon directories
- **Multi-drive detection** for external tool sources
- **File type recognition** (executables, scripts, etc.)
- **Options file support** (.Options.txt configuration)
- **Category classification** (Tool, PEAutoRun, Logon)

### ✅ Validation & Diagnostics
- **Dependency checking** (DISM, 7-Zip, oscdimg)
- **Admin privilege verification**
- **Configuration validation**
- **Tool scanning and reporting**

## 🚧 In Progress

### 🖥️ GUI Framework (Planned: Slint)
- Modern native GUI for WinPE environment
- Tool browser and launcher interface
- Install mode selection (Normal vs Automated)
- System status and network information
- VNC server management

## 📁 Architecture

```
src/
├── main.rs           # Entry point and CLI routing
├── cli/              # Command implementations
│   ├── mod.rs        # CLI types and configuration
│   ├── build.rs      # ISO building workflow
│   ├── gui.rs        # GUI launcher (placeholder)
│   ├── validate.rs   # System validation
│   └── tools.rs      # Tool detection and listing
├── wim/              # WIM file management
│   └── mod.rs        # DISM integration and WIM operations
├── config/           # Configuration management
│   └── mod.rs        # TOML/JSON config loading
├── tools/            # Tool detection system
│   └── mod.rs        # Folder scanning and tool classification
└── utils/            # Utility functions
    └── mod.rs        # Admin checks, dependency validation
```

## 🔄 Migration from AutoIt

### AutoIt → Rust Equivalents

| AutoIt Feature | Rust Implementation | Status |
|---------------|-------------------|--------|
| `Main.au3` GUI | Slint-based GUI | 🚧 Planned |
| `Build.bat` | `ghostwin build` | ✅ Complete |
| Tool scanning | `ToolDetector` | ✅ Complete |
| WIM operations | `WimManager` | ✅ Complete |
| Config handling | `ConfigManager` | ✅ Complete |
| Password auth | Security module | 🚧 Planned |
| VNC integration | Network module | 🚧 Planned |

### Key Improvements

1. **Type Safety**: Rust's type system prevents many runtime errors
2. **Memory Safety**: No memory leaks or buffer overflows
3. **Performance**: Compiled binary with better resource usage
4. **Error Handling**: Structured error reporting with context
5. **Maintainability**: Modular architecture with clear separation
6. **Cross-platform**: Core logic works on Windows/Linux (GUI Windows-only)

## 🚀 Usage

### Building an ISO
```bash
# Basic build
ghostwin build \
  --source-iso "C:\Windows11.iso" \
  --output-dir "C:\temp\build" \
  --output-iso "C:\GhostWin.iso"

# With extra files
ghostwin build \
  --source-iso "C:\Windows11.iso" \
  --output-dir "C:\temp\build" \
  --output-iso "C:\GhostWin.iso" \
  --extra-files "C:\MyTools"

# With custom config
ghostwin build \
  --source-iso "C:\Windows11.iso" \
  --output-dir "C:\temp\build" \
  --output-iso "C:\GhostWin.iso" \
  --config "my-config.toml"
```

### Tool Management
```bash
# Scan for tools
ghostwin tools

# Validate configuration
ghostwin validate

# Launch GUI (when implemented)
ghostwin gui
```

### Configuration
```toml
# ghostwin.toml
[iso]
wim_index = "Microsoft Windows Setup (amd64)"

[winpe]
packages = ["WinPE-WMI", "WinPE-NetFX", "WinPE-PowerShell"]
disable_dpi_scaling = true
set_resolution = "1024x768"

[tools]
folders = ["Tools", "PEAutoRun", "Logon"]
auto_detect = true

[security]
vnc_enabled = true
vnc_port = 5950
vnc_password = "vncwatch"
```

## 🔮 Roadmap

### Phase 1: Core Functionality (✅ Complete)
- CLI framework and basic commands
- WIM management and ISO building
- Tool detection system
- Configuration management

### Phase 2: GUI Development (🚧 Current)
- Slint GUI framework integration
- WinPE-compatible interface
- Tool launcher and script runner
- System status display

### Phase 3: Advanced Features
- Network integration (VNC, NetBird)
- Automated installation workflows
- Driver injection system
- Remote management capabilities

### Phase 4: Polish & Distribution
- Windows installer/package
- Documentation and tutorials
- Performance optimization
- Comprehensive testing

## 🏆 Benefits Over AutoIt Version

1. **Reliability**: Compiled binary with robust error handling
2. **Performance**: Faster execution and lower resource usage
3. **Maintainability**: Modern language with excellent tooling
4. **Security**: Type-safe operations and memory safety
5. **Extensibility**: Modular design for easy feature addition
6. **Debugging**: Rich logging and error reporting
7. **Distribution**: Single executable with clear dependencies

## 🎯 Conclusion

The Rust implementation provides a solid foundation for GhostWin with:
- **Complete CLI replacement** for Build.bat workflows
- **Robust tool detection** matching AutoIt functionality
- **Modern architecture** for future GUI development
- **Better error handling** and user experience
- **Foundation for advanced features** like remote management

The core functionality is ready for production use, with GUI development being the next major milestone.
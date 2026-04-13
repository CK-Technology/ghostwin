# Setup

## Windows Install

Recommended one-liner:

```powershell
iwr -useb https://win.cktech.sh | iex -PreBuilt
```

Source-build path:

```powershell
iwr -useb https://win.cktech.sh | iex
```

Quiet pre-built install:

```powershell
iwr -useb https://win.cktech.sh | iex -PreBuilt -NonInteractive
```

## Manual Development Setup

```bash
cargo check
cargo test
cargo check --target x86_64-pc-windows-gnu
```

## Current Reality

- Linux is useful for development and validation
- the real Windows media build path still requires Windows-host testing
- ADK, WinPE, DISM, and `oscdimg` are Windows-side requirements

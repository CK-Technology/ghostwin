# Windows Build And Smoke Test Guide

## Purpose

This guide describes the current Windows-side expectations for building and validating GhostWin media.

## Required Host Capabilities

- Windows host with administrator access
- Windows ADK installed
- Windows PE add-on installed
- `dism`
- `7z`
- `oscdimg`
- enough free disk space for ISO extraction and WIM mounting

## Expected Inputs

- a source Windows ISO
- writable output directory
- optional helper content under `concept/windows-setup-helper-master/Helper`
- optional Windows overlay content under `concept/windows-setup-helper-master/Windows`
- optional driver folders and extra files

## Basic Smoke Test Checklist

### 1. Validate host tooling

Run:

```bash
cargo run -- validate
```

Expected:

- config loads successfully
- required dependencies are visible
- ADK and WinPE paths are detected if installed in expected locations

### 2. Preview destructive host actions first

Run:

```bash
cargo run -- logon --dry-run
cargo run -- system-setup --dry-run
```

Expected:

- commands do not modify the host
- intended actions are logged clearly

### 3. Build a customized ISO

Run:

```bash
cargo run -- build --source-iso <path-to-iso> --output-dir <workdir> --output-iso <output-iso> --verify
```

Expected:

- source ISO validation passes
- extracted media layout validation passes
- WIM mounts successfully
- helper content copies into the expected image destinations
- optional DPI registry fix completes without host-registry corruption
- WIM unmounts and commits successfully
- ISO creation completes
- ISO verification passes

### 4. Verify offline registry behavior manually

After build:

1. Mount the modified `boot.wim` again.
2. Inspect offline SOFTWARE and SYSTEM hives.
3. Confirm these values exist in the offline image:

- `HKLM\WIM_SOFTWARE\Microsoft\Windows\CurrentVersion\SideBySide\PreferExternalManifest`
- `HKLM\WIM_SYSTEM\ControlSetXXX\Control\GraphicsDrivers\Configuration\DisableScalingOptimizations`

### 5. Verify output media layout

Confirm the media tree contains:

- `bootmgr`
- `sources/boot.wim`
- `boot/bcd`
- `boot/etfsboot.com`
- `efi/microsoft/boot/efisys.bin`

### 6. Boot-test the output

Boot the generated ISO in a VM first.

Minimum checks:

- WinPE boots successfully
- helper folders are present in expected destinations
- PE autorun content is present
- logon content is present for post-install flow
- expected drivers are available
- setup launch path works

## Known Gaps

- no automated Windows-host smoke test harness yet
- GUI flow still does not expose full backend step-by-step progress
- some host operations still rely on fixed paths under `C:\temp`
- Linux builds still include simulated success paths and are not proof of Windows correctness

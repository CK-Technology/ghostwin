# GhostWin Architecture

## Scope

This document describes the current project shape and the intended execution boundaries of the major modules.

## Primary Workflow

GhostWin is currently centered around building customized Windows media from an existing Windows ISO.

High-level flow:

1. Validate CLI input and host prerequisites.
2. Extract the source ISO into a working media directory.
3. Validate extracted media layout.
4. Mount `sources/boot.wim` at the configured image index.
5. Copy helper content, tools, drivers, and optional extra files into the mounted image.
6. Apply package injection and optional registry fixes.
7. Unmount and commit the WIM.
8. Validate ISO creation layout.
9. Build the final ISO.
10. Optionally verify the resulting ISO.

## Module Responsibilities

### `src/cli/`

- `build.rs`: top-level media build flow and validation
- `gui.rs`: Slint UI wiring and background action triggers
- `validate.rs`: host/config/dependency checks
- `tools.rs`: tool listing command
- `logon.rs`: post-install logon script execution and guarded host changes
- `system_setup.rs`: pre-logon setup script execution and guarded host changes

### `src/wim/`

- mount/unmount lifecycle for WIM images
- package injection
- file copy into mounted image
- offline registry edits for image customization

### `src/tools/`

- folder discovery for `Tools`, `PEAutoRun`, and `Logon`
- file classification into executable/script types
- helper destination mapping inside the mounted image
- `.Options.txt` parsing

### `src/drivers/`

- driver directory scanning
- INF/CAB/SYS detection
- storage-driver prioritization
- driver injection and copy-to-image behavior

### `src/executor/`

- tool and script execution
- PE autorun orchestration
- cross-platform simulation behavior for non-Windows hosts

### `src/utils/`

- common host validation helpers
- disk space checks
- ISO input validation
- recovery logic for failed builds

### `src/vnc/`

- VNC startup/shutdown and password configuration
- runtime connection reporting for the UI

## Current Repo Layout Assumptions

These assumptions still exist in code and should be treated as current contract until replaced:

- helper content may come from `concept/windows-setup-helper-master/Helper`
- Windows overlay content may come from `concept/windows-setup-helper-master/Windows`
- tool folders are expected to be named `Tools`, `PEAutoRun`, and `Logon`
- drivers may come from folders such as `Drivers`, `Tools/Drivers`, or `PEAutoRun/Drivers`

## Mounted Image Destinations

- `Tools` content goes to `Helper/Tools`
- `PEAutoRun` content goes to `Helper/PEAutoRun`
- `Logon` content goes to `Helper/Logon`
- copied drivers go to `Windows/System32/Drivers`

## Platform Reality

- The intended deployment path is Windows.
- Linux development builds are useful, but many execution paths are still simulated there.
- A passing Linux build does not prove that DISM, registry, ADK, or ISO creation behavior is correct.

## Current Risks

- no automated Windows-host smoke execution yet
- host-image registry behavior still needs real validation
- `RecoveryManager` is present but not fully integrated throughout all failure paths
- GUI progress reporting is improved, but still not a full backend state model

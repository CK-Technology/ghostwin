# Changelog

## 0.3.5

Release date: 2026-04-14

### GUI Overhaul

Major multi-phase overhaul of GUI correctness, state management, and workflow safety. ~2000 lines changed across `gui.rs` and `ghostwin.slint`.

**View & Navigation**
- fixed "menu" vs "home" view mode mismatch between Rust and Slint
- build flow now stays in build view instead of routing to wrong view
- install/build state resets when leaving views via sidebar or back button
- navigation preserves build state when a build is actively running
- go-back handlers properly reset progress state

**State Management**
- separated `build_progress` from `install_progress` to prevent state bleed
- removed premature completion - progress updates never set `completed = true`
- progress bar capped at 95% during updates; only `finalize_build_progress` sets 100%
- build generation counter (`AtomicU64`) prevents stale finalization race conditions

**Build Workflow**
- concurrent build guard (`AtomicBool`) prevents multiple simultaneous builds
- `build_running` UI property disables button and shows "Building..." during active builds
- build finalization checks generation ID before modifying state
- stale build completions logged and ignored

**Install Workflow**
- normal install now launches interactive `setup.exe` (no automation flags)
- automated upgrade uses `setup.exe /auto upgrade` for upgrade-style automation (not unattended clean install)
- `setup.exe` resolver now media-aware: prefers drives with `install.wim` or `boot.wim` present
- resolver searches D:-Z: before C: (media drives before system drive)
- C: only selected if it has media validation markers (`install.wim` or `boot.wim`)
- removed cwd-dependent fallbacks (`.\setup.exe`, `sources\setup.exe`) for deterministic behavior
- clear error if setup.exe not found: "Mount the Windows installation media first"
- setup.exe validation now runs BEFORE PE scripts, preventing side effects when media is missing
- concurrent install guard (`AtomicBool`) prevents multiple simultaneous install launches
- install generation counter (`AtomicU64`) prevents stale finalization race conditions
- `install_running` UI property disables both install buttons and shows "Launching..." during launch
- install state preserved during navigation when `install_running` is true (no invisible active installs)
- back button and sidebar navigation respect active install state
- install handoff uses `progress: 0.0` with "Handed off to Windows Setup" (no fake 100%)
- non-Windows simulation shows "Simulated - requires Windows host" with no progress bar
- PE script failures report which script failed with first line of stderr

**Tools & Script Execution**
- tool/script execution moved to worker threads (non-blocking UI)
- execution helper function consolidates callback logic
- failures with empty stderr now show exit code instead of false success
- non-zero exit codes always reported as errors
- action buttons consistently show "Run" instead of misleading "Open"
- tools view shows empty state when no tools detected

**VNC Integration**
- VNC errors show truncated detail (20 chars) instead of just "Error"
- `is-error` property detects error state (not enabled && not "Disconnected")
- status text truncates gracefully with `overflow: elide`

**Notifications**
- fixed notification animation by removing conditional wrapper that removed component from tree
- auto-dismiss timer working correctly
- toast notifications use proper error/success/info styling

**UI Copy & Wording**
- "Automated Install" renamed to "Automated Upgrade" to accurately describe `/auto upgrade` behavior
- install cards now say "Launch Windows Setup" / "Run Automated Upgrade"
- descriptions clarify that upgrade flow uses Windows Setup upgrade mode
- HomeView quick-action cards updated to match
- narrow layout cards updated for consistency

**Code Quality**
- UTF-8 safe `truncate_error()` using `chars()` iterator instead of byte slicing
- removed unused `SearchInput` and `ProgressSteps` components
- removed unused `warn` import (restored with `#[cfg(target_os = "windows")]`)
- fixed unsafe `set_var` for Rust 2024 edition
- added 8 GUI tests: UTF-8 truncation, build progress mapping, resolver search-order verification

### Added

- `docs/guides/adk-winpe-guide.md` - manual ADK/WinPE installation with version matrix and download links

### Changed

- README removed outdated "ocean blue theme" references
- README simplified install examples to working one-liner
- `docs/getting-started/setup.md` removed broken flag examples, added link to ADK guide

### Verification

- `cargo check`
- `cargo test` (57 total tests, 9 GUI tests pass)
- `cargo check --target x86_64-pc-windows-gnu`

## 0.3.4

Release date: 2026-04-13

### Added

- explicit deployment phase configuration for PE system setup, PE driver loader, and post-install logon paths
- GUI media-build workflow with real build inputs and backend progress updates
- shared build progress state and callback plumbing
- driver risk summaries before WIM injection
- Windows installer cleanup in root `install.ps1`
- root `CONTRIBUTING.md`
- root `SECURITY.md`
- architecture and Windows build docs
- root `CHANGELOG.md`
- `dev/` directory for local verification helpers instead of root-level test script clutter
- real command-dispatch tests for guarded and dry-run host action paths
- build/media layout validation for extracted media and ISO creation prerequisites
- WIM registry helper tests and control-set parsing coverage

### Changed

- `install.ps1` is now the single supported root installer entrypoint
- README Windows install examples now use `https://win.cktech.sh`
- build flow now attempts discard unmount when a mounted WIM step fails
- helper and overlay source paths are config-driven
- `logon` and `system-setup` now resolve explicit phase script lists instead of broad path heuristics
- GUI copy now distinguishes build-media workflow from installation workflow more clearly
- `ghostwin.toml` defaults now align with current validation rules
- project docs were reorganized into lowercase topical paths under `docs/`
- local test/debug PowerShell scripts were moved under `dev/powershell/`
- installer packaging now bundles the `docs/` tree instead of hardcoded removed files
- command dispatch now uses idempotent tracing initialization for repeatable tests

### Fixed

- Windows target compilation issues across executor, drivers, utils, and WIM logic
- insecure default VNC behavior and plaintext password logging
- invalid default WIM index handling
- tool destination mapping for `Tools`, `PEAutoRun`, and `Logon`
- stale config examples and README installer examples
- stale `ghostwin.toml` defaults that no longer matched validation rules
- duplicate CLI short flag conflict on `build` output arguments
- Slint warnings in the install/build UI panel
- overly broad host-action script discovery for `logon` and `system-setup`
- legacy installer and docs clutter in project root

### Verification

- `cargo check`
- `cargo test`
- `cargo check --target x86_64-pc-windows-gnu`
- PowerShell syntax validation for `install.ps1`

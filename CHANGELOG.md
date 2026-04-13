# Changelog

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

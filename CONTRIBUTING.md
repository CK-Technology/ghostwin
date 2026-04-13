# Contributing

## Scope

GhostWin is a Windows deployment media builder and phased automation toolkit. Contributions should improve build reliability, Windows-host correctness, deployment workflow clarity, and test coverage.

## Priorities

- fix real build/runtime issues before adding new surface area
- keep Windows deployment behavior explicit, not heuristic-heavy
- prefer small, test-backed changes
- keep `install.ps1` as the single supported root installer entrypoint

## Development Workflow

1. Run `cargo check`
2. Run `cargo test`
3. Run `cargo check --target x86_64-pc-windows-gnu`
4. Update docs when user-visible workflows or configuration change

## Project Structure

- `src/` Rust application code
- `ui/` Slint UI
- `pe_autorun/` WinPE boot-time assets and scripts
- `scripts/` post-build or post-install scripts
- `docs/` project documentation
- `install.ps1` Windows installer bootstrap

## Coding Expectations

- keep changes minimal and direct
- add tests for new logic when practical
- avoid adding new installer entrypoints unless there is a strong reason
- prefer explicit phase configuration over path-name guessing

## Pull Requests

Include:

- what changed
- why it changed
- what commands were used to verify it
- any Windows-host follow-up still required

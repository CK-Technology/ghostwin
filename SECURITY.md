# Security Policy

## Scope

GhostWin handles Windows deployment media, offline registry modification, tool injection, and optional remote-access configuration. Security-sensitive issues should be reported responsibly.

## Reporting

If you find a security issue, report it privately to the project maintainers instead of opening a public issue immediately.

Include:

- affected version or commit
- impact summary
- reproduction details
- whether the issue affects build-time media creation, WinPE runtime, or installed Windows systems

## High-Risk Areas

- offline registry editing
- remote access and VNC configuration
- driver injection and unsigned/risky driver handling
- installer bootstrap and dependency download behavior
- host-machine actions run by `logon` and `system-setup`

## Current Security Posture

- VNC is disabled by default
- no default VNC password is shipped
- Windows-host actions require explicit `--dry-run` or `--force`
- build validation includes stronger config checks than earlier versions

## Expected Follow-Up

Security fixes should include:

- code change
- test coverage when practical
- documentation updates if user behavior changes

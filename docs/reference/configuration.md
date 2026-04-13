# Configuration Reference

## Main Sections

- `[iso]`
- `[winpe]`
- `[tools]`
- `[phases]`
- `[security]`

## Important Current Fields

### `[iso]`

- `wim_index`
- `mount_path`
- `adk_path`
- `helper_source`
- `windows_overlay_source`

### `[phases]`

- `pe_system_setup_paths`
- `pe_driver_loader_paths`
- `post_install_logon_paths`

These phase paths are now preferred over folder-name heuristics for execution intent.

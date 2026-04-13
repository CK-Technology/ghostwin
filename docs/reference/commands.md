# Command Reference

## Core Commands

```bash
ghostwin gui
ghostwin build --source-iso <ISO> --output-dir <DIR> --output-iso <ISO>
ghostwin validate
ghostwin tools
ghostwin logon --dry-run
ghostwin system-setup --dry-run
```

## Notes

- `build` is the real media customization path
- `logon` targets explicit `post_install_logon_paths`
- `system-setup` targets explicit `pe_system_setup_paths`
- `validate` is only partial on non-Windows hosts

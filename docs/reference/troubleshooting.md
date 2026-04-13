# Troubleshooting

## Common Issues

### Build fails on Linux

That is expected for the real media-customization path. Linux can validate code and extract ISOs, but ADK, DISM, WIM mount, and `oscdimg` require Windows.

### `ghostwin validate` warns on Linux

The project intentionally reports that non-Windows validation is partial.

### Windows build fails while WIM is mounted

The build now attempts discard unmount on failure, but you may still need:

```powershell
dism /Cleanup-Wim
dism /Cleanup-Mountpoints
```

### ADK or WinPE not found

Install:

- Windows ADK
- Windows PE add-on

Then re-run:

```powershell
ghostwin validate
```

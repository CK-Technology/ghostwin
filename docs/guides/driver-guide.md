# GhostWin Driver Quick Reference Guide

Complete guide for adding storage drivers to ensure Dell Optiplex and modern NVMe drives work during Windows installation.

## 🎯 Quick Start - Essential Drivers

### 1. Intel RapidStorage Technology (RST) - **CRITICAL for Dell Optiplex**

**Why needed:** Dell Optiplex systems with Intel VMD (Virtual RAID on CPU) won't see NVMe drives without this.

**Download:**
```
https://www.intel.com/content/www/us/en/download/19512/
```

**Alternative direct link:**
```
https://downloadmirror.intel.com/782868/SetupRST.exe
```

**Installation:**
1. Download `SetupRST.exe` (or similar package)
2. Extract using 7-Zip or run and cancel to extract to temp
3. Find the "F6" or "Driver" folder
4. Copy entire folder to: `pe_autorun/drivers/Intel_RST/`

**Key files to verify:**
- ✅ `iaStorAC.inf` - AHCI Controller
- ✅ `iaStorAVC.inf` - VMD Controller (CRITICAL!)
- ✅ `iaStorAV.inf` or `iaStorAC.sys` - Driver binaries

### 2. Samsung NVMe Drivers

**Why needed:** Some Samsung NVMe drives (especially 970 EVO Plus, 980 PRO) work better with vendor drivers.

**Download:**
```
https://semiconductor.samsung.com/consumer-storage/support/tools/
```

Look for "NVMe Driver" section.

**Installation:**
1. Download Samsung NVMe Driver package
2. Extract the ZIP
3. Copy folder to: `pe_autorun/drivers/Samsung_NVMe/`

**Key files:**
- ✅ `SamsungNVMeController.inf`
- ✅ `nvmexpressSamsung.inf`

### 3. Micron NVMe Drivers

**Why needed:** Micron 2200/2300/3400 NVMe SSDs may need specific drivers.

**Download:**
```
https://www.micron.com/products/storage/ssd/client-ssd
```

Search for your specific model and download Windows drivers.

**Installation:**
1. Download driver package for your Micron model
2. Extract files
3. Copy to: `pe_autorun/drivers/Micron_NVMe/`

---

## 🔧 Dell Optiplex Specific Setup

### Identifying If You Need Intel RST

**Run this on your Dell system:**
```powershell
Get-WmiObject Win32_IDEController | Format-List *
```

Look for:
- "Intel(R) Volume Management Device NVMe RAID Controller"
- "RST VMD Controller"
- "Intel Rapid Storage Technology"

**If you see any of these:** You MUST include Intel RST drivers!

### Dell Systems That Need Intel RST

- ✅ Optiplex 3000 Series (3080, 3090, 30XX)
- ✅ Optiplex 5000 Series (5080, 5090, 50XX)
- ✅ Optiplex 7000 Series (7080, 7090, 70XX)
- ✅ Optiplex 9000 Series
- ✅ Any system with "Intel VMD" or "Intel Optane" enabled

### Dell BIOS Settings

**Option 1: Disable VMD (Easiest)**
1. Boot into BIOS (F2 at startup)
2. Navigate to: System Configuration → SATA Operation
3. Change from "RAID On" to "AHCI"
4. Save and reboot

**Pros:** No special drivers needed, NVMe visible immediately
**Cons:** Can't use Intel Optane or RAID

**Option 2: Keep VMD Enabled (Requires Drivers)**
1. Keep BIOS on "RAID On" or "Intel Optane"
2. Ensure Intel RST drivers are in `pe_autorun/drivers/Intel_RST/`
3. Drivers will load automatically at WinPE boot

**Pros:** Supports Intel Optane and RAID
**Cons:** Must have correct drivers in ISO

---

## 📥 Download & Setup Workflow

### Step-by-Step for Dell Optiplex

```powershell
# 1. Create driver directories
New-Item -ItemType Directory -Path "pe_autorun\drivers\Intel_RST" -Force
New-Item -ItemType Directory -Path "pe_autorun\drivers\Samsung_NVMe" -Force
New-Item -ItemType Directory -Path "pe_autorun\drivers\Micron_NVMe" -Force

# 2. Download Intel RST (visit link in browser, extract)
# Save extracted folder contents to: pe_autorun\drivers\Intel_RST\

# 3. Download Samsung NVMe drivers (if you have Samsung drives)
# Save to: pe_autorun\drivers\Samsung_NVMe\

# 4. Verify driver files exist
Get-ChildItem -Path "pe_autorun\drivers" -Recurse -Include "*.inf" | Select-Object Name, Directory
```

### Verification Before Building ISO

Run this to check you have the critical drivers:

```powershell
$critical = @("iaStorAVC.inf", "iaStorAC.inf")
$found = Get-ChildItem -Path "pe_autorun\drivers" -Recurse -Include $critical

if ($found.Count -eq $critical.Count) {
    Write-Host "✅ All critical Intel VMD drivers found!" -ForegroundColor Green
} else {
    Write-Host "❌ Missing critical drivers!" -ForegroundColor Red
    Write-Host "Found: $($found.Name -join ', ')"
}
```

---

## 🚀 Testing Your Drivers

### After Building ISO

1. **Boot WinPE from ISO/USB**
2. **Check driver log:**
   ```cmd
   notepad X:\Windows\Temp\GhostWin-DriverLoad.log
   ```
   Look for "✅ Successfully loaded" messages

3. **List loaded drivers:**
   ```powershell
   Get-WindowsDriver -Online | Where-Object {$_.ClassName -eq "SCSIAdapter"}
   ```

4. **Check if disks are visible:**
   ```powershell
   Get-Disk
   diskpart
   list disk
   ```

### If Disks Still Not Visible

1. **Load drivers manually during Windows Setup:**
   - Click "Load Driver"
   - Browse to `X:\PEAutoRun\Drivers\Intel_RST\`
   - Select `iaStorAVC.inf`

2. **Check Device Manager in WinPE:**
   ```cmd
   devmgmt.msc
   ```
   Look for devices with yellow exclamation marks under "Storage Controllers"

3. **Force rescan storage:**
   ```cmd
   echo rescan | diskpart
   ```

---

## 📝 Driver Checklist

Use this checklist before building your ISO:

### Intel RST (Dell Optiplex)
- [ ] Downloaded Intel RST package
- [ ] Extracted F6/Driver folder
- [ ] Copied to `pe_autorun/drivers/Intel_RST/`
- [ ] Verified `iaStorAVC.inf` exists (VMD driver)
- [ ] Verified `iaStorAC.inf` exists (AHCI driver)

### Samsung NVMe (if applicable)
- [ ] Downloaded Samsung NVMe driver
- [ ] Extracted driver files
- [ ] Copied to `pe_autorun/drivers/Samsung_NVMe/`
- [ ] Verified `.inf` file exists

### Micron NVMe (if applicable)
- [ ] Downloaded Micron driver for specific model
- [ ] Extracted driver files
- [ ] Copied to `pe_autorun/drivers/Micron_NVMe/`
- [ ] Verified `.inf` file exists

### Build & Test
- [ ] Run `ghostwin build` to create ISO
- [ ] Boot test system from ISO
- [ ] Check driver load log
- [ ] Verify disks visible in Windows Setup
- [ ] Complete test installation

---

## 🆘 Common Issues & Solutions

### Issue: "No drives were found" in Windows Setup

**Solution 1: Load drivers manually**
- Click "Load Driver" in Windows Setup
- Browse to `X:\PEAutoRun\Drivers\Intel_RST\`
- Select `iaStorAVC.inf` or `iaStorAC.inf`

**Solution 2: Check BIOS settings**
- Change SATA Operation from "RAID On" to "AHCI"
- Disable Intel VMD in BIOS

**Solution 3: Verify driver loaded in WinPE**
```powershell
Get-WindowsDriver -Online | Where-Object {$_.ProviderName -like "*Intel*"}
```

### Issue: Driver loads but disks still not visible

**Cause:** Wrong driver for your chipset generation

**Solution:** Get drivers for YOUR specific platform
- 11th Gen Intel: Use RST 18.x or newer
- 12th/13th Gen Intel: Use RST 19.x or newer
- 14th Gen Intel: Use latest RST drivers

Check Intel's site for "your platform + RapidStorage Technology"

### Issue: "Driver is not signed" error

This shouldn't happen with GhostWin (we use `/ForceUnsigned`), but if it does:

```cmd
bcdedit /set testsigning on
bcdedit /set nointegritychecks on
```

---

## 📚 Additional Resources

### Intel RST Documentation
- https://www.intel.com/content/www/us/en/support/articles/000058758/

### Dell Driver Download
- https://www.dell.com/support/home/
- Enter your Service Tag

### Driver Hardware ID Lookup
- https://devicehunt.com/ - Search by Hardware ID
- https://pcilookup.com/ - PCI Vendor/Device lookup

### Windows Hardware Dev Center
- https://learn.microsoft.com/en-us/windows-hardware/drivers/

---

## ✅ Success Criteria

Your ISO is ready when:

1. ✅ Boot WinPE from ISO
2. ✅ Check `X:\Windows\Temp\GhostWin-DriverLoad.log` shows drivers loaded
3. ✅ Run Windows Setup - all disks visible
4. ✅ Can install Windows without manually loading drivers
5. ✅ Installation completes successfully

---

**GhostWin** - Making Windows deployment actually work on modern hardware! 🚀

# Intel VMD Bypass Guide - Make NVMe Drives Visible Without Drivers

**Problem:** Dell/Lenovo/HP systems with Intel VMD (Volume Management Device) hide NVMe drives during Windows installation.

**Solution:** Either disable VMD in BIOS OR load Intel RST drivers.

---

## 🎯 Recommended Approach

**For fresh Windows installations (wiping factory OS):**
✅ **Disable VMD in BIOS** - No drivers needed, drives visible immediately

**For systems requiring RAID/Intel Optane:**
✅ **Keep VMD enabled + use Intel RST drivers** - GhostWin loads automatically

---

## 🔧 Method 1: Disable Intel VMD in BIOS (Recommended)

### Dell OptiPlex (All Models)

**Steps:**
1. Boot system and press **F2** repeatedly during POST
2. Navigate to: **System Configuration** → **SATA Operation**
3. Change from **"RAID On"** or **"Intel Optane"** to **"AHCI"**
4. Press **F10** to save and exit

**Alternative - Direct VMD Disable:**
1. In BIOS, go to: **System Configuration** → **Integrated Devices**
2. Find **"Intel Volume Management Device (VMD)"**
3. Set to **"Disabled"**
4. Save and Exit (F10)

**Dell Models Affected:**
- OptiPlex 3000/3080/3090/3000 series (2020+)
- OptiPlex 5000/5080/5090/5000 series (2020+)
- OptiPlex 7000/7080/7090/7000 series (2020+)
- OptiPlex 9000 series (2020+)
- Precision workstations with Intel VMD

**Visual Guide:**
```
BIOS Setup
  └── System Configuration
       └── SATA Operation
            ├── [·] AHCI            ← Select this!
            ├── [ ] RAID On         ← Currently selected (VMD active)
            └── [ ] Intel Optane    ← Also uses VMD
```

---

### Lenovo ThinkCentre / ThinkStation

**Steps:**
1. Boot and press **F1** or **F12** during POST (varies by model)
2. Navigate to: **Devices** → **ATA Drive Setup** → **SATA Controller Mode**
3. Change from **"Intel RST"** or **"RAID"** to **"AHCI"**
4. Save and Exit (F10)

**Alternative Path:**
1. Navigate to: **Advanced** → **Intel VMD Technology**
2. Set to **"Disabled"**
3. Save and Exit

**Lenovo Models Affected:**
- ThinkCentre M Series (M70/M80/M90, 2020+)
- ThinkCentre Neo Series
- ThinkStation P Series with Intel VMD
- ThinkPad P Series workstations

**Reference:** https://support.lenovo.com/us/en/solutions/ht506197

**Visual Guide:**
```
BIOS Setup
  └── Devices
       └── ATA Drive Setup
            └── SATA Controller Mode
                 ├── [·] AHCI        ← Select this!
                 ├── [ ] Intel RST   ← VMD active
                 └── [ ] RAID        ← VMD active
```

---

### HP EliteDesk / ProDesk / Z Workstations

**Steps:**
1. Boot and press **F10** during POST
2. Navigate to: **Advanced** → **System Options** → **SATA Emulation**
3. Change from **"RAID"** to **"AHCI"**
4. Save and Exit

**Alternative:**
1. Look for **"Intel VMD Controller"** option
2. Set to **"Disabled"**

**HP Models Affected:**
- EliteDesk 800 G6/G7/G8/G9 (2020+)
- ProDesk 400/600 G6+ (2020+)
- Z Workstations with VMD

---

## ⚙️ Automatic VMD Disable (Dell Only)

GhostWin can automatically disable VMD on Dell systems using **Dell Command | Configure**.

### Setup:

1. **Download Dell Command Configure:**
   ```
   https://www.dell.com/support/kbdoc/000177240
   ```

2. **Extract to ISO:**
   Place `cctk.exe` in: `pe_autorun/system_setup/dell_cctk/`

3. **Auto-Run Script:**
   GhostWin's `Disable-IntelVMD.ps1` will automatically detect and use it

### Manual Use in WinPE:

```cmd
REM Set SATA to AHCI (disables VMD)
cctk.exe --sataoperation=ahci

REM Or directly disable VMD
cctk.exe --intelvmd=disabled

REM Reboot to apply
wpeutil reboot
```

---

## 🚨 Important Warnings

### ⚠️ RAID Configuration Loss

**BEFORE disabling VMD:**
- Changing to AHCI will **destroy RAID arrays**
- **Backup all data** if system has existing RAID
- Only safe for **fresh Windows installations**

### ⚠️ Existing Windows Unbootable

**If Windows is already installed with VMD/RAID:**
- Changing to AHCI will make Windows unbootable (Blue Screen)
- This guide is ONLY for **clean installs**
- For existing systems, use Intel RST drivers instead

### ⚠️ Intel Optane Memory

**If system uses Intel Optane:**
- Optane requires VMD/RAID mode
- Disabling VMD disables Optane
- Use Intel RST drivers instead

---

## 🔄 When to Use Each Method

### Use AHCI Mode (Disable VMD):
✅ Fresh Windows installations
✅ No RAID needed
✅ No Intel Optane
✅ Maximum compatibility
✅ **Recommended for most users**

### Use VMD Mode (Keep Enabled + Drivers):
✅ RAID arrays required
✅ Intel Optane acceleration
✅ Enterprise storage configurations
✅ Requires Intel RST drivers in ISO

---

## 📋 Verification Steps

### After Disabling VMD:

1. **Boot into BIOS** and verify:
   - SATA Operation: **AHCI**
   - Intel VMD: **Disabled** (if option exists)

2. **Boot Windows Setup** (or WinPE):
   ```cmd
   diskpart
   list disk
   ```
   **Expected:** All NVMe drives visible

3. **No drivers needed!** Windows installation proceeds normally

### If Drives Still Not Visible:

1. **Check BIOS again** - verify SATA mode is AHCI
2. **Check for BIOS updates** - update to latest version
3. **Try different BIOS settings:**
   - Disable Secure Boot temporarily
   - Enable Legacy OpROM
4. **As last resort:** Load Intel RST drivers manually in Windows Setup

---

## 🛠️ Troubleshooting

### "No drives found" after changing to AHCI

**Possible Causes:**
1. BIOS change didn't save - re-enter BIOS and verify
2. NVMe drive hardware issue - check in BIOS "Storage" section
3. M.2 slot disabled in BIOS - enable all M.2 slots

**Solutions:**
```cmd
# In WinPE, force rescan
echo rescan | diskpart

# Check for PCI NVMe devices
wmic diskdrive list brief

# List all storage controllers
wmic path Win32_IDEController get /format:list
```

### Can't find SATA/VMD option in BIOS

**Dell Systems:**
- Look under **System Configuration** → **SATA Operation**
- Or **Storage** → **SATA/NVMe Configuration**
- Update BIOS if option missing

**Lenovo Systems:**
- Look under **Config** → **Serial ATA (SATA)**
- Or **Devices** → **ATA Drive Setup**
- May be labeled "SATA Controller Mode"

**HP Systems:**
- Look under **Storage Options**
- Or **Advanced** → **System Options**

### BIOS Password Protected

**Corporate/Enterprise Systems:**
- Contact IT department for BIOS password
- Or use manufacturer's password reset procedure
- Dell: Service Tag + master password
- HP: Master unlock key from support

---

## 📚 Additional Resources

### Official Documentation
- **Dell:** https://www.dell.com/support/kbdoc/en-us/000130110
- **Lenovo:** https://support.lenovo.com/us/en/solutions/ht506197
- **HP:** https://support.hp.com/us-en/document/c06236084
- **Intel:** https://www.intel.com/content/www/us/en/support/articles/000058758/

### Video Tutorials
- Search YouTube: "disable intel vmd [your model]"
- Example: "dell optiplex 7090 disable vmd"

### GhostWin Documentation
- [Driver Guide](driver-guide.md) - If keeping VMD enabled
- [Troubleshooting](../reference/troubleshooting.md) - Common issues

---

## 🎯 Quick Decision Matrix

| Scenario | Recommendation | Why |
|----------|---------------|-----|
| **Fresh Windows install, no RAID** | Disable VMD (AHCI) | Easiest, no drivers needed |
| **Fresh install, need RAID** | Keep VMD + use drivers | RAID requires VMD |
| **Intel Optane system** | Keep VMD + use drivers | Optane requires VMD |
| **Existing Windows (VMD)** | Load drivers in recovery | Changing mode breaks Windows |
| **Enterprise mass deployment** | Disable VMD | Consistent, driver-free |
| **Not sure** | Disable VMD first | Can re-enable if needed |

---

## ✅ Success Checklist

Before deploying Windows:

- [ ] Determined if RAID/Optane needed (Yes = keep VMD + drivers, No = disable VMD)
- [ ] Entered BIOS setup during boot
- [ ] Changed SATA Operation to AHCI (or disabled VMD)
- [ ] Saved BIOS changes (F10)
- [ ] Rebooted system
- [ ] Verified in BIOS that change applied
- [ ] Booted Windows Setup / WinPE
- [ ] Ran `diskpart` → `list disk` → All drives visible ✅
- [ ] Proceeded with Windows installation

---

**GhostWin VMD Bypass** - Making modern hardware actually work for Windows deployment! 🚀

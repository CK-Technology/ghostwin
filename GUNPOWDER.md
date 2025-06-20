# GUNPOWDER Setup Guide 💥

[![Windows](https://img.shields.io/badge/Platform-Windows-0078D4?style=flat-square&logo=windows)](https://www.microsoft.com/windows)
[![Coffee](https://img.shields.io/badge/Powered%20by-Coffee%20%26%20Adrenaline-8B4513?style=flat-square&logo=coffeescript)](https://en.wikipedia.org/wiki/Adrenaline)

**The "I Need This Working Yesterday" Setup Guide for GhostWin**

> *Hey Gunpowder! Since your adrenals are more shot than a coffee shop espresso machine, here's a step-by-step guide that even a sleep-deprived IT manager can follow. No judgment - we've all been there after the 47th "urgent" deployment this week.* ☕💀

---

## 🚨 TL;DR - The Nuclear Option (One-Liner)

**Just want it working RIGHT NOW?** Copy-paste this into PowerShell **as Administrator**:

```powershell
iwr -useb https://raw.githubusercontent.com/yourusername/ghostwin/main/install.ps1 | iex
```

*That's it. Go grab a coffee while it installs. Come back in 5-10 minutes and you're done.*

---

## 📋 Prerequisites (The Boring But Necessary Stuff)

Before we start, make sure you have:

- [ ] **Windows 10/11** (preferably not Windows ME from 2001)
- [ ] **Administrator privileges** (you're the boss, right?)
- [ ] **Internet connection** (surprisingly important for downloading things)
- [ ] **20GB+ free space** (for all the magic to happen)
- [ ] **At least 1 cup of coffee** (optional but highly recommended)

---

## 🎯 Method 1: The "I Trust Scripts" Approach

### Step 1: Open PowerShell as Administrator
1. Press `Win + X`
2. Click **"Windows PowerShell (Admin)"** or **"Terminal (Admin)"**
3. Click **"Yes"** when Windows asks if you want to allow changes
4. You should see a blue window with scary text (this is normal)

### Step 2: Run the Magic Installer
Copy this EXACT line and paste it into PowerShell:

```powershell
iwr -useb https://raw.githubusercontent.com/yourusername/ghostwin/main/install.ps1 | iex
```

### Step 3: Watch the Magic Happen
- The script will install Rust (the programming language, not the thing that destroys metal)
- It will download GhostWin
- It will build everything automatically
- **Go get coffee ☕ - this takes 5-10 minutes**

### Step 4: Verify It Worked
After the script finishes, you should see:
```
🎉 GhostWin Installation Complete!
```

If you see that, **you're done!** Skip to the [Testing Section](#-testing-does-this-thing-actually-work).

---

## 🛠️ Method 2: The "I Don't Trust Scripts" Approach

*For when you're feeling paranoid (which is healthy in IT)*

### Step 1: Install Rust Manually
1. Go to [https://rustup.rs/](https://rustup.rs/)
2. Download the **"rustup-init.exe"** file
3. Run it (yes, as Administrator)
4. Follow the prompts (just hit Enter for defaults)
5. **Restart your terminal** when it's done

### Step 2: Verify Rust Works
Open a new PowerShell and type:
```powershell
cargo --version
```
You should see something like `cargo 1.75.0` (version may vary)

### Step 3: Download GhostWin
Pick your poison:

**Option A: Git (if you have it)**
```powershell
git clone https://github.com/yourusername/ghostwin.git
cd ghostwin
```

**Option B: Download ZIP (simpler)**
1. Go to the GitHub repo in your browser
2. Click the green **"Code"** button
3. Click **"Download ZIP"**
4. Extract it to `C:\GhostWin\` or wherever you want

### Step 4: Build It
```powershell
cd C:\GhostWin  # or wherever you put it
cargo build --release
```

**This takes 5-10 minutes. Perfect time for:**
- ☕ Coffee refill
- 📧 Checking if anything's on fire
- 🎯 Questioning your life choices that led to manual software compilation

---

## 🧪 Testing: Does This Thing Actually Work?

### Quick Test
```powershell
cd C:\GhostWin  # or wherever you installed it
.\target\release\ghostwin.exe --version
```

You should see: `ghostwin 0.1.0` or similar.

### Full Test (Launch the GUI)
```powershell
.\target\release\ghostwin.exe gui
```

**If you see a beautiful dark blue interface with "GhostWin" branding:**
🎉 **SUCCESS!** You're ready to impress people.

**If you see error messages:**
😅 Continue to the [Troubleshooting Section](#-when-things-go-sideways-troubleshooting).

---

## 🚀 Quick Start: Your First ISO

### What You Need
- A Windows 11 ISO file (download from Microsoft)
- About 30 minutes
- Strong coffee ☕

### The Process
1. **Put your Windows ISO somewhere obvious:**
   ```
   C:\WindowsISOs\Windows11.iso
   ```

2. **Navigate to GhostWin:**
   ```powershell
   cd C:\GhostWin
   ```

3. **Build your custom ISO:**
   ```powershell
   .\target\release\ghostwin.exe build --source-iso "C:\WindowsISOs\Windows11.iso" --output-iso "C:\GhostWin-Custom.iso"
   ```

4. **Wait and watch the magic:**
   - It extracts the ISO
   - Mounts the Windows image
   - Injects all the GhostWin tools
   - Builds a new bootable ISO
   - **Takes 10-20 minutes depending on your machine**

5. **Boot from the new ISO:**
   - Burn `C:\GhostWin-Custom.iso` to a USB (use Rufus)
   - Boot a test machine
   - **Marvel at your beautiful custom Windows deployment interface**

---

## 🎭 Demo Mode: Impressing Your Boss

### The 5-Minute Demo Script

1. **Open GhostWin GUI:**
   ```powershell
   .\target\release\ghostwin.exe gui
   ```

2. **Point out the professional dark theme:**
   *"This is our new deployment interface - much more professional than those old batch scripts."*

3. **Show the installation modes:**
   *"Normal install for standard deployments, Automated for full hands-off installation."*

4. **Demonstrate tool management:**
   *"All our diagnostic tools are integrated into one interface - no more hunting for utilities."*

5. **Highlight remote access:**
   *"Built-in VNC means we can assist with difficult deployments remotely."*

6. **Emphasize business value:**
   *"Faster deployments, professional appearance on client sites, reduced training time."*

### What NOT to Mention
- ❌ "I spent 3 days fighting with Rust compilation errors"
- ❌ "This replaced my 500-line batch file monstrosity"
- ❌ "The old system broke every Tuesday"
- ❌ "I found this on GitHub at 2 AM"

### What TO Emphasize
- ✅ "Modern, reliable deployment toolkit"
- ✅ "Professional interface suitable for client demonstrations"
- ✅ "Integrated remote access capabilities"
- ✅ "Standardized deployment process"

---

## 💥 When Things Go Sideways: Troubleshooting

### "Administrator privileges required"
**Fix:** Right-click PowerShell → "Run as administrator"
*Yes, you need admin rights. Windows is paranoid like that.*

### "Rust installation failed"
**Fixes:**
1. Download rustup-init.exe directly from [rustup.rs](https://rustup.rs/)
2. Run it manually
3. Restart your terminal
4. Try the GhostWin installer again

### "Command 'cargo' is not recognized"
**Fix:** Restart your terminal after installing Rust
*Windows needs a gentle nudge to recognize new programs*

### "Build failed with strange errors"
**Fixes:**
1. Make sure you have internet (Rust downloads dependencies)
2. Check you have 5GB+ free space
3. Try running from a different directory
4. Restart your computer (the IT classic)

### "GUI doesn't launch"
**Possible causes:**
- Missing Visual C++ Redistributable (download from Microsoft)
- Antivirus blocking the executable (add exception)
- Running on Windows Server without desktop experience

### "ISO build fails"
**Common issues:**
- Not running as Administrator
- Source ISO is corrupted (re-download)
- Insufficient disk space (need 20GB+)
- Windows ADK not installed

---

## 🏆 Advanced Configuration (For When You're Feeling Fancy)

### Custom Tool Integration
1. Create folders in your GhostWin directory:
   ```
   mkdir tools
   mkdir pe_autorun
   mkdir scripts
   ```

2. Drop your tools in the appropriate folders:
   - **tools/**: Manual utilities (shown in GUI)
   - **pe_autorun/**: Auto-run scripts (execute at boot)
   - **scripts/**: Post-install automation

3. Run validation:
   ```powershell
   .\target\release\ghostwin.exe validate
   ```

### VNC Remote Access Setup
Edit `ghostwin.toml`:
```toml
[security]
vnc_enabled = true
vnc_port = 5950
vnc_password = "yourpassword"
```

### Automation Scripts
Create custom deployment scripts in the `scripts/` folder for:
- Software installation
- Registry modifications
- System configuration
- User account setup

---

## 📞 When All Else Fails

### Emergency Contacts
- **Developer:** "Hey, your thing broke again" 
- **Coffee Machine:** Extension 911
- **Rubber Duck:** Silent but effective debugging partner

### Nuclear Reset Option
If everything is completely broken:
1. Delete the GhostWin folder
2. Run the installer script again
3. Blame cosmic rays or solar flares
4. Consider a career in agriculture

---

## 🎯 Success Metrics

**You know it's working when:**
- ✅ The GUI launches without errors
- ✅ Tools are detected and listed
- ✅ You can build a custom ISO
- ✅ Your stress levels decrease by at least 23%
- ✅ You feel like a deployment wizard 🧙‍♂️

**Time to panic when:**
- ❌ Nothing works and you have a demo in 30 minutes
- ❌ The boss is asking technical questions
- ❌ The coffee machine is broken
- ❌ It's Monday

---

## 💡 Pro Tips from the Trenches

1. **Always test on a VM first** - because production is not a playground
2. **Keep backup ISOs** - because Murphy's Law is real
3. **Document your custom configs** - because future you will thank present you
4. **Have a rollback plan** - because optimism doesn't fix broken deployments
5. **Keep the coffee machine stocked** - because priorities

---

*Good luck, Gunpowder! May your deployments be swift, your ISOs bootable, and your adrenaline levels return to something resembling normal. Remember: if it works, don't touch it. If it doesn't work, restart it. If it still doesn't work, blame DNS.* 

**– The Management (and your friendly neighborhood developer)** 🎯☕💥

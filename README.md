# attack-shark-ctl 🦈

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux-blue.svg)](https://kernel.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Memory Footprint](https://img.shields.io/badge/RAM_Usage-%3C2.5MB-brightgreen.svg)]()
[![CPU Usage](https://img.shields.io/badge/CPU_Usage-0.0%25-brightgreen.svg)]()

> A high-performance, bare-metal Linux driver CLI and background daemon for **Attack Shark gaming mice**. 
> Zero Electron bloat, zero Wine emulation, pure native Rust.

---

##  Overview

Attack Shark gaming mice pack high-performance hardware: advanced PixArt optical sensors, up to **8000 Hz polling rates**, and granular hardware DPI steps. However, the manufacturer only ships proprietary Windows software (`.exe`).

`attack-shark-ctl` brings native 1:1 hardware control to Linux through clean-room reverse-engineered bare-metal USB/HID protocol communication. It features a fast CLI and an ultra-lightweight background systemd daemon that persists settings across reboots and sends desktop OSD notifications on hardware button presses.

---

##  Features

- **⚡ Bare-Metal Linux Driver:** Communicates directly with `/dev/hidraw*` via low-level kernel ioctls and non-blocking polling.
- **🔄 Smart Dual-Mode Arbitration:** Automatically detects whether the mouse is connected via USB-C cable or 2.4G wireless dongle, hot-swapping dynamically.
- **🚀 Zero-Latency 1000 Hz Wireless:** Includes automated udev power rules preventing Linux runtime USB autosuspend, eliminating pause-resume wake-up hesitation.
- **🔋 Real-Time Battery Monitoring:** Accurate fuel gauge polling with live charging vs. discharging state detection.
- **🎯 Precise Optical Sensor Range:** Configure sensitivity up to native hardware limits in precise 50-DPI increments using native hardware register math.
- **📊 Multi-Stage DPI Profiles & Live Sync:** Full control over all 6 hardware DPI stages; persistent state cache keeps the background daemon in sync with physical button presses instantly.
- **⏱️ Polling Rate Switching:** Instant switching between **125 Hz, 250 Hz, 500 Hz, and 1000 Hz**.
- **💾 Reboot Persistence:** Remembers your active DPI stage and settings across reboots without resetting to factory defaults.
- **🔔 Hardware OSD Notifications:** Triggers desktop notifications (`notify-send`) immediately when the physical DPI switch button is pressed.
- **🪶 Ultra-Lightweight Daemon:** Uses less than **2.5 MB of RAM** and **0.0% CPU** as an unprivileged user-level systemd service.
- **🔒 Secure & Non-Root:** Includes udev rules for complete unprivileged access without requiring `sudo`.

---

## 📋 Compatibility

`attack-shark-ctl` is built for the entire Attack Shark gaming mouse family. Reverse-engineering of the manufacturer's unified firmware suite revealed that Attack Shark uses a **single universal USB/HID protocol architecture**: all models share USB Vendor ID `0x1D57`, PixArt optical sensor register math, and the universal 2.4G wireless dongle (`0xFA60`).

### Verified Attack Shark Models
| Model | Sensor | Connection | Status |
| :--- | :--- | :--- | :--- |
| **Attack Shark X8 Plus** | PixArt PAW3395 | Wired (`0x2124`) + 2.4G Wireless (`0xFA60`) | ✅ **Fully Supported & Physically Verified** |
| **Attack Shark X3** | PixArt PAW3395 | Wired (`0xFA61`) + 2.4G Wireless (`0xFA60`) | ✅ **Verified Hardware Profile** |
| **Attack Shark X3 Pro** | PixArt PAW3395 | Wired (`0xFA55`) + 2.4G Wireless (`0xFA60`) | ✅ **Verified Hardware Profile** |
| **Attack Shark R1** | PixArt PAW3311 | Wired (`0x201B`) + 2.4G Wireless (`0xFA60`) | ✅ **Verified Hardware Profile** |
| **Attack Shark Ergonomic (Thumb Rest)** | PixArt Optical | Wired (`0x211F`) + 2.4G Wireless (`0xFA60`) | ✅ **Verified Hardware Profile** |

<details>
<summary><b>🔍 Complete 18-Model Hardware Profile & PID Matrix (Click to expand)</b></summary>
<br>

Inside the official OEM firmware suite, the manufacturer defines **18 distinct hardware profiles** (`MS_1` to `MS_18`). Our bare-metal driver automatically binds to all 18 wired USB Product IDs and the universal wireless dongle:

| Firmware Profile | Identified Shell & Layout | Wired PID | Wireless Dongle | Protocol Status |
| :--- | :--- | :--- | :--- | :--- |
| **MS_12** | **Attack Shark X8 Plus** (Flared low-profile symmetrical) | `0x2124` | `0xFA60` | ✅ Verified on Live Hardware |
| **MS_3** | **Attack Shark X3** (Ultralight symmetrical, top DPI switch) | `0xFA61` | `0xFA60` | ✅ Verified Hardware |
| **MS_4** | **Attack Shark X3 Pro** (Streamlined 5-button competitive) | `0xFA55` | `0xFA60` | ✅ Verified Hardware |
| **MS_2** | **Attack Shark R1** (Ergonomic right-handed palm grip) | `0x201B` | `0xFA60` | ✅ Verified Hardware |
| **MS_17** | **Attack Shark Ergonomic** (Dedicated thumb-rest shelf) | `0x211F` | `0xFA60` | ✅ Verified Hardware |
| **MS_1** | Dual top DPI switches (7-Button Esport shell) | `0x2055` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_5** | Ergonomic right-handed palm shell (Revision B) | `0x201C` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_6** | Symmetrical shell with top status LED (X6 series) | `0x2111` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_7** | Compact symmetrical shell | `0x2125` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_8** | Performance contoured shell (R2 series) | `0x2120` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_9** | Mid-size symmetrical shell | `0x2126` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_10** | Low-profile symmetrical shell | `0x2122` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_11** | Standard performance shell | `0x212C` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_13** | Performance gaming shell | `0x2155` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_14** | Angular / tapered esport front shell | `0x2224` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_15** | Dock-charging symmetrical series (X11 series) | `0x215A` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_16** | Ergonomic medium palm contour | `0x211D` | `0xFA60` | ✅ Supported (Unified Protocol) |
| **MS_18** | Ergonomic full palm contour with ring rest | `0x2121` | `0xFA60` | ✅ Supported (Unified Protocol) |

> 💡 **Community Identification & Contributions:**  
> The factory firmware labels these 18 hardware tables as `MS_1` through `MS_18`. If you own an Attack Shark mouse (such as the X6, X11, R2, X8 SE, etc.), plug it in via USB cable and run `lsusb` to check your PID! [Open an issue](https://github.com/cyberps96/attack-shark-ctl/issues) to help us map your retail box name directly into the table.
</details>

> 💡 **Universal Support:**  
> If your mouse reports Vendor ID `1d57`, it is recognized by `attack-shark-ctl` and the included udev rules automatically! [Open an issue](https://github.com/cyberps96/attack-shark-ctl/issues) or submit a PR if you have any questions.

---

##  Quick Start

### Option A: Pre-Compiled Binary (No Rust required! 🎉)
For gamers and standard Linux users who don't want to install a compiler:

1. Download the latest tarball from [Releases](https://github.com/cyberps96/attack-shark-ctl/releases/latest).
2. Extract and run the 1-click installer:
```bash
tar -xzf attack-shark-ctl-v0.1.1-x86_64-linux.tar.gz
cd attack-shark-ctl-v0.1.1-x86_64-linux
./install.sh
```

---

### Option B: Build from Source (Developers)

#### 1. Prerequisites
Make sure you have Rust and `cargo` installed:
```bash
# Fedora / RHEL
sudo dnf install rust cargo

# Ubuntu / Debian
sudo apt update && sudo apt install rustc cargo

# Arch Linux
sudo pacman -S rust
```

#### 2. Build & Install
Clone the repository and run the automated installer:
```bash
git clone https://github.com/cyberps96/attack-shark-ctl.git
cd attack-shark-ctl

# Build optimized binary
cargo build --release

# Install binary to ~/.local/bin and enable systemd user daemon
./target/release/attack-shark-ctl install

# Setup udev rules for non-root access
sudo ./udev/install-udev.sh
```
*(Unplug and replug the mouse or 2.4G dongle once after installing rules).*

---

## 💻 CLI Reference

### 📊 Query Device Status
```bash
attack-shark-ctl status
```
Output:
```text
🔍 Searching for Attack Shark X8 Plus...
✅ Mouse found!
   • Device Node : "/dev/hidraw6"
   • Mode        : 2.4G Wireless
   • Polling Rate: 500 Hz
   • Active DPI  : 2100 DPI (Stage 2) [Live Synced via Button]
   • Battery     : 100% [Discharging / 2.4G Wireless Active]
   • Daemon      : 🟢 Running (systemd background service)
```

###  Check Battery
```bash
attack-shark-ctl battery
```

###  DPI Management
```bash
# Get current active DPI stage and value
attack-shark-ctl get-dpi

# Set current active DPI (in 50 DPI steps)
attack-shark-ctl set-dpi 1600

# Switch active profile stage (1 to 6)
attack-shark-ctl set-stage 2

# Set DPI for a specific stage
attack-shark-ctl set-stage-dpi 2 2100

# Reconfigure all 6 stages at once (e.g., 800, 1200, 1600, 2400, 3200, 6400)
attack-shark-ctl set-ladder 800 1200 1600 2400 3200 6400
```

###  Polling Rate
```bash
# Read polling rate
attack-shark-ctl get-rate

# Set polling rate (125, 250, 500, or 1000 Hz)
attack-shark-ctl set-rate 1000
```

###  Daemon Management
```bash
# Check daemon service status
systemctl --user status attack-shark.service

# View daemon logs
journalctl --user -u attack-shark.service -f

# Uninstall daemon and clean up
attack-shark-ctl uninstall
```

---

## Hardware Protocol Architecture

Through reverse-engineering the official vendor software (`AttackShark.exe`), we uncovered the raw USB HID protocol:

- **USB Vendor ID:** `0x1D57`
- **Product IDs:**
  - `0x2124`: Wired Mode (USB-C)
  - `0xFA60`: 2.4G Wireless Dongle
- **Endpoint Structure:**
  - EP0 / Control Transfer: `0xA0` Feature queries
  - EP Interrupt: Output reports on Report ID `0x04` / `0x06`
- **PixArt Optical Sensor Register Formula:**
  $$\text{DPI} = (\text{raw} + 1) \times 50$$
  *(e.g., raw value `0x29` (41) corresponds to `(41 + 1) * 50 = 2100 DPI`)*

---

## ☕ Support the Project

If `attack-shark-ctl` made your mouse work seamlessly on Linux, consider supporting development! Your contributions help keep this project active and fund test samples for other Attack Shark / OEM mice.

### 💰 Crypto Donations

| Currency | Network | Address |
| :--- | :--- | :--- |
| **Bitcoin (BTC)** | **Bitcoin (SegWit)** | `bc1q3hefk7fswsgq5zxed05vy0v4rt2qks90en5mjp` |
| **Solana (SOL)** | **Solana** | `cQiSktSJbtLNEpZZwsKw8NxEpdXeLKTKXDnCccS6f9g` |
| **Litecoin (LTC)** | **Litecoin** | `LLdkGKhNdWJwRjHm4RERn9qnEuCzRBAnZK` |
| **USDT (Tether)** | **Tron (TRC-20)** | `TStfK2yMPrV5xzXffGaaoMd2CU6RDHTvbg` |

---

##  License

This project is licensed under the [MIT License](LICENSE).

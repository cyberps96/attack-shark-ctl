# attack-shark-ctl 🦈

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux-blue.svg)](https://kernel.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Memory Footprint](https://img.shields.io/badge/RAM_Usage-%3C2.5MB-brightgreen.svg)]()
[![CPU Usage](https://img.shields.io/badge/CPU_Usage-0.0%25-brightgreen.svg)]()

> A high-performance, bare-metal Linux driver CLI and background daemon for **Attack Shark gaming mice** (PixArt PAW3395 sensor). 
> Zero Electron bloat, zero Wine emulation, pure native Rust.

---

## ⚡ Overview

Attack Shark gaming mice (such as the **X8 Plus**, **X3**, **X6**) pack top-of-the-line hardware: the flagship **PixArt PAW3395 sensor**, up to **40,000 DPI**, and **1000 Hz polling rates**. However, the manufacturer only ships proprietary Windows software (`.exe`).

`attack-shark-ctl` brings native 1:1 hardware control to Linux through clean-room reverse-engineered bare-metal USB/HID protocol communication. It features a fast CLI and an ultra-lightweight background systemd daemon that persists settings across reboots and sends desktop OSD notifications on hardware button presses.

---

## ✨ Features

- **🦈 Bare-Metal Linux Driver:** Communicates directly with `/dev/hidraw*` via low-level kernel ioctls and non-blocking polling.
- **🔌 Smart Dual-Mode Arbitration:** Automatically detects whether the mouse is connected via USB-C cable or 2.4G wireless dongle, hot-swapping dynamically.
- **🔋 Real-Time Battery Monitoring:** Accurate fuel gauge polling with live charging vs. discharging state detection.
- **🎯 Full PAW3395 Sensor Range:** Configure sensitivity from **100 to 40,000 DPI** in precise 50-DPI increments using native hardware register math.
- **🪜 Multi-Stage DPI Profiles:** Full control over all 6 hardware DPI stages and their respective RGB status indicators.
- **⚡ Polling Rate Switching:** Instant switching between **125 Hz, 250 Hz, 500 Hz, and 1000 Hz**.
- **💾 Reboot Persistence:** Remembers your active DPI stage and settings across reboots without resetting to factory defaults.
- **🔔 Hardware OSD Notifications:** Triggers desktop notifications (`notify-send`) immediately when the physical DPI switch button is pressed.
- **🪶 Ultra-Lightweight Daemon:** Uses less than **2.5 MB of RAM** and **0.0% CPU** as an unprivileged user-level systemd service.
- **🛡️ Secure & Non-Root:** Includes udev rules for complete unprivileged access without requiring `sudo`.

---

## 📋 Compatibility

| Model | Sensor | Connection | Status |
| :--- | :--- | :--- | :--- |
| **Attack Shark X8 Plus** | PixArt PAW3395 | Wired (Type-C) + 2.4G Wireless | ✅ **Fully Supported & Verified** |
| **Attack Shark X3 / X3 Pro** | PixArt PAW3395 | Wired + 2.4G | 🧪 Protocol Compatible (Testing) |
| **Attack Shark X6** | PixArt PAW3395 | Wired + 2.4G + BT | 🧪 Protocol Compatible (Testing) |
| **Attack Shark R1** | PixArt PAW3311 | Wired + 2.4G | 🧪 Protocol Compatible (Testing) |

> 💡 **Have another Attack Shark or OEM mouse?**  
> Run `lsusb` to check your vendor/product IDs. If your mouse has VID `1d57`, it likely shares this exact protocol! [Open an issue](https://github.com/cyberps96/attack-shark-ctl/issues) or submit a PR.

---

## 🚀 Quick Start

### Option A: Pre-Compiled Binary (No Rust required! 🎉)
For gamers and standard Linux users who don't want to install a compiler:

1. Download the latest tarball from [Releases](https://github.com/cyberps96/attack-shark-ctl/releases/latest).
2. Extract and run the 1-click installer:
```bash
tar -xzf attack-shark-ctl-v0.1.0-x86_64-linux.tar.gz
cd attack-shark-ctl-v0.1.0-x86_64-linux
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

### 🔋 Check Battery
```bash
attack-shark-ctl battery
```

### 🎯 DPI Management
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

### ⚡ Polling Rate
```bash
# Read polling rate
attack-shark-ctl get-rate

# Set polling rate (125, 250, 500, or 1000 Hz)
attack-shark-ctl set-rate 1000
```

### ⚙️ Daemon Management
```bash
# Check daemon service status
systemctl --user status attack-shark.service

# View daemon logs
journalctl --user -u attack-shark.service -f

# Uninstall daemon and clean up
attack-shark-ctl uninstall
```

---

## 🔬 Hardware Protocol Architecture

Through reverse-engineering the official vendor software (`AttackShark.exe`), we uncovered the raw USB HID protocol:

- **USB Vendor ID:** `0x1D57`
- **Product IDs:**
  - `0x2124`: Wired Mode (USB-C)
  - `0xFA60`: 2.4G Wireless Dongle
- **Endpoint Structure:**
  - EP0 / Control Transfer: `0xA0` Feature queries
  - EP Interrupt: Output reports on Report ID `0x04` / `0x06`
- **PixArt PAW3395 Register Formula:**
  $$\text{DPI} = (\text{raw} + 1) \times 50$$
  *(e.g., raw value `0x29` (41) corresponds to `(41 + 1) * 50 = 2100 DPI`)*

---

## ☕ Support the Project

If `attack-shark-ctl` made your mouse work seamlessly on Linux, consider supporting development! Your contributions help keep this project active and fund test samples for other Attack Shark / OEM mice.

### 🪙 Crypto Donations

| Currency | Network | Address |
| :--- | :--- | :--- |
| **USDT (Tether)** | **Tron (TRC-20)** | `TStfK2yMPrV5xzXffGaaoMd2CU6RDHTvbg` |

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).

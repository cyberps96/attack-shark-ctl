mod device;
mod protocol;

use clap::{Parser, Subcommand};
use device::MouseDevice;
use protocol::PollingRate;

fn state_dir() -> std::path::PathBuf {
    let dir = if let Ok(state_home) = std::env::var("XDG_STATE_HOME") {
        std::path::PathBuf::from(state_home).join("attack-shark")
    } else if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".local/state/attack-shark")
    } else {
        std::path::PathBuf::from("/tmp/attack-shark")
    };
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn get_cached_rate() -> Option<u32> {
    if let Ok(s) = std::fs::read_to_string(state_dir().join("rate")) {
        if let Ok(val) = s.trim().parse::<u32>() {
            return Some(val);
        }
    }
    std::fs::read_to_string("/tmp/attack-shark-rate")
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

fn set_cached_rate(rate: u32) {
    let _ = std::fs::write(state_dir().join("rate"), rate.to_string());
    let _ = std::fs::write("/tmp/attack-shark-rate", rate.to_string());
}

fn get_live_stage() -> Option<u8> {
    if let Ok(s) = std::fs::read_to_string(state_dir().join("stage")) {
        if let Ok(val) = s.trim().parse::<u8>() {
            if (1..=6).contains(&val) {
                return Some(val);
            }
        }
    }
    std::fs::read_to_string("/tmp/attack-shark-stage")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .filter(|&s| (1..=6).contains(&s))
}

fn set_live_stage(stage: u8) {
    let _ = std::fs::write(state_dir().join("stage"), stage.to_string());
    let _ = std::fs::write("/tmp/attack-shark-stage", stage.to_string());
}

fn get_cached_battery() -> Option<u8> {
    if let Ok(s) = std::fs::read_to_string(state_dir().join("battery")) {
        if let Ok(val) = s.trim().parse::<u8>() {
            return Some(val);
        }
    }
    std::fs::read_to_string("/tmp/attack-shark-battery")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
}

fn set_cached_battery(pct: u8) {
    let _ = std::fs::write(state_dir().join("battery"), pct.to_string());
    let _ = std::fs::write("/tmp/attack-shark-battery", pct.to_string());
}

fn format_battery_status(status_code: u8) -> &'static str {
    match status_code {
        2 => "Charging ⚡",
        3 => "USB Wired / Fully Charged ⚡",
        _ => "Discharging / 2.4G Wireless Active",
    }
}

fn send_desktop_notification(title: &str, body: &str, urgency: &str) {
    let _ = std::process::Command::new("notify-send")
        .args([
            "-a",
            "Attack Shark",
            "-i",
            "input-mouse",
            "-t",
            "3000",
            "-u",
            urgency,
            title,
            body,
        ])
        .spawn();
}

fn simple_time() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let local_secs = secs + 3 * 3600; // Local time UTC+3
    let s = local_secs % 60;
    let m = (local_secs / 60) % 60;
    let h = (local_secs / 3600) % 24;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

#[derive(Parser)]
#[command(name = "attack-shark-ctl")]
#[command(author = "Cyber & Antigravity")]
#[command(version = "0.2.0")]
#[command(
    about = "Bare-metal Linux driver CLI for Attack Shark gaming mice (X8 Plus, X3, X6, R1, and all 18 models)",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get mouse status, connection mode, polling rate, active DPI, and battery
    Status,

    /// Query the mouse battery percentage directly from hardware
    Battery,

    /// Live background monitor for physical DPI button clicks, desktop OSD alerts, and battery tracking
    Monitor {
        /// Optional timeout in seconds (runs indefinitely if omitted)
        #[arg(short, long)]
        seconds: Option<u64>,
    },

    /// Query the current hardware polling rate from mouse MCU
    GetRate,

    /// Set hardware polling rate directly in mouse internal flash (125, 250, 500, 1000)
    SetRate {
        /// Polling rate in Hz (125, 250, 500, 1000)
        rate: u32,
    },

    /// Query all DPI stages and show the currently active stage
    GetDpi,

    /// Set DPI value for active stage, or all stages with --all (e.g. 800, 1200, 1600, 3200)
    #[command(alias = "dpi")]
    SetDpi {
        /// DPI value (between 100 and 26000, step of 100)
        dpi: u32,
        /// Set all 6 presets to this DPI (locks mouse DPI against accidental button clicks)
        #[arg(short, long)]
        all: bool,
    },

    /// Set all 6 presets to the same DPI (locks mouse DPI against accidental button clicks)
    SetAllDpi {
        /// DPI value (between 100 and 26000, step of 100)
        dpi: u32,
    },

    /// Switch active DPI stage (1 to 6)
    #[command(alias = "stage")]
    SetStage {
        /// Stage number (1 to 6)
        stage: u8,
    },

    /// Configure DPI: 'set-stage-dpi <DPI>' or 'set-stage-dpi <STAGE> <DPI>'
    SetStageDpi {
        /// Stage number (1 to 6) or DPI value if single argument
        stage_or_dpi: u32,
        /// DPI value if stage was specified first
        dpi: Option<u32>,
    },

    /// Program all 6 stages at once with a ladder (e.g. 800, 1200, 1600, 2100, 2600, 3200)
    SetLadder {
        /// Stage 1 DPI (default: 800)
        #[arg(default_value = "800")]
        s1: u32,
        /// Stage 2 DPI (default: 1200)
        #[arg(default_value = "1200")]
        s2: u32,
        /// Stage 3 DPI (default: 1600)
        #[arg(default_value = "1600")]
        s3: u32,
        /// Stage 4 DPI (default: 2100)
        #[arg(default_value = "2100")]
        s4: u32,
        /// Stage 5 DPI (default: 2600)
        #[arg(default_value = "2600")]
        s5: u32,
        /// Stage 6 DPI (default: 3200)
        #[arg(default_value = "3200")]
        s6: u32,
    },

    /// Install attack-shark-ctl system-wide to ~/.local/bin and enable background autostart service
    Install,

    /// Completely uninstall attack-shark-ctl, disable background service, and clean up
    Uninstall,

    /// Manage the background monitor service (status, start, stop, restart, enable, disable, logs)
    Service {
        #[command(subcommand)]
        action: Option<ServiceAction>,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum ServiceAction {
    /// Show current background service status
    Status,
    /// Start background service
    Start,
    /// Stop background service
    Stop,
    /// Restart background service
    Restart,
    /// Enable and start background service on boot
    Enable,
    /// Disable and stop background service on boot
    Disable,
    /// View live background service log output
    Logs,
}

fn handle_install() {
    println!("🚀 Installing attack-shark-ctl for your Linux user session...\n");

    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Failed to get current executable path: {}", e);
            std::process::exit(1);
        }
    };

    let home = match std::env::var("HOME") {
        Ok(h) => std::path::PathBuf::from(h),
        Err(_) => {
            eprintln!("❌ Could not determine user HOME directory.");
            std::process::exit(1);
        }
    };

    let bin_dir = home.join(".local/bin");
    let target_bin = bin_dir.join("attack-shark-ctl");

    if let Err(e) = std::fs::create_dir_all(&bin_dir) {
        eprintln!("❌ Failed to create directory {:?}: {}", bin_dir, e);
        std::process::exit(1);
    }

    if current_exe != target_bin {
        println!("📦 Copying binary to {:?}...", target_bin);
        if let Err(e) = std::fs::copy(&current_exe, &target_bin) {
            eprintln!("❌ Failed to copy binary: {}", e);
            std::process::exit(1);
        }
    }

    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(&target_bin, std::fs::Permissions::from_mode(0o755));
    println!("✅ Binary installed to {:?}", target_bin);

    // Initialize state directory
    let _ = state_dir();

    // Create systemd service unit
    let service_dir = home.join(".config/systemd/user");
    if let Err(e) = std::fs::create_dir_all(&service_dir) {
        eprintln!("❌ Failed to create systemd user dir: {}", e);
        std::process::exit(1);
    }

    let service_file = service_dir.join("attack-shark.service");
    let service_content = format!(
"[Unit]
Description=Attack Shark Gaming Mice Background Monitor Daemon
Documentation=https://github.com/cyberps96/attack-shark-ctl
After=graphical-session.target default.target

[Service]
Type=simple
ExecStart={}/.local/bin/attack-shark-ctl monitor
Restart=always
RestartSec=3
Environment=PATH=/usr/local/bin:/usr/bin:/bin:{}/.local/bin

[Install]
WantedBy=default.target
",
        home.display(),
        home.display()
    );

    if let Err(e) = std::fs::write(&service_file, service_content) {
        eprintln!("❌ Failed to write systemd unit: {}", e);
        std::process::exit(1);
    }
    println!("⚙️ Created systemd service unit: {:?}", service_file);

    println!("⚡ Enabling and starting background monitor daemon...");
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    let enable_res = std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", "attack-shark.service"])
        .status();

    match enable_res {
        Ok(s) if s.success() => {
            println!("✅ Background monitor service enabled & started!");
        }
        _ => {
            eprintln!("⚠️ Warning: Failed to enable service via systemctl.");
        }
    }

    let path_env = std::env::var("PATH").unwrap_or_default();
    let has_in_path = path_env.split(':').any(|p| std::path::Path::new(p) == bin_dir);

    println!("\n╔═══════════════════════════════════════════════════════════════════╗");
    println!("║              🎉 Installation Completed Successfully!              ║");
    println!("╠═══════════════════════════════════════════════════════════════════╣");
    println!("║  • Binary Path   : ~/.local/bin/attack-shark-ctl                 ║");
    println!("║  • Service Name  : attack-shark.service (Enabled on boot)        ║");
    println!("║  • Background OSD: ACTIVE (Desktop toasts enabled 24/7)           ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝");

    if !has_in_path {
        println!("\n💡 Tip: Add ~/.local/bin to your PATH in ~/.bashrc if needed:");
        println!("   echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> ~/.bashrc");
    }

    println!("\n👉 Try clicking your physical DPI button right now to see the desktop toast!\n");
}

fn handle_uninstall() {
    println!("🗑️ Uninstalling attack-shark-ctl...\n");

    let home = std::env::var("HOME").map(std::path::PathBuf::from).ok();

    println!("🛑 Stopping and disabling background service...");
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "attack-shark.service"])
        .status();

    if let Some(h) = &home {
        let service_file = h.join(".config/systemd/user/attack-shark.service");
        if service_file.exists() {
            let _ = std::fs::remove_file(&service_file);
            println!("🧹 Removed {:?}", service_file);
        }

        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();

        let target_bin = h.join(".local/bin/attack-shark-ctl");
        if target_bin.exists() {
            let _ = std::fs::remove_file(&target_bin);
            println!("🧹 Removed {:?}", target_bin);
        }

        let state_dir = h.join(".local/state/attack-shark");
        if state_dir.exists() {
            let _ = std::fs::remove_dir_all(&state_dir);
            println!("🧹 Cleaned state directory {:?}", state_dir);
        }
    }

    let _ = std::fs::remove_file("/tmp/attack-shark-stage");
    let _ = std::fs::remove_file("/tmp/attack-shark-battery");
    let _ = std::fs::remove_file("/tmp/attack-shark-rate");

    println!("\n✅ attack-shark-ctl has been cleanly uninstalled from your system.");
}

fn handle_service(action: Option<ServiceAction>) {
    let act = action.unwrap_or(ServiceAction::Status);
    match act {
        ServiceAction::Status => {
            println!("📡 Checking background service status...\n");
            let _ = std::process::Command::new("systemctl")
                .args(["--user", "status", "attack-shark.service"])
                .status();
        }
        ServiceAction::Start => {
            println!("⚡ Starting background service...");
            let status = std::process::Command::new("systemctl")
                .args(["--user", "start", "attack-shark.service"])
                .status();
            if let Ok(s) = status {
                if s.success() { println!("✅ Service started!"); }
            }
        }
        ServiceAction::Stop => {
            println!("🛑 Stopping background service...");
            let status = std::process::Command::new("systemctl")
                .args(["--user", "stop", "attack-shark.service"])
                .status();
            if let Ok(s) = status {
                if s.success() { println!("✅ Service stopped."); }
            }
        }
        ServiceAction::Restart => {
            println!("🔄 Restarting background service...");
            let status = std::process::Command::new("systemctl")
                .args(["--user", "restart", "attack-shark.service"])
                .status();
            if let Ok(s) = status {
                if s.success() { println!("✅ Service restarted!"); }
            }
        }
        ServiceAction::Enable => {
            println!("⚡ Enabling background service on boot...");
            let status = std::process::Command::new("systemctl")
                .args(["--user", "enable", "--now", "attack-shark.service"])
                .status();
            if let Ok(s) = status {
                if s.success() { println!("✅ Service enabled and started!"); }
            }
        }
        ServiceAction::Disable => {
            println!("🛑 Disabling background service on boot...");
            let status = std::process::Command::new("systemctl")
                .args(["--user", "disable", "--now", "attack-shark.service"])
                .status();
            if let Ok(s) = status {
                if s.success() { println!("✅ Service disabled and stopped."); }
            }
        }
        ServiceAction::Logs => {
            println!("📋 Viewing recent service logs (Ctrl+C to exit)...\n");
            let _ = std::process::Command::new("journalctl")
                .args(["--user", "-u", "attack-shark.service", "-n", "30", "--no-pager"])
                .status();
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            println!("🔍 Searching for Attack Shark gaming mouse...");
            let mut dev = match MouseDevice::find_and_open() {
                Ok(dev) => dev,
                Err(err) => {
                    eprintln!("❌ Error: {}", err);
                    std::process::exit(1);
                }
            };

            println!("✅ Mouse found!");
            println!("   • Device Node : {:?}", dev.path);
            println!(
                "   • Mode        : {}",
                if dev.is_wireless {
                    "2.4G Wireless"
                } else {
                    "USB Wired"
                }
            );

            // Read Polling Rate
            print!("   • Polling Rate: ");
            match dev.get_polling_rate() {
                Ok(r) => {
                    let rate_val = r as u32;
                    set_cached_rate(rate_val);
                    println!("{} Hz", rate_val);
                }
                Err(_) => {
                    if let Some(r) = get_cached_rate() {
                        println!("{} Hz [Cached]", r);
                    } else {
                        println!("Unable to query (Mouse asleep)");
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(60));

            // Read DPI
            print!("   • Active DPI  : ");
            match dev.get_dpi_config() {
                Ok(cfg) => {
                    let live_stage = get_live_stage();
                    let (active_stage, active_dpi, is_live) = if let Some(ls) = live_stage {
                        let dpi = cfg.stages[(ls - 1) as usize].dpi;
                        (ls, dpi, true)
                    } else {
                        (cfg.active_stage_idx + 1, cfg.get_active_dpi(), false)
                    };
                    println!(
                        "{} DPI (Stage {}){}",
                        active_dpi,
                        active_stage,
                        if is_live { " [Live Synced via Button]" } else { " [Flash Default]" }
                    );
                }
                Err(_) => println!("Unable to query"),
            }

            // Read Battery
            print!("   • Battery     : ");
            match dev.get_battery() {
                Ok((pct, status_code)) => {
                    let chg_str = format_battery_status(status_code);
                    set_cached_battery(pct);
                    println!("{}% [{}]", pct, chg_str);
                }
                Err(_) => {
                    if let Some(pct) = get_cached_battery() {
                        println!("{}% [Live Cached]", pct);
                    } else {
                        println!("Unable to query");
                    }
                }
            }

            // Daemon status
            let is_daemon_active = std::process::Command::new("systemctl")
                .args(["--user", "is-active", "attack-shark.service"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
                .unwrap_or(false);

            println!(
                "   • Daemon      : {}",
                if is_daemon_active {
                    "🟢 Running (systemd background service)"
                } else {
                    "⚪ Inactive (Run 'attack-shark-ctl install' to enable)"
                }
            );
        }

        Commands::Install => handle_install(),

        Commands::Uninstall => handle_uninstall(),

        Commands::Service { action } => handle_service(action),

        Commands::Battery => {
            println!("📡 Listening for mouse battery broadcast (takes ~1-2s)...");
            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            match dev.get_battery() {
                Ok((pct, status_code)) => {
                    set_cached_battery(pct);
                    let chg_str = format_battery_status(status_code);
                    println!("🔋 Mouse Battery: {}% [{}]", pct, chg_str);
                }
                Err(e) => {
                    if let Some(pct) = get_cached_battery() {
                        println!("🔋 Mouse Battery: {}% [Cached from Monitor]", pct);
                    } else {
                        eprintln!("❌ Failed to query battery: {}", e);
                        eprintln!("   Tip: Move the mouse slightly or start the monitor: attack-shark-ctl monitor");
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Monitor { seconds } => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║      Attack Shark — Live Hardware Monitor Daemon          ║");
            println!("╠═══════════════════════════════════════════════════════════╣");
            println!("║  • Notifications : Desktop OSD Enabled (via notify-send)  ║");
            if let Some(sec) = seconds {
                println!("║  • Mode          : Running for {:>2} seconds                ║", sec);
            } else {
                println!("║  • Mode          : Continuous Daemon (Auto-reconnects)    ║");
            }
            println!("╚═══════════════════════════════════════════════════════════╝\n");
            println!("👉 Click your physical DPI button to see live desktop toasts!\n");

            let start_time = std::time::Instant::now();

            'reconnect: loop {
                if let Some(sec) = seconds {
                    if start_time.elapsed().as_secs() >= sec {
                        println!("\n⏱️ Monitor duration ({} seconds) completed.", sec);
                        break 'reconnect;
                    }
                }

                let mut dev = match MouseDevice::find_and_open() {
                    Ok(d) => d,
                    Err(e) => {
                        if seconds.is_some() {
                            eprintln!("❌ Error: {}", e);
                            std::process::exit(1);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(1500));
                        continue 'reconnect;
                    }
                };

                let cfg = match dev.get_dpi_config() {
                    Ok(c) => c,
                    Err(_) => {
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                        continue 'reconnect;
                    }
                };
                let stage_dpis: Vec<u32> = cfg.stages.iter().map(|s| s.dpi).collect();

                println!("[{}] 🟢 Connected to mouse on {:?}", simple_time(), dev.path);

                let fd = dev.get_raw_fd();
                let mut pfd = libc::pollfd {
                    fd,
                    events: libc::POLLIN,
                    revents: 0,
                };

                loop {
                    if let Some(sec) = seconds {
                        if start_time.elapsed().as_secs() >= sec {
                            println!("\n⏱️ Monitor duration ({} seconds) completed.", sec);
                            break 'reconnect;
                        }
                    }

                    let poll_timeout_ms = 500;
                    let ret = unsafe { libc::poll(&mut pfd, 1, poll_timeout_ms) };

                    if ret > 0 {
                        if (pfd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL)) != 0 {
                            println!("[{}] ⚠️ Device disconnected. Waiting to reconnect...", simple_time());
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                            continue 'reconnect;
                        }

                        if (pfd.revents & libc::POLLIN) != 0 {
                            let mut buf = [0u8; 64];
                            match dev.read_report(&mut buf) {
                                Ok(n) if n >= 5 && buf[0] == 0x03 && buf[1] == 0x06 => {
                                    let now = simple_time();

                                    // DPI Stage Change Event (0x10 or 0x02)
                                    if buf[2] == 0x10 || buf[2] == 0x02 {
                                        let detected_stage = if (1..=6).contains(&buf[4]) {
                                            buf[4]
                                        } else if (1..=6).contains(&buf[3]) {
                                            buf[3]
                                        } else {
                                            0
                                        };

                                        if (1..=6).contains(&detected_stage) {
                                            set_live_stage(detected_stage);
                                            let dpi_val = stage_dpis
                                                .get((detected_stage - 1) as usize)
                                                .copied()
                                                .unwrap_or(0);

                                            println!(
                                                "[{}] 🎯 DPI Button Clicked! -> Stage {} ({} DPI)",
                                                now, detected_stage, dpi_val
                                            );

                                            send_desktop_notification(
                                                "Attack Shark",
                                                &format!("🎯 DPI Changed: {} DPI (Stage {})", dpi_val, detected_stage),
                                                "normal",
                                            );
                                        }
                                    }
                                    // Battery Update Event (0x40)
                                    else if buf[2] == 0x40 {
                                        let status_code = buf[3];
                                        let is_charging = status_code == 2 || status_code == 3;
                                        let pct = buf[4];
                                        set_cached_battery(pct);
                                        let chg_str = format_battery_status(status_code);

                                        println!("[{}] 🔋 Battery Update: {}% [{}]", now, pct, chg_str);

                                        if pct <= 20 && !is_charging {
                                            send_desktop_notification(
                                                "⚠️ Low Mouse Battery",
                                                &format!("Attack Shark battery is low: {}%!\nPlease connect USB charging cable.", pct),
                                                "critical",
                                            );
                                        }
                                    }
                                }
                                Ok(_) => {}
                                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                Err(_) => {
                                    println!("[{}] ⚠️ Connection lost. Waiting to reconnect...", simple_time());
                                    std::thread::sleep(std::time::Duration::from_millis(1000));
                                    continue 'reconnect;
                                }
                            }
                        }
                    } else if ret < 0 {
                        let err = std::io::Error::last_os_error();
                        if err.raw_os_error() != Some(libc::EINTR) {
                            println!("[{}] ⚠️ Poll error ({}). Reconnecting...", simple_time(), err);
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                            continue 'reconnect;
                        }
                    }
                }
            }
        }


        Commands::GetRate => {
            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            println!("📡 Querying mouse hardware polling rate...");
            match dev.get_polling_rate() {
                Ok(rate) => {
                    let rate_val = rate as u32;
                    set_cached_rate(rate_val);
                    println!("🎯 Current Hardware Polling Rate: {} Hz", rate_val);
                }
                Err(e) => {
                    if let Some(r) = get_cached_rate() {
                        println!("🎯 Current Hardware Polling Rate: {} Hz [Cached]", r);
                    } else {
                        eprintln!("❌ Query failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::SetRate { rate } => {
            let target_rate = match PollingRate::from_hz(rate) {
                Some(r) => r,
                None => {
                    eprintln!("❌ Invalid polling rate: {} Hz.", rate);
                    eprintln!("   Supported rates: 125, 250, 500, 1000");
                    std::process::exit(1);
                }
            };

            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            println!(
                "⚡ Sending hardware command: Set Polling Rate to {} Hz...",
                rate
            );

            match dev.set_polling_rate(target_rate) {
                Ok(()) => {
                    set_cached_rate(rate);
                    println!("✅ Successfully programmed {} Hz to mouse hardware!", rate);
                    println!("   The setting is saved in the mouse's internal flash memory.");
                }
                Err(e) => {
                    eprintln!("❌ Failed to write to hardware: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::GetDpi => {
            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            println!("📡 Reading DPI configuration from mouse internal flash...");
            match dev.get_dpi_config() {
                Ok(cfg) => {
                    let live_stage = get_live_stage();
                    let active_idx = if let Some(ls) = live_stage {
                        ls - 1
                    } else {
                        cfg.active_stage_idx
                    };

                    println!("\n╔═══════════════════════════════════╗");
                    println!("║        Attack Shark Presets       ║");
                    println!("╠════════╦══════════════╦═══════════╣");
                    println!("║ Preset ║     DPI      ║  Status   ║");
                    println!("╠════════╬══════════════╬═══════════╣");

                    for stage in &cfg.stages {
                        let is_active = (stage.stage_num - 1) == active_idx;
                        let active_str = if is_active {
                            "ACTIVE ▶"
                        } else if stage.enabled {
                            "Enabled"
                        } else {
                            "Disabled"
                        };

                        println!(
                            "║   {}    ║  {:>5} DPI   ║ {:<9} ║",
                            stage.stage_num, stage.dpi, active_str
                        );
                    }
                    println!("╚════════╩══════════════╩═══════════╝");
                    let active_dpi = cfg.stages[active_idx as usize].dpi;
                    let source_str = if live_stage.is_some() {
                        " [Live Synced via Button]"
                    } else {
                        " [Flash Default]"
                    };
                    println!(
                        "\n🎯 Active Preset: Preset {} ({} DPI){}",
                        active_idx + 1,
                        active_dpi,
                        source_str
                    );
                }
                Err(e) => {
                    eprintln!("❌ Failed to read DPI: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::SetDpi { dpi, all } => {
            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            let mut cfg = match dev.get_dpi_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to read current DPI configuration: {}", e);
                    std::process::exit(1);
                }
            };

            if all {
                for s in 1..=6 {
                    if let Err(e) = cfg.set_stage_dpi(s, dpi) {
                        eprintln!("❌ Invalid parameter: {}", e);
                        std::process::exit(1);
                    }
                }
                println!(
                    "⚡ Locking ALL 6 stages to {} DPI in mouse internal flash...",
                    dpi
                );
                match dev.set_dpi_config(&cfg) {
                    Ok(()) => {
                        println!("✅ Successfully programmed ALL 6 stages to {} DPI!", dpi);
                        println!("   The physical DPI button will now stay locked at {} DPI.", dpi);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to write to mouse hardware: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                let active_stage = cfg.active_stage_idx + 1;
                if let Err(e) = cfg.set_stage_dpi(active_stage, dpi) {
                    eprintln!("❌ Invalid parameter: {}", e);
                    std::process::exit(1);
                }

                println!(
                    "⚡ Writing {} DPI to active Stage {} in mouse internal flash...",
                    dpi, active_stage
                );

                match dev.set_dpi_config(&cfg) {
                    Ok(()) => {
                        println!("✅ Successfully updated Stage {} to {} DPI!", active_stage, dpi);
                        println!("   Setting is permanently saved in mouse flash memory.");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to write to mouse hardware: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::SetAllDpi { dpi } => {
            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            let mut cfg = match dev.get_dpi_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to read current DPI configuration: {}", e);
                    std::process::exit(1);
                }
            };

            for s in 1..=6 {
                if let Err(e) = cfg.set_stage_dpi(s, dpi) {
                    eprintln!("❌ Invalid parameter: {}", e);
                    std::process::exit(1);
                }
            }
            println!(
                "⚡ Locking ALL 6 stages to {} DPI in mouse internal flash...",
                dpi
            );
            match dev.set_dpi_config(&cfg) {
                Ok(()) => {
                    println!("✅ Successfully programmed ALL 6 stages to {} DPI!", dpi);
                    println!("   The physical DPI button will now stay locked at {} DPI.", dpi);
                }
                Err(e) => {
                    eprintln!("❌ Failed to write to mouse hardware: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::SetStage { stage } => {
            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            let mut cfg = match dev.get_dpi_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to read current DPI configuration: {}", e);
                    std::process::exit(1);
                }
            };

            if let Err(e) = cfg.set_active_stage(stage) {
                eprintln!("❌ Invalid stage: {}", e);
                std::process::exit(1);
            }

            let new_dpi = cfg.stages[(stage - 1) as usize].dpi;
            println!(
                "⚡ Switching mouse active stage to Stage {} ({} DPI)...",
                stage, new_dpi
            );

            match dev.set_dpi_config(&cfg) {
                Ok(()) => {
                    set_live_stage(stage);
                    println!("✅ Successfully switched active DPI to Stage {} ({} DPI)!", stage, new_dpi);
                }
                Err(e) => {
                    eprintln!("❌ Failed to write to mouse hardware: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::SetStageDpi { stage_or_dpi, dpi } => {
            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            let mut cfg = match dev.get_dpi_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to read current DPI configuration: {}", e);
                    std::process::exit(1);
                }
            };

            let (stage, target_dpi) = match (stage_or_dpi, dpi) {
                (val, None) => {
                    if val > 6 {
                        // User typed `set-stage-dpi 100` or `set-stage-dpi 1600`
                        let active = cfg.active_stage_idx + 1;
                        (active, val)
                    } else {
                        eprintln!(
                            "❌ Ambiguous argument: '{}'.\n   Did you mean to set Stage {} DPI? Use: attack-shark-ctl set-stage-dpi {} <DPI>",
                            val, val, val
                        );
                        std::process::exit(1);
                    }
                }
                (s, Some(d)) => {
                    if s < 1 || s > 6 {
                        eprintln!("❌ Invalid stage number: {}. Stage must be between 1 and 6.", s);
                        std::process::exit(1);
                    }
                    (s as u8, d)
                }
            };

            if let Err(e) = cfg.set_stage_dpi(stage, target_dpi) {
                eprintln!("❌ Invalid parameter: {}", e);
                std::process::exit(1);
            }

            println!(
                "⚡ Programming Stage {} to {} DPI in mouse internal flash...",
                stage, target_dpi
            );

            match dev.set_dpi_config(&cfg) {
                Ok(()) => {
                    println!("✅ Successfully programmed Stage {} to {} DPI!", stage, target_dpi);
                }
                Err(e) => {
                    eprintln!("❌ Failed to write to mouse hardware: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::SetLadder { s1, s2, s3, s4, s5, s6 } => {
            let mut dev = match MouseDevice::find_and_open() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            };

            let mut cfg = match dev.get_dpi_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to read current DPI configuration: {}", e);
                    std::process::exit(1);
                }
            };

            let ladder = [s1, s2, s3, s4, s5, s6];
            for (idx, &dpi) in ladder.iter().enumerate() {
                if let Err(e) = cfg.set_stage_dpi((idx + 1) as u8, dpi) {
                    eprintln!("❌ Invalid parameter for Stage {}: {}", idx + 1, e);
                    std::process::exit(1);
                }
            }

            println!("⚡ Programming 6-stage DPI ladder to mouse internal flash...");
            for (idx, &dpi) in ladder.iter().enumerate() {
                println!("   • Stage {}: {} DPI", idx + 1, dpi);
            }

            match dev.set_dpi_config(&cfg) {
                Ok(()) => {
                    set_live_stage(cfg.active_stage_idx + 1);
                    println!("✅ Successfully programmed all 6 stages!");
                    println!("   Now pressing the physical DPI button will cycle through these exact speeds.");
                }
                Err(e) => {
                    eprintln!("❌ Failed to write to mouse hardware: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}


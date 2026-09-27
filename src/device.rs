use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::protocol::{
    build_query_packet, build_set_polling_rate_packet, DpiConfig, PollingRate,
    PRODUCT_ID_WIRED, PRODUCT_ID_WIRELESS, REPORT_ID_CONFIG, REPORT_ID_DPI, REPORT_ID_QUERY,
    VENDOR_ID,
};

// Linux ioctl macro definitions for HIDIOCSFEATURE and HIDIOCGFEATURE
// _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x06, len)
const fn hidiocsfeature(len: usize) -> libc::c_ulong {
    let dir = 3 as libc::c_ulong; // _IOC_READ | _IOC_WRITE
    let typ = b'H' as libc::c_ulong;
    let nr = 0x06 as libc::c_ulong;
    let size = len as libc::c_ulong;
    (dir << 30) | (size << 16) | (typ << 8) | nr
}

const fn hidiocgfeature(len: usize) -> libc::c_ulong {
    let dir = 3 as libc::c_ulong; // _IOC_READ | _IOC_WRITE
    let typ = b'H' as libc::c_ulong;
    let nr = 0x07 as libc::c_ulong;
    let size = len as libc::c_ulong;
    (dir << 30) | (size << 16) | (typ << 8) | nr
}

#[derive(Debug)]
pub struct MouseDevice {
    pub path: PathBuf,
    pub is_wireless: bool,
    file: File,
}

impl MouseDevice {
    /// Discovers and opens the Attack Shark X8 configuration hidraw device.
    /// If both the USB cable and 2.4G dongle are plugged in, wired is automatically prioritized.
    pub fn find_and_open() -> Result<Self, String> {
        let entries = fs::read_dir("/sys/class/hidraw")
            .map_err(|e| format!("Failed to read /sys/class/hidraw: {}", e))?;

        let mut candidates: Vec<(PathBuf, bool)> = Vec::new();

        for entry in entries.flatten() {
            let hidraw_name = entry.file_name();
            let hidraw_str = hidraw_name.to_string_lossy();
            let uevent_path = entry.path().join("device/uevent");
            let rdesc_path = entry.path().join("device/report_descriptor");

            if let Ok(uevent_content) = fs::read_to_string(&uevent_path) {
                let vid_str = format!("{:04X}", VENDOR_ID);
                let pid_wired = format!("{:04X}", PRODUCT_ID_WIRED);
                let pid_wireless = format!("{:04X}", PRODUCT_ID_WIRELESS);

                let is_our_vendor = uevent_content.to_uppercase().contains(&vid_str);
                let is_wired = uevent_content.to_uppercase().contains(&pid_wired);
                let is_wireless = uevent_content.to_uppercase().contains(&pid_wireless);

                if is_our_vendor && (is_wired || is_wireless) {
                    if let Ok(rdesc_bytes) = fs::read(&rdesc_path) {
                        let has_config_report = rdesc_bytes
                            .windows(2)
                            .any(|w| w[0] == 0x85 && w[1] == REPORT_ID_CONFIG);

                        if has_config_report {
                            let dev_node = Path::new("/dev").join(&*hidraw_str);
                            candidates.push((dev_node, is_wireless));
                        }
                    }
                }
            }
        }

        // Sort candidates: wired (is_wireless == false) comes first
        candidates.sort_by_key(|&(_, is_wireless)| is_wireless);

        if let Some((dev_node, is_wireless)) = candidates.into_iter().next() {
            return Self::open_device(dev_node, is_wireless);
        }

        Err("Attack Shark X8 Plus mouse was not found. Please make sure the USB cable or 2.4G wireless dongle is plugged in.".to_string())
    }

    fn open_device(path: PathBuf, is_wireless: bool) -> Result<Self, String> {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path)
        {
            Ok(file) => Ok(Self {
                path,
                is_wireless,
                file,
            }),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                Err(format!(
                    "Permission denied opening {:?}.\n\n\
                    To grant access without sudo, create the udev rule:\n  \
                    echo 'KERNEL==\"hidraw*\", ATTRS{{idVendor}}==\"1d57\", MODE=\"0666\"' | sudo tee /etc/udev/rules.d/99-attackshark.rules\n  \
                    sudo udevadm control --reload-rules && sudo udevadm trigger\n\n\
                    Or run this command with 'sudo'.",
                    path
                ))
            }
            Err(e) => Err(format!("Failed to open {:?}: {}", path, e)),
        }
    }

    /// Dispatches a query on Report ID 0xA0 and reads the resulting Feature Report.
    pub fn query_feature<const N: usize>(&mut self, report_id: u8) -> Result<[u8; N], String> {
        let q = build_query_packet(report_id, N as u8);
        let fd: RawFd = self.file.as_raw_fd();
        let set_ioc_8 = hidiocsfeature(q.len());
        let get_ioc_8 = hidiocgfeature(8);
        let get_ioc_n = hidiocgfeature(N);

        let mut ready = false;
        for _ in 0..25 {
            // Send query
            let ret = unsafe { libc::ioctl(fd, set_ioc_8, q.as_ptr()) };
            if ret < 0 {
                let err = std::io::Error::last_os_error();
                return Err(format!("Failed to send query command (ioctl error): {}", err));
            }

            thread::sleep(Duration::from_millis(60));

            // Check ack on Report ID 0xA0
            let mut ack = [0u8; 8];
            ack[0] = REPORT_ID_QUERY;
            let ret = unsafe { libc::ioctl(fd, get_ioc_8, ack.as_mut_ptr()) };
            if ret >= 0 && ack[1] == 1 {
                ready = true;
                break;
            }

            thread::sleep(Duration::from_millis(60));
        }

        if !ready {
            return Err("Mouse is asleep or not responding. Please move/click the mouse to wake up the 2.4G link and try again.".to_string());
        }

        thread::sleep(Duration::from_millis(50));

        let mut buf = [0u8; N];
        let mut read_ok = false;
        for _ in 0..15 {
            buf = [0u8; N];
            buf[0] = report_id;
            let ret = unsafe { libc::ioctl(fd, get_ioc_n, buf.as_mut_ptr()) };
            if ret >= 0 && buf[0] == report_id {
                read_ok = true;
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        if !read_ok {
            return Err(format!(
                "Failed to read hardware feature report 0x{:02X} (Got 0x{:02X})",
                report_id, buf[0]
            ));
        }

        Ok(buf)
    }

    /// Sets the hardware polling rate directly on the mouse MCU.
    pub fn set_polling_rate(&mut self, rate: PollingRate) -> Result<(), String> {
        let packet = build_set_polling_rate_packet(rate);
        let fd: RawFd = self.file.as_raw_fd();
        let ioctl_num = hidiocsfeature(packet.len());

        let ret = unsafe { libc::ioctl(fd, ioctl_num, packet.as_ptr()) };

        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!(
                "Hardware write failed (ioctl error {}): {}",
                err.raw_os_error().unwrap_or(0),
                err
            ));
        }

        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    /// Queries the current hardware polling rate from the mouse MCU.
    pub fn get_polling_rate(&mut self) -> Result<PollingRate, String> {
        let buf: [u8; 9] = match self.query_feature(REPORT_ID_CONFIG) {
            Ok(b) => b,
            Err(_) => {
                // If it timed out because mouse was waking up, retry once
                thread::sleep(Duration::from_millis(100));
                self.query_feature(REPORT_ID_CONFIG)?
            }
        };
        let rate_code = buf[3];
        PollingRate::from_code(rate_code).ok_or_else(|| {
            format!(
                "Unknown hardware polling rate code returned: 0x{:02X} (Raw: {:?})",
                rate_code, buf
            )
        })
    }

    /// Queries the full DPI configuration directly from mouse internal flash.
    pub fn get_dpi_config(&mut self) -> Result<DpiConfig, String> {
        let buf: [u8; 56] = match self.query_feature(REPORT_ID_DPI) {
            Ok(b) => b,
            Err(_) => {
                thread::sleep(Duration::from_millis(100));
                self.query_feature(REPORT_ID_DPI)?
            }
        };
        DpiConfig::from_raw(buf)
    }

    /// Writes the full DPI configuration directly to mouse internal flash.
    pub fn set_dpi_config(&mut self, config: &DpiConfig) -> Result<(), String> {
        let fd: RawFd = self.file.as_raw_fd();
        let ioctl_num = hidiocsfeature(config.raw.len());

        let ret = unsafe { libc::ioctl(fd, ioctl_num, config.raw.as_ptr()) };

        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!(
                "Hardware write failed (ioctl error {}): {}",
                err.raw_os_error().unwrap_or(0),
                err
            ));
        }

        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    /// Reads an HID input report from the open device node (non-blocking).
    pub fn read_report(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read;
        self.file.read(buf)
    }

    /// Returns the raw file descriptor for polling/event-loop integration.
    pub fn get_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }

    /// Listens for the mouse hardware battery broadcast report (broadcasts every ~2 seconds).
    pub fn get_battery(&mut self) -> Result<(u8, u8), String> {
        let fd = self.get_raw_fd();
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };

        let start = std::time::Instant::now();
        while start.elapsed().as_millis() < 2800 {
            let ret = unsafe { libc::poll(&mut pfd, 1, 150) };
            if ret > 0 && (pfd.revents & libc::POLLIN) != 0 {
                let mut buf = [0u8; 64];
                if let Ok(n) = self.read_report(&mut buf) {
                    if n >= 5 && buf[0] == 0x03 && buf[1] == 0x06 && buf[2] == 0x40 {
                        let status_code = buf[3];
                        let percent = buf[4];
                        return Ok((percent, status_code));
                    }
                }
            }
        }

        Err("Mouse did not emit battery broadcast within 2.8s".to_string())
    }
}


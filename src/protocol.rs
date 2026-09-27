pub const VENDOR_ID: u16 = 0x1d57;
pub const PRODUCT_ID_WIRELESS: u16 = 0xfa60;
#[allow(dead_code)]
pub const PRODUCT_ID_WIRED: u16 = 0x2124;

/// All 18 Wired USB Product IDs extracted directly from AttackShark.exe firmware tables (MS_1 to MS_18)
pub const SUPPORTED_WIRED_PIDS: &[u16] = &[
    0x2124, // MS_12: Attack Shark X8 Plus
    0x2055, // MS_1
    0x201B, // MS_2
    0xFA61, // MS_3
    0xFA55, // MS_4
    0x201C, // MS_5
    0x2111, // MS_6
    0x2125, // MS_7
    0x2120, // MS_8
    0x2126, // MS_9
    0x2122, // MS_10
    0x212C, // MS_11
    0x2155, // MS_13
    0x2224, // MS_14
    0x215A, // MS_15
    0x211D, // MS_16
    0x211F, // MS_17
    0x2121, // MS_18
];

#[allow(dead_code)]
pub fn is_supported_wired(pid: u16) -> bool {
    SUPPORTED_WIRED_PIDS.contains(&pid)
}

pub const REPORT_ID_DPI: u8 = 0x04;
pub const REPORT_ID_CONFIG: u8 = 0x06;
pub const REPORT_ID_QUERY: u8 = 0xa0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollingRate {
    Hz125 = 125,
    Hz250 = 250,
    Hz500 = 500,
    Hz1000 = 1000,
}

impl PollingRate {
    pub fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            125 => Some(PollingRate::Hz125),
            250 => Some(PollingRate::Hz250),
            500 => Some(PollingRate::Hz500),
            1000 => Some(PollingRate::Hz1000),
            _ => None,
        }
    }

    pub fn to_code(self) -> u8 {
        match self {
            PollingRate::Hz1000 => 0x01,
            PollingRate::Hz500 => 0x02,
            PollingRate::Hz250 => 0x04,
            PollingRate::Hz125 => 0x08,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0x01 => Some(PollingRate::Hz1000),
            0x02 => Some(PollingRate::Hz500),
            0x04 => Some(PollingRate::Hz250),
            0x08 => Some(PollingRate::Hz125),
            _ => None,
        }
    }
}

/// Builds the 9-byte hardware feature report to change the polling rate.
pub fn build_set_polling_rate_packet(rate: PollingRate) -> [u8; 9] {
    let code = rate.to_code();
    let checksum = !code; // Bitwise NOT checksum verified from AttackShark.exe at 0x43a222
    [
        REPORT_ID_CONFIG, // Byte 0: 0x06 (Report ID)
        0x09,             // Byte 1: Subcommand (Polling rate)
        0x01,             // Byte 2: Parameter type
        code,             // Byte 3: Polling rate code
        checksum,         // Byte 4: Checksum (~code)
        0x00,             // Byte 5
        0x00,             // Byte 6
        0x00,             // Byte 7
        0x00,             // Byte 8
    ]
}

/// Builds the 8-byte hardware feature report to dispatch a query to the mouse MCU.
pub fn build_query_packet(report_id: u8, len: u8) -> [u8; 8] {
    [
        REPORT_ID_QUERY, // 0xA0: Command dispatcher
        report_id,       // Report ID to query (e.g. 0x04 for DPI, 0x06 for Polling Rate)
        len,             // Expected report length
        0x00,            // Reserved
        0x01,            // Query opcode
        0x00,
        0x00,
        0x00,
    ]
}

#[derive(Debug, Clone)]
pub struct DpiStage {
    pub stage_num: u8, // 1-based (1..=6)
    pub dpi: u32,      // DPI value (e.g. 1200)
    pub enabled: bool,
    #[allow(dead_code)]
    pub color: (u8, u8, u8), // Raw color bytes in protocol
}

#[derive(Debug, Clone)]
pub struct DpiConfig {
    pub raw: [u8; 56],
    pub active_stage_idx: u8, // 0-based index (0..5)
    pub stages: Vec<DpiStage>,
}

impl DpiConfig {
    pub fn from_raw(raw: [u8; 56]) -> Result<Self, String> {
        if raw[0] != REPORT_ID_DPI || raw[1] != 0x38 {
            return Err(format!(
                "Invalid DPI report header: 0x{:02X} 0x{:02X} (Expected 0x04 0x38)",
                raw[0], raw[1]
            ));
        }

        let active_stage_idx = raw[3] >> 4;
        let enabled_mask = raw[5];
        let mut stages = Vec::new();

        for i in 0..6 {
            let low = raw[8 + i] as u32;
            let high = raw[16 + i] as u32;
            let raw_val = (high << 8) | low;
            let dpi = (raw_val + 1) * 50;
            let enabled = (enabled_mask & (1 << i)) != 0;

            let r = raw[25 + i * 3];
            let g = raw[26 + i * 3];
            let b = raw[27 + i * 3];

            stages.push(DpiStage {
                stage_num: (i + 1) as u8,
                dpi,
                enabled,
                color: (r, g, b),
            });
        }

        Ok(Self {
            raw,
            active_stage_idx,
            stages,
        })
    }

    /// Returns the currently active DPI value.
    pub fn get_active_dpi(&self) -> u32 {
        if (self.active_stage_idx as usize) < self.stages.len() {
            self.stages[self.active_stage_idx as usize].dpi
        } else {
            1600
        }
    }

    /// Sets the active DPI stage (1-based: 1..=6).
    pub fn set_active_stage(&mut self, stage_1based: u8) -> Result<(), String> {
        if stage_1based < 1 || stage_1based > self.stages.len() as u8 {
            return Err(format!(
                "Invalid stage: {}. Must be between 1 and {}.",
                stage_1based,
                self.stages.len()
            ));
        }
        let idx = stage_1based - 1;
        self.active_stage_idx = idx;
        // Upper nibble is stage index, lower nibble preserves flags
        self.raw[3] = (idx << 4) | (self.raw[3] & 0x0F);
        self.update_checksum();
        Ok(())
    }

    /// Sets the DPI value for a specific stage (1-based: 1..=6).
    pub fn set_stage_dpi(&mut self, stage_1based: u8, dpi: u32) -> Result<(), String> {
        if stage_1based < 1 || stage_1based > self.stages.len() as u8 {
            return Err(format!(
                "Invalid stage: {}. Must be between 1 and {}.",
                stage_1based,
                self.stages.len()
            ));
        }
        if dpi < 50 || dpi > 26000 || dpi % 50 != 0 {
            return Err(format!(
                "Invalid DPI: {}. DPI must be between 50 and 26000, and a multiple of 50 (e.g. 400, 800, 1200, 1600, 2100).",
                dpi
            ));
        }

        let idx = (stage_1based - 1) as usize;
        let raw_val = (dpi / 50).saturating_sub(1);
        self.raw[8 + idx] = (raw_val & 0xFF) as u8;
        self.raw[16 + idx] = ((raw_val >> 8) & 0xFF) as u8;
        self.stages[idx].dpi = dpi;

        // Ensure all 6 stages remain enabled and cyclable by the physical DPI button
        self.raw[5] = 0x3F;
        self.raw[24] = 0x06;

        self.update_checksum();
        Ok(())
    }

    /// Recalculates the 16-bit big-endian checksum over bytes 3..49.
    pub fn update_checksum(&mut self) {
        let mut sum: u16 = 0;
        for &b in &self.raw[3..50] {
            sum = sum.wrapping_add(b as u16);
        }
        self.raw[50] = (sum >> 8) as u8;
        self.raw[51] = (sum & 0xFF) as u8;
    }
}

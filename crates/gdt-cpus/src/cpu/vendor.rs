/// The CPU's manufacturer.
///
/// All vendors the detectors actually emit are named variants, which keeps the
/// enum `Copy` (the old `Other(String)` payload was the only allocation in the
/// data model and the raw vendor string is recoverable from `model_name` /
/// `/proc/cpuinfo` anyway).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Vendor {
    /// Intel Corporation.
    Intel,
    /// Advanced Micro Devices.
    Amd,
    /// ARM reference designs (implementer 0x41) when no more specific vendor applies.
    Arm,
    /// Apple Silicon (M-series, implementer 0x61).
    Apple,
    /// Qualcomm (implementer 0x51 - Snapdragon, Oryon).
    Qualcomm,
    /// Broadcom (implementer 0x42).
    Broadcom,
    /// NVIDIA (implementer 0x4e).
    Nvidia,
    /// Marvell / Cavium (implementers 0x56 / 0x43).
    Marvell,
    /// A vendor not listed above.
    Other,
    /// The vendor could not be determined.
    Unknown,
}

impl std::fmt::Display for Vendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vendor::Intel => write!(f, "Intel"),
            Vendor::Amd => write!(f, "AMD"),
            Vendor::Arm => write!(f, "ARM"),
            Vendor::Apple => write!(f, "Apple"),
            Vendor::Qualcomm => write!(f, "Qualcomm"),
            Vendor::Broadcom => write!(f, "Broadcom"),
            Vendor::Nvidia => write!(f, "NVIDIA"),
            Vendor::Marvell => write!(f, "Marvell"),
            Vendor::Other => write!(f, "Other"),
            Vendor::Unknown => write!(f, "Unknown"),
        }
    }
}

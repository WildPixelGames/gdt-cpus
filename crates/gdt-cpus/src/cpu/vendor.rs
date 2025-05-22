/// Represents the manufacturer or vendor of a CPU.
///
/// Identifying the CPU vendor is often the first step in understanding its
/// architecture, features, and capabilities. This enum provides common
/// CPU vendors and a way to represent others.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Vendor {
    /// Intel Corporation.
    ///
    /// One of the largest manufacturers of x86 architecture CPUs, known for Core, Xeon, and Atom series.
    Intel,
    /// Advanced Micro Devices, Inc.
    ///
    /// A major manufacturer of x86 architecture CPUs, known for Ryzen, EPYC, and Threadripper series.
    Amd,
    /// Arm Holdings.
    ///
    /// Designer of the ARM architecture, which is licensed by many manufacturers
    /// for a wide range of devices, from mobile phones to servers and embedded systems.
    /// This variant typically refers to CPUs based on ARM's reference designs or
    /// when a more specific vendor (like Apple for M-series) is not identified.
    Arm,
    /// Apple Inc.
    ///
    /// Specifically for Apple Silicon CPUs (e.g., M1, M2, M3 series) based on the ARM architecture,
    /// designed by Apple for their Mac computers and other devices.
    Apple,
    /// The CPU vendor could not be determined or is not recognized.
    Unknown,
    /// A CPU vendor not explicitly listed in the other variants.
    ///
    /// The `String` field contains the name of the vendor as identified by the system,
    /// if available.
    Other(String),
}

impl std::fmt::Display for Vendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vendor::Intel => write!(f, "Intel"),
            Vendor::Amd => write!(f, "AMD"),
            Vendor::Arm => write!(f, "ARM"),
            Vendor::Apple => write!(f, "Apple"),
            Vendor::Unknown => write!(f, "Unknown"),
            Vendor::Other(name) => write!(f, "{}", name),
        }
    }
}

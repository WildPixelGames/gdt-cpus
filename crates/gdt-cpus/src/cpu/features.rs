use bitflags::bitflags;

#[cfg(target_arch = "x86_64")]
bitflags! {
    /// Represents a set of CPU features and instruction set extensions available on an x86_64 architecture.
    ///
    /// These flags indicate hardware support for various SIMD (Single Instruction, Multiple Data)
    /// extensions, cryptographic accelerators, and other specialized instructions.
    /// Applications can query these flags to optimize performance by using the most advanced
    /// instruction sets supported by the CPU.
    ///
    /// The specific flags are derived from CPUID instruction results.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    pub struct CpuFeatures: u32 {
        /// MMX (MultiMedia eXtensions) support.
        const MMX       = 0x00000001;
        /// SSE (Streaming SIMD Extensions) support.
        const SSE       = 0x00000002;
        /// SSE2 (Streaming SIMD Extensions 2) support.
        const SSE2      = 0x00000004;
        /// SSE3 (Streaming SIMD Extensions 3) support.
        const SSE3      = 0x00000008;
        /// SSSE3 (Supplemental Streaming SIMD Extensions 3) support.
        const SSSE3     = 0x00000010;
        /// SSE4.1 (Streaming SIMD Extensions 4.1) support.
        const SSE4_1    = 0x00000020;
        /// SSE4.2 (Streaming SIMD Extensions 4.2) support.
        const SSE4_2    = 0x00000040;
        /// FMA3 (Fused Multiply-Add 3-operand) support.
        const FMA3      = 0x00000080;
        /// AVX (Advanced Vector Extensions) support.
        const AVX       = 0x00000100;
        /// AVX2 (Advanced Vector Extensions 2) support.
        const AVX2      = 0x00000200;
        /// AVX-512 Foundation support.
        const AVX512F   = 0x00000400;
        /// AVX-512 Byte and Word Instructions support.
        const AVX512BW  = 0x00000800;
        /// AVX-512 Conflict Detection Instructions support.
        const AVX512CD  = 0x00001000;
        /// AVX-512 Doubleword and Quadword Instructions support.
        const AVX512DQ  = 0x00002000;
        /// AVX-512 Vector Length Extensions support.
        const AVX512VL  = 0x00004000;
        /// AES (Advanced Encryption Standard) hardware acceleration support.
        const AES       = 0x00008000;
        /// SHA (Secure Hash Algorithm) hardware acceleration support.
        const SHA       = 0x00010000;
        /// CRC32 (Cyclic Redundancy Check) hardware acceleration support.
        const CRC32     = 0x00020000;
    }
}

#[cfg(target_arch = "aarch64")]
bitflags! {
    /// Represents a set of CPU features and instruction set extensions available on an AArch64 architecture.
    ///
    /// These flags indicate hardware support for various SIMD extensions (NEON, SVE),
    /// cryptographic accelerators, and other specialized instructions.
    /// Applications can query these flags to optimize performance.
    ///
    /// The specific flags are typically derived from system registers (e.g., ID_AA64ISAR0_EL1).
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    pub struct CpuFeatures: u32 {
        /// NEON (Advanced SIMD) support.
        const NEON      = 0x00000001;
        /// SVE (Scalable Vector Extension) support.
        const SVE       = 0x00000002;
        /// AES (Advanced Encryption Standard) hardware acceleration support.
        const AES       = 0x00000004;
        /// SHA (Secure Hash Algorithm) hardware acceleration support (SHA1, SHA256, SHA512).
        const SHA       = 0x00000008; // Might need to be more granular (SHA1, SHA2, SHA3, SHA512)
        /// CRC32 (Cyclic Redundancy Check) hardware acceleration support.
        const CRC32     = 0x00000010;
    }
}

// Fallback for other architectures to ensure CpuFeatures is always defined.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
bitflags! {
    /// Represents CPU features. On unsupported architectures, this will be empty.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    pub struct CpuFeatures: u32 {
        // No features defined for this architecture by default.
    }
}

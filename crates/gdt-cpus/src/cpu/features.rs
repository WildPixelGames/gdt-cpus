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
    ///
    /// Each flag's bit VALUE is a stable serialization format: a stored or
    /// transmitted feature set deserializes by bit. APPEND new flags at the end
    /// (next free bit); NEVER insert mid-list or renumber -- that would break
    /// deserialization of an existing set. The conformance test below pins it.
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
        /// POPCNT (population count) support.
        const POPCNT    = 0x00040000;
        /// BMI1 (Bit Manipulation Instructions 1: ANDN, BLSI, BLSR, TZCNT) support.
        const BMI1      = 0x00080000;
        /// BMI2 (Bit Manipulation Instructions 2: PDEP, PEXT, BZHI) support.
        const BMI2      = 0x00100000;
        /// F16C (half-precision <-> single-precision float conversion) support.
        const F16C      = 0x00200000;
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
        /// FP16 (half-precision floating-point arithmetic, FEAT_FP16) support.
        const FP16      = 0x00000020;
        /// DotProd (SDOT/UDOT int8 dot product, FEAT_DotProd) support.
        const DOTPROD   = 0x00000040;
        /// I8MM (int8 matrix multiply SMMLA, FEAT_I8MM) support.
        const I8MM      = 0x00000080;
        /// BF16 (bfloat16 BFDOT/BFMMLA, FEAT_BF16) support.
        const BF16      = 0x00000100;
        /// SVE2 (Scalable Vector Extension 2) support.
        const SVE2      = 0x00000200;
        /// LSE (Large System Extensions atomics, FEAT_LSE) support.
        const LSE       = 0x00000400;
        /// JSCVT (JavaScript conversion instruction, FEAT_JSCVT) support.
        const JSCVT     = 0x00000800;
        /// LRCPC (Load-Acquire RCpc instructions, FEAT_LRCPC) support.
        const LRCPC     = 0x00001000;
        /// PMULL (polynomial multiply, FEAT_PMULL) support.
        const PMULL     = 0x00002000;
        /// RDM (rounding doubling multiply-add, FEAT_RDM) support.
        const RDM       = 0x00004000;
        /// FHM (half-precision multiply-add to single precision, FEAT_FHM) support.
        const FHM       = 0x00008000;
        /// FCMA (floating-point complex multiply-add, FEAT_FCMA) support.
        const FCMA      = 0x00010000;
        /// LSE2 (Large System Extensions 2 atomics, FEAT_LSE2) support.
        const LSE2      = 0x00020000;
        /// LRCPC2 (immediate-offset RCpc load-acquire, FEAT_LRCPC2) support.
        const LRCPC2    = 0x00040000;
        /// SM3 cryptographic hash instructions (FEAT_SM3) support.
        const SM3       = 0x00080000;
        /// SM4 cryptographic cipher instructions (FEAT_SM4) support.
        const SM4       = 0x00100000;
        /// SVE AES instructions (FEAT_SVE_AES) support.
        const SVEAES    = 0x00200000;
        /// SVE PMULL instructions (FEAT_SVE_PMULL128) support.
        const SVEPMULL  = 0x00400000;
        /// SVE bit permutation instructions (FEAT_SVE_BitPerm) support.
        const SVEBITPERM = 0x00800000;
        /// SVE SHA3 instructions (FEAT_SVE_SHA3) support.
        const SVESHA3   = 0x01000000;
        /// SVE SM4 instructions (FEAT_SVE_SM4) support.
        const SVESM4    = 0x02000000;
        /// SVE I8MM instructions (FEAT_I8MM) support.
        const SVEI8MM   = 0x04000000;
        /// SVE BF16 instructions (FEAT_BF16) support.
        const SVEBF16   = 0x08000000;
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

// Bit-position stability: each flag's bit is a serialization format. Pinning the
// exact bit per flag fails CI if anyone inserts a flag mid-list or renumbers,
// which would break deserialization of a stored/transmitted feature set.
// APPEND-only; update this table only when adding a flag at the next free bit.
#[cfg(all(test, target_arch = "x86_64"))]
mod conformance_x86 {
    use super::CpuFeatures;
    #[test]
    fn bit_positions_are_stable() {
        let pins: &[(CpuFeatures, u32)] = &[
            (CpuFeatures::MMX, 0),
            (CpuFeatures::SSE, 1),
            (CpuFeatures::SSE2, 2),
            (CpuFeatures::SSE3, 3),
            (CpuFeatures::SSSE3, 4),
            (CpuFeatures::SSE4_1, 5),
            (CpuFeatures::SSE4_2, 6),
            (CpuFeatures::FMA3, 7),
            (CpuFeatures::AVX, 8),
            (CpuFeatures::AVX2, 9),
            (CpuFeatures::AVX512F, 10),
            (CpuFeatures::AVX512BW, 11),
            (CpuFeatures::AVX512CD, 12),
            (CpuFeatures::AVX512DQ, 13),
            (CpuFeatures::AVX512VL, 14),
            (CpuFeatures::AES, 15),
            (CpuFeatures::SHA, 16),
            (CpuFeatures::CRC32, 17),
            (CpuFeatures::POPCNT, 18),
            (CpuFeatures::BMI1, 19),
            (CpuFeatures::BMI2, 20),
            (CpuFeatures::F16C, 21),
        ];
        for &(flag, bit) in pins {
            assert_eq!(
                flag.bits(),
                1u32 << bit,
                "flag {flag:?} moved off bit {bit}"
            );
        }
    }
}

#[cfg(all(test, target_arch = "aarch64"))]
mod conformance_arm {
    use super::CpuFeatures;
    #[test]
    fn bit_positions_are_stable() {
        let pins: &[(CpuFeatures, u32)] = &[
            (CpuFeatures::NEON, 0),
            (CpuFeatures::SVE, 1),
            (CpuFeatures::AES, 2),
            (CpuFeatures::SHA, 3),
            (CpuFeatures::CRC32, 4),
            (CpuFeatures::FP16, 5),
            (CpuFeatures::DOTPROD, 6),
            (CpuFeatures::I8MM, 7),
            (CpuFeatures::BF16, 8),
            (CpuFeatures::SVE2, 9),
            (CpuFeatures::LSE, 10),
            (CpuFeatures::JSCVT, 11),
            (CpuFeatures::LRCPC, 12),
            (CpuFeatures::PMULL, 13),
            (CpuFeatures::RDM, 14),
            (CpuFeatures::FHM, 15),
            (CpuFeatures::FCMA, 16),
            (CpuFeatures::LSE2, 17),
            (CpuFeatures::LRCPC2, 18),
            (CpuFeatures::SM3, 19),
            (CpuFeatures::SM4, 20),
            (CpuFeatures::SVEAES, 21),
            (CpuFeatures::SVEPMULL, 22),
            (CpuFeatures::SVEBITPERM, 23),
            (CpuFeatures::SVESHA3, 24),
            (CpuFeatures::SVESM4, 25),
            (CpuFeatures::SVEI8MM, 26),
            (CpuFeatures::SVEBF16, 27),
        ];
        for &(flag, bit) in pins {
            assert_eq!(
                flag.bits(),
                1u32 << bit,
                "flag {flag:?} moved off bit {bit}"
            );
        }
    }
}

// Copyright 2016 Google Inc. BSD-style license.
// Rust atomics and Once retain native feature-cache publication and one-time initialization.
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Once,
};

// Source: src/core/SkCpu.h:15-44
pub mod SkX64 {
    pub const SSE1: u32 = 1 << 0;
    pub const SSE2: u32 = 1 << 1;
    pub const SSE3: u32 = 1 << 2;
    pub const SSSE3: u32 = 1 << 3;
    pub const SSE41: u32 = 1 << 4;
    pub const SSE42: u32 = 1 << 5;
    pub const AVX: u32 = 1 << 6;
    pub const F16C: u32 = 1 << 7;
    pub const FMA: u32 = 1 << 8;
    pub const AVX2: u32 = 1 << 9;
    pub const BMI1: u32 = 1 << 10;
    pub const BMI2: u32 = 1 << 11;
    pub const AVX512F: u32 = 1 << 12;
    pub const AVX512DQ: u32 = 1 << 13;
    pub const AVX512IFMA: u32 = 1 << 14;
    pub const AVX512PF: u32 = 1 << 15;
    pub const AVX512ER: u32 = 1 << 16;
    pub const AVX512CD: u32 = 1 << 17;
    pub const AVX512BW: u32 = 1 << 18;
    pub const AVX512VL: u32 = 1 << 19;
    pub const ERMS: u32 = 1 << 20;
    pub const ML3: u32 = AVX2 | BMI1 | BMI2 | F16C | FMA;
    pub const ML4: u32 = AVX512F | AVX512DQ | AVX512CD | AVX512BW | AVX512VL;
}

// Source: src/core/SkCpu.h:46-49
pub mod SkLoongArch {
    pub const SX: u32 = 1 << 0;
    pub const ASX: u32 = 1 << 1;
}

// Source: src/core/SkCpu.h:58-114
// Adaptation: Rust target_feature describes the selected compiler target. The
// highest native x64 level implies all lower ordered levels, just as the native
// SK_CPU_X64_LEVEL comparisons do. Feature-limit cfgs retain native precedence.
pub fn Supports(mask: u32) -> bool {
    use crate::include::private::SkFeatures as F;
    let mut features = gCachedFeatures.load(Ordering::Relaxed);
    if F::SK_CPU_X86.unwrap_or(0) != 0 {
        let level = F::SK_CPU_X64_LEVEL.unwrap_or(0);
        if level >= F::SK_CPU_X64_LEVEL_SSE1 {
            features |= SkX64::SSE1;
        }
        if level >= F::SK_CPU_X64_LEVEL_SSE2 {
            features |= SkX64::SSE2;
        }
        if level >= F::SK_CPU_X64_LEVEL_SSE3 {
            features |= SkX64::SSE3;
        }
        if level >= F::SK_CPU_X64_LEVEL_SSSE3 {
            features |= SkX64::SSSE3;
        }
        if level >= F::SK_CPU_X64_LEVEL_SSE41 {
            features |= SkX64::SSE41;
        }
        if level >= F::SK_CPU_X64_LEVEL_SSE42 {
            features |= SkX64::SSE42;
        }
        if level >= F::SK_CPU_X64_LEVEL_AVX {
            features |= SkX64::AVX;
        }
        if level >= F::SK_CPU_X64_LEVEL_AVX2 {
            features |= SkX64::AVX2;
        }
        if level >= F::SK_CPU_X64_LEVEL_ML4 {
            features |= SkX64::ML4;
        }
        if cfg!(SK_CPU_LIMIT_AVX) {
            features &= SkX64::SSE1
                | SkX64::SSE2
                | SkX64::SSE3
                | SkX64::SSSE3
                | SkX64::SSE41
                | SkX64::SSE42
                | SkX64::AVX;
        } else if cfg!(SK_CPU_LIMIT_SSE41) {
            features &= SkX64::SSE1 | SkX64::SSE2 | SkX64::SSE3 | SkX64::SSSE3 | SkX64::SSE41;
        } else if cfg!(SK_CPU_LIMIT_SSE2) {
            features &= SkX64::SSE1 | SkX64::SSE2;
        }
    } else if F::SK_CPU_LOONGARCH.unwrap_or(0) != 0 {
        let level = F::SK_CPU_LSX_LEVEL.unwrap_or(0);
        if level >= F::SK_CPU_LSX_LEVEL_LSX {
            features |= SkLoongArch::SX;
        }
        if level >= F::SK_CPU_LSX_LEVEL_LASX {
            features |= SkLoongArch::ASX;
        }
    }
    (features & mask) == mask
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    use super::SkX64;
    #[cfg(target_arch = "x86")]
    use std::arch::x86::__cpuid_count;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::__cpuid_count;

    // Source: src/core/SkCpu.cpp:40-40
    const kVendorLeaf: u32 = 0;
    // Source: src/core/SkCpu.cpp:42-42
    const kFMSFLeaf: u32 = 1;
    // Source: src/core/SkCpu.cpp:44-44
    const kFlagsLeaf: u32 = 7;
    // Source: src/core/SkCpu.cpp:45-45
    const kFlagsSubleaf: u32 = 0;
    // Rust's architecture intrinsic replaces the native compiler-specific ABI.
    fn cpuid(leaf: u32, subleaf: u32, abcd: &mut [u32; 4]) {
        let r = unsafe { __cpuid_count(leaf, subleaf) };
        *abcd = [r.eax, r.ebx, r.ecx, r.edx];
    }
    // Source: src/core/SkCpu.cpp:50-50
    // Source: src/core/SkCpu.cpp:66-68
    fn cpu_vendor(abcd: &mut [u32; 4]) {
        cpuid(kVendorLeaf, 0, abcd);
    }
    // Source: src/core/SkCpu.cpp:51-51
    // Source: src/core/SkCpu.cpp:69-71
    fn cpu_features(abcd: &mut [u32; 4]) {
        cpuid(kFMSFLeaf, 0, abcd);
    }
    // Source: src/core/SkCpu.cpp:52-52
    // Source: src/core/SkCpu.cpp:72-76
    fn cpu_flags(abcd: &mut [u32; 4]) {
        cpuid(kFlagsLeaf, kFlagsSubleaf, abcd);
    }
    // Source: src/core/SkCpu.cpp:54-58
    // Native MSVC branch uses the architecture intrinsic after the caller has
    // established both XSAVE hardware support and the OSXSAVE enable flag.
    #[cfg(target_env = "msvc")]
    #[target_feature(enable = "xsave")]
    unsafe fn xgetbv() -> u64 {
        const XCR: u32 = 0;
        #[cfg(target_arch = "x86")]
        {
            unsafe { std::arch::x86::_xgetbv(XCR) }
        }
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { std::arch::x86_64::_xgetbv(XCR) }
        }
    }
    // Source: src/core/SkCpu.cpp:78-85
    // Called only after hardware XSAVE and OSXSAVE have both been established.
    #[cfg(not(target_env = "msvc"))]
    unsafe fn xgetbv() -> u64 {
        let eax: u32;
        let edx: u32;
        std::arch::asm!("xgetbv", in("ecx") 0u32, out("eax") eax, out("edx") edx,
            options(nomem, nostack, preserves_flags));
        ((edx as u64) << 32) | eax as u64
    }

    // Source: src/core/SkCpu.cpp:91-91
    const kSSE: u32 = (1 << 25);
    // Source: src/core/SkCpu.cpp:92-92
    const kSSE2: u32 = (1 << 26);
    // Source: src/core/SkCpu.cpp:95-95
    const kSSE3: u32 = (1 << 0);
    // Source: src/core/SkCpu.cpp:96-96
    const kSSSE3: u32 = (1 << 9);
    // Source: src/core/SkCpu.cpp:97-97
    const kSSE41: u32 = (1 << 19);
    // Source: src/core/SkCpu.cpp:98-98
    const kSSE42: u32 = (1 << 20);
    // Source: src/core/SkCpu.cpp:99-99
    const kFMA: u32 = (1 << 12);
    // Source: src/core/SkCpu.cpp:100-100
    const kAVX: u32 = (1 << 28);
    // Source: src/core/SkCpu.cpp:101-101
    const kF16C: u32 = (1 << 29);
    // Source: src/core/SkCpu.cpp:102-102
    const kXSAVE: u32 = (1 << 26);
    // Source: src/core/SkCpu.cpp:103-103
    const kOSXSAVE: u32 = (1 << 27);
    // Source: src/core/SkCpu.cpp:106-106
    const kBMI1: u32 = (1 << 3);
    // Source: src/core/SkCpu.cpp:107-107
    const kAVX2: u32 = (1 << 5);
    // Source: src/core/SkCpu.cpp:108-108
    const kBMI2: u32 = (1 << 8);
    // Source: src/core/SkCpu.cpp:109-109
    const kERMS: u32 = (1 << 9);
    // Source: src/core/SkCpu.cpp:110-110
    const kAVX512F: u32 = (1 << 16);
    // Source: src/core/SkCpu.cpp:111-111
    const kAVX512DQ: u32 = (1 << 17);
    // Source: src/core/SkCpu.cpp:112-112
    const kAVX512IFMA: u32 = (1 << 21);
    // Source: src/core/SkCpu.cpp:113-113
    const kAVX512PF: u32 = (1 << 26);
    // Source: src/core/SkCpu.cpp:114-114
    const kAVX512ER: u32 = (1 << 27);
    // Source: src/core/SkCpu.cpp:115-115
    const kAVX512CD: u32 = (1 << 28);
    // Source: src/core/SkCpu.cpp:116-116
    const kAVX512BW: u32 = (1 << 30);
    // Source: src/core/SkCpu.cpp:117-117
    const kAVX512VL: u32 = (1 << 31);
    // Source: src/core/SkCpu.cpp:120-120
    const kAVX512VBMI2: u32 = (1 << 6);
    // Source: src/core/SkCpu.cpp:123-123
    const kXCR0_XMM_YMM_STATE: u64 = 0b00000110;
    // Source: src/core/SkCpu.cpp:124-124
    const kXCR0_ZMM_STATE: u64 = 0b11100000;
    // Source: src/core/SkCpu.cpp:127-130
    const fn ASCII_LE(s: &[u8; 4]) -> u32 {
        let [a, b, c, d] = *s;
        ((d as u32) << 24) | ((c as u32) << 16) | ((b as u32) << 8) | a as u32
    }
    // Source: src/core/SkCpu.cpp:132-213
    pub(super) fn read_cpu_features() -> u32 {
        let mut features = 0;
        let mut abcd = [0; 4];
        cpu_vendor(&mut abcd);
        let isAMD = abcd[1] == ASCII_LE(b"Auth")
            && abcd[3] == ASCII_LE(b"enti")
            && abcd[2] == ASCII_LE(b"cAMD");
        cpu_features(&mut abcd);
        if abcd[3] & kSSE != 0 {
            features |= SkX64::SSE1;
        }
        if abcd[3] & kSSE2 != 0 {
            features |= SkX64::SSE2;
        }
        if abcd[2] & kSSE3 != 0 {
            features |= SkX64::SSE3;
        }
        if abcd[2] & kSSSE3 != 0 {
            features |= SkX64::SSSE3;
        }
        if abcd[2] & kSSE41 != 0 {
            features |= SkX64::SSE41;
        }
        if abcd[2] & kSSE42 != 0 {
            features |= SkX64::SSE42;
        }
        if abcd[2] & (kXSAVE | kOSXSAVE) == (kXSAVE | kOSXSAVE) {
            let xcr = unsafe { xgetbv() };
            if xcr & kXCR0_XMM_YMM_STATE == kXCR0_XMM_YMM_STATE {
                if abcd[2] & kAVX != 0 {
                    features |= SkX64::AVX;
                }
                if abcd[2] & kF16C != 0 {
                    features |= SkX64::F16C;
                }
                if abcd[2] & kFMA != 0 {
                    features |= SkX64::FMA;
                }
                cpu_flags(&mut abcd);
                if abcd[1] & kAVX2 != 0 {
                    features |= SkX64::AVX2;
                }
                if abcd[1] & kBMI1 != 0 {
                    features |= SkX64::BMI1;
                }
                if abcd[1] & kBMI2 != 0 {
                    features |= SkX64::BMI2;
                }
                if abcd[1] & kERMS != 0 {
                    features |= SkX64::ERMS;
                }
                if xcr & kXCR0_ZMM_STATE == kXCR0_ZMM_STATE {
                    let isNewerIntel = abcd[2] & kAVX512VBMI2 != 0;
                    if isAMD || isNewerIntel {
                        if abcd[1] & kAVX512F != 0 {
                            features |= SkX64::AVX512F;
                        }
                        if abcd[1] & kAVX512DQ != 0 {
                            features |= SkX64::AVX512DQ;
                        }
                        if abcd[1] & kAVX512IFMA != 0 {
                            features |= SkX64::AVX512IFMA;
                        }
                        if abcd[1] & kAVX512PF != 0 {
                            features |= SkX64::AVX512PF;
                        }
                        if abcd[1] & kAVX512ER != 0 {
                            features |= SkX64::AVX512ER;
                        }
                        if abcd[1] & kAVX512CD != 0 {
                            features |= SkX64::AVX512CD;
                        }
                        if abcd[1] & kAVX512BW != 0 {
                            features |= SkX64::AVX512BW;
                        }
                        if abcd[1] & kAVX512VL != 0 {
                            features |= SkX64::AVX512VL;
                        }
                    }
                }
            }
        }
        features
    }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use x86::read_cpu_features;
// Source: src/core/SkCpu.cpp:215-223
// Linux ABI spellings come from Chromium's vendored libc-v0_2:
// linux_l4re_shared.rs (AT_HWCAP), linux/gnu/mod.rs (getauxval),
// linux/gnu/b64/loongarch64/mod.rs (LSX/LASX capability bits).
// The native getauxval call and feature tests remain unchanged.
#[cfg(target_arch = "loongarch64")]
fn read_cpu_features() -> u32 {
    use core::ffi::c_ulong;
    unsafe extern "C" {
        fn getauxval(kind: c_ulong) -> c_ulong;
    }
    const AT_HWCAP: c_ulong = 16;
    const HWCAP_LOONGARCH_LSX: c_ulong = 1 << 4;
    const HWCAP_LOONGARCH_LASX: c_ulong = 1 << 5;
    let mut features = 0;
    let hwcap = unsafe { getauxval(AT_HWCAP) };
    if hwcap & HWCAP_LOONGARCH_LSX != 0 {
        features |= SkLoongArch::SX;
    }
    if hwcap & HWCAP_LOONGARCH_LASX != 0 {
        features |= SkLoongArch::ASX;
    }
    features
}
// Source: src/core/SkCpu.cpp:225-227
#[cfg(not(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "loongarch64"
)))]
fn read_cpu_features() -> u32 {
    0
}
// Source: src/core/SkCpu.cpp:231-231
static gCachedFeatures: AtomicU32 = AtomicU32::new(0);
// Source: src/core/SkCpu.cpp:233-236
pub fn CacheRuntimeFeatures() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| gCachedFeatures.store(read_cpu_features(), Ordering::Relaxed));
}

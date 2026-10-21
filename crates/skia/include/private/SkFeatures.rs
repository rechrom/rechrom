// Compile-time transport only; no runtime CPU probing or platform provider.
// SK_FEATURES_INPUT_<macro> transports the native input at its conditional,
// including actual TargetConditionals outputs; not post-user-config outputs. "undefined" explicitly transports an absent macro;
// any other flag text transports defined presence, including the text "0".
// Integer inputs contain resolved decimal integers (not unevaluated C tokens).
const COMPLETE_NATIVE_INPUTS: bool = option_env!("SK_FEATURES_COMPLETE_NATIVE_INPUTS").is_some();
const fn text_eq(text: &str, expected: &[u8]) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != expected.len() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != expected[i] {
            return false;
        }
        i += 1;
    }
    true
}
const fn native_flag(value: Option<&str>, target_default: bool) -> bool {
    match value {
        Some(value) => !text_eq(value, b"undefined"),
        None => {
            if COMPLETE_NATIVE_INPUTS {
                false
            } else {
                target_default
            }
        }
    }
}
const fn native_integer(value: Option<&str>) -> Option<i32> {
    let Some(text) = value else {
        return None;
    };
    if text_eq(text, b"undefined") {
        return None;
    }
    let bytes = text.as_bytes();
    assert!(!bytes.is_empty(), "empty native integer input");
    let negative = bytes[0] == b'-';
    let mut i = if negative { 1 } else { 0 };
    let mut result = 0i64;
    assert!(i < bytes.len(), "invalid native integer input");
    while i < bytes.len() {
        let b = bytes[i];
        assert!(
            b >= b'0' && b <= b'9',
            "native integer input must be resolved decimal"
        );
        result = result * 10 + (b - b'0') as i64;
        assert!(
            result <= 2147483648,
            "native integer input exceeds i32 transport"
        );
        i += 1;
    }
    if negative {
        Some((-result) as i32)
    } else {
        assert!(
            result <= i32::MAX as i64,
            "native integer input exceeds i32 transport"
        );
        Some(result as i32)
    }
}
macro_rules! flag {
    ($name:literal,$fallback:expr) => {
        native_flag(option_env!(concat!("SK_FEATURES_INPUT_", $name)), $fallback)
    };
}
macro_rules! integer {
    ($name:literal) => {
        native_integer(option_env!(concat!("SK_FEATURES_INPUT_", $name)))
    };
}
#[derive(Clone, Copy)]
enum CxxCompiler {
    Clang,
    Gcc,
    Msvc,
    Other,
}
const CXX: CxxCompiler = match option_env!("SK_FEATURES_CXX_COMPILER") {
    Some(value) => {
        if text_eq(value, b"clang") {
            CxxCompiler::Clang
        } else if text_eq(value, b"gcc") {
            CxxCompiler::Gcc
        } else if text_eq(value, b"msvc") {
            CxxCompiler::Msvc
        } else if text_eq(value, b"other") {
            assert!(
                COMPLETE_NATIVE_INPUTS,
                "unmapped C++ compiler requires complete actual native inputs"
            );
            CxxCompiler::Other
        } else {
            panic!("unknown native C++ compiler input")
        }
    }
    None => {
        if COMPLETE_NATIVE_INPUTS {
            CxxCompiler::Other
        } else {
            panic!("SkFeatures requires actual SK_FEATURES_CXX_COMPILER input")
        }
    }
};
const GNU_STYLE_TARGET: bool = matches!(CXX, CxxCompiler::Clang | CxxCompiler::Gcc);
const MSVC_TARGET: bool = matches!(CXX, CxxCompiler::Msvc);
const fn target_conditional(raw: Option<&str>) -> bool {
    match native_integer(raw) {
        Some(value) => value != 0,
        None => {
            if raw.is_some() || COMPLETE_NATIVE_INPUTS {
                false
            } else if APPLE {
                panic!("actual Apple TargetConditionals input required")
            } else {
                false
            }
        }
    }
}

// Source: include/private/SkFeatures.h:11-36
// Explicit existing platform presences suppress ALL automatic selection, even
// if multiple are present. WASM intentionally is not in the original guard.
const OVERRIDE_ANDROID: bool = flag!("SK_BUILD_FOR_ANDROID", false);
const OVERRIDE_IOS: bool = flag!("SK_BUILD_FOR_IOS", false);
const OVERRIDE_WIN: bool = flag!("SK_BUILD_FOR_WIN", false);
const OVERRIDE_UNIX: bool = flag!("SK_BUILD_FOR_UNIX", false);
const OVERRIDE_MAC: bool = flag!("SK_BUILD_FOR_MAC", false);
const OS_OVERRIDDEN: bool =
    OVERRIDE_ANDROID || OVERRIDE_IOS || OVERRIDE_WIN || OVERRIDE_UNIX || OVERRIDE_MAC;
#[derive(Clone, Copy)]
enum Platform {
    None,
    Win,
    Android,
    Wasm,
    Unix,
    Ios,
    Mac,
}
const APPLE: bool = flag!("__APPLE__", cfg!(target_vendor = "apple"));
const DETECTED_PLATFORM: Platform = if OS_OVERRIDDEN {
    Platform::None
} else if flag!("_WIN32", cfg!(target_os = "windows")) || flag!("__SYMBIAN32__", false) {
    Platform::Win
} else if flag!("ANDROID", false) || flag!("__ANDROID__", cfg!(target_os = "android")) {
    Platform::Android
} else if flag!("__EMSCRIPTEN__", cfg!(target_os = "emscripten")) {
    Platform::Wasm
} else if flag!("linux", false)
    || flag!("__linux", cfg!(target_os = "linux"))
    || flag!("__FreeBSD__", cfg!(target_os = "freebsd"))
    || flag!("__OpenBSD__", cfg!(target_os = "openbsd"))
    || flag!(
        "__sun",
        cfg!(any(target_os = "solaris", target_os = "illumos"))
    )
    || flag!("__NetBSD__", cfg!(target_os = "netbsd"))
    || flag!("__DragonFly__", cfg!(target_os = "dragonfly"))
    || flag!("__Fuchsia__", cfg!(target_os = "fuchsia"))
    || flag!("__GLIBC__", false)
    || flag!("__GNU__", false)
    || flag!("__unix__", false)
{
    Platform::Unix
} else if target_conditional(option_env!("SK_FEATURES_INPUT_TARGET_OS_IPHONE"))
    || target_conditional(option_env!("SK_FEATURES_INPUT_TARGET_IPHONE_SIMULATOR"))
{
    Platform::Ios
} else {
    Platform::Mac
};
pub const SK_BUILD_FOR_WIN: bool = OVERRIDE_WIN || matches!(DETECTED_PLATFORM, Platform::Win);
pub const SK_BUILD_FOR_ANDROID: bool =
    OVERRIDE_ANDROID || matches!(DETECTED_PLATFORM, Platform::Android);
pub const SK_BUILD_FOR_WASM: bool =
    flag!("SK_BUILD_FOR_WASM", false) || matches!(DETECTED_PLATFORM, Platform::Wasm);
pub const SK_BUILD_FOR_UNIX: bool = OVERRIDE_UNIX || matches!(DETECTED_PLATFORM, Platform::Unix);
pub const SK_BUILD_FOR_IOS: bool = OVERRIDE_IOS || matches!(DETECTED_PLATFORM, Platform::Ios);
pub const SK_BUILD_FOR_MAC: bool = OVERRIDE_MAC || matches!(DETECTED_PLATFORM, Platform::Mac);

// Source: include/private/SkFeatures.h:39-47
// ADAPTED metadata only: Rust has no C++ restrict spelling with this guarantee.
// &mut exclusivity/raw-pointer caller leases must be established independently.
// No consumer may infer noalias merely from this preserved native token string.
pub const SK_RESTRICT_NATIVE_SPELLING: &str = match option_env!("SK_FEATURES_INPUT_SK_RESTRICT") {
    Some(value) => {
        if text_eq(value, b"undefined") {
            if SK_BUILD_FOR_WIN && !flag!("__clang__", matches!(CXX, CxxCompiler::Clang)) {
                "__restrict"
            } else {
                "__restrict__"
            }
        } else {
            value
        }
    }
    None => {
        if SK_BUILD_FOR_WIN && !flag!("__clang__", matches!(CXX, CxxCompiler::Clang)) {
            "__restrict"
        } else {
            "__restrict__"
        }
    }
};

// Source: include/private/SkFeatures.h:49-66
const OVERRIDE_BENDIAN: bool = flag!("SK_CPU_BENDIAN", false);
const OVERRIDE_LENDIAN: bool = flag!("SK_CPU_LENDIAN", false);
const fn byte_order_match(big: bool) -> bool {
    if let Some(raw) = integer!("__BYTE_ORDER__") {
        // If an explicit numeric compiler macro is supplied, its comparison
        // constants must also be actual resolved inputs (or explicit undefined).
        if GNU_STYLE_TARGET && !COMPLETE_NATIVE_INPUTS {
            if big {
                assert!(
                    option_env!("SK_FEATURES_INPUT___ORDER_BIG_ENDIAN__").is_some(),
                    "actual __ORDER_BIG_ENDIAN__ input required"
                );
            } else {
                assert!(
                    option_env!("SK_FEATURES_INPUT___ORDER_LITTLE_ENDIAN__").is_some(),
                    "actual __ORDER_LITTLE_ENDIAN__ input required"
                );
            }
        }
        let order = if big {
            integer!("__ORDER_BIG_ENDIAN__")
        } else {
            integer!("__ORDER_LITTLE_ENDIAN__")
        };
        raw == match order {
            Some(value) => value,
            None => 0,
        }
    } else if flag!("__BYTE_ORDER__", GNU_STYLE_TARGET) {
        if big {
            cfg!(target_endian = "big")
        } else {
            cfg!(target_endian = "little")
        }
    } else {
        false
    }
}
const DETECTED_BENDIAN: bool = if OVERRIDE_BENDIAN || OVERRIDE_LENDIAN {
    false
} else if byte_order_match(true) {
    true
} else if byte_order_match(false) {
    false
} else if flag!("__sparc", false)
    || flag!(
        "__sparc__",
        GNU_STYLE_TARGET && cfg!(any(target_arch = "sparc", target_arch = "sparc64"))
    )
    || flag!("_POWER", false)
    || flag!(
        "__powerpc__",
        GNU_STYLE_TARGET && cfg!(any(target_arch = "powerpc", target_arch = "powerpc64"))
    )
    || flag!("__ppc__", false)
    || flag!("__hppa", false)
    || flag!("__PPC__", false)
    || flag!("__PPC64__", false)
    || flag!("_MIPSEB", false)
    || flag!("__ARMEB__", false)
    || flag!("__s390__", GNU_STYLE_TARGET && cfg!(target_arch = "s390x"))
    || (flag!("__sh__", false) && flag!("__BIG_ENDIAN__", false))
    || (flag!("__ia64", false) && flag!("__BIG_ENDIAN__", false))
{
    true
} else {
    false
};
pub const SK_CPU_BENDIAN: bool = OVERRIDE_BENDIAN || (!OVERRIDE_LENDIAN && DETECTED_BENDIAN);
pub const SK_CPU_LENDIAN: bool = OVERRIDE_LENDIAN || (!OVERRIDE_BENDIAN && !DETECTED_BENDIAN);

// Source: include/private/SkFeatures.h:68-70
const DETECTED_X86: bool = flag!("__i386", GNU_STYLE_TARGET && cfg!(target_arch = "x86"))
    || flag!("_M_IX86", MSVC_TARGET && cfg!(target_arch = "x86"))
    || flag!(
        "__x86_64__",
        GNU_STYLE_TARGET && cfg!(target_arch = "x86_64")
    )
    || flag!("_M_X64", MSVC_TARGET && cfg!(target_arch = "x86_64"));
pub const SK_CPU_X86: Option<i32> = if DETECTED_X86 {
    Some(1)
} else {
    integer!("SK_CPU_X86")
};
// Source: include/private/SkFeatures.h:72-74
const DETECTED_LOONGARCH: bool = flag!(
    "__loongarch__",
    GNU_STYLE_TARGET && cfg!(target_arch = "loongarch64")
) || flag!(
    "__loongarch64",
    GNU_STYLE_TARGET && cfg!(target_arch = "loongarch64")
);
pub const SK_CPU_LOONGARCH: Option<i32> = if DETECTED_LOONGARCH {
    Some(1)
} else {
    integer!("SK_CPU_LOONGARCH")
};
// Source: include/private/SkFeatures.h:76-78
const DETECTED_PPC: bool = flag!(
    "__powerpc__",
    GNU_STYLE_TARGET && cfg!(any(target_arch = "powerpc", target_arch = "powerpc64"))
) || flag!(
    "__powerpc64__",
    GNU_STYLE_TARGET && cfg!(target_arch = "powerpc64")
);
pub const SK_CPU_PPC: Option<i32> = if DETECTED_PPC {
    Some(1)
} else {
    integer!("SK_CPU_PPC")
};

// Source: include/private/SkFeatures.h:86-94
pub const SK_CPU_X64_LEVEL_SSE1: i32 = 10;
pub const SK_CPU_X64_LEVEL_SSE2: i32 = 20;
pub const SK_CPU_X64_LEVEL_SSE3: i32 = 30;
pub const SK_CPU_X64_LEVEL_SSSE3: i32 = 31;
pub const SK_CPU_X64_LEVEL_SSE41: i32 = 41;
pub const SK_CPU_X64_LEVEL_SSE42: i32 = 42;
pub const SK_CPU_X64_LEVEL_AVX: i32 = 51;
pub const SK_CPU_X64_LEVEL_AVX2: i32 = 52;
pub const SK_CPU_X64_LEVEL_ML4: i32 = 60;
// Source: include/private/SkFeatures.h:102-103
pub const SK_CPU_LSX_LEVEL_LSX: i32 = 10;
pub const SK_CPU_LSX_LEVEL_LASX: i32 = 20;

// Source: include/private/SkFeatures.h:108-129
const X64_FIRST: Option<i32> = if let Some(level) = integer!("SK_CPU_X64_LEVEL") {
    Some(level)
} else if flag!(
    "__AVX512F__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512f")
) && flag!(
    "__AVX512DQ__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512dq")
) && flag!(
    "__AVX512CD__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512cd")
) && flag!(
    "__AVX512BW__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512bw")
) && flag!(
    "__AVX512VL__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512vl")
) {
    Some(SK_CPU_X64_LEVEL_ML4)
} else if flag!(
    "__AVX2__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx2")
) {
    Some(SK_CPU_X64_LEVEL_AVX2)
} else if flag!("__AVX__", GNU_STYLE_TARGET && cfg!(target_feature = "avx")) {
    Some(SK_CPU_X64_LEVEL_AVX)
} else if flag!(
    "__SSE4_2__",
    GNU_STYLE_TARGET && cfg!(target_feature = "sse4.2")
) {
    Some(SK_CPU_X64_LEVEL_SSE42)
} else if flag!(
    "__SSE4_1__",
    GNU_STYLE_TARGET && cfg!(target_feature = "sse4.1")
) {
    Some(SK_CPU_X64_LEVEL_SSE41)
} else if flag!(
    "__SSSE3__",
    GNU_STYLE_TARGET && cfg!(target_feature = "ssse3")
) {
    Some(SK_CPU_X64_LEVEL_SSSE3)
} else if flag!(
    "__SSE3__",
    GNU_STYLE_TARGET && cfg!(target_feature = "sse3")
) {
    Some(SK_CPU_X64_LEVEL_SSE3)
} else if flag!(
    "__SSE2__",
    GNU_STYLE_TARGET && cfg!(target_feature = "sse2")
) {
    Some(SK_CPU_X64_LEVEL_SSE2)
} else {
    None
};
// Source: include/private/SkFeatures.h:131-137
pub const SK_CPU_LSX_LEVEL: Option<i32> = if let Some(level) = integer!("SK_CPU_LSX_LEVEL") {
    Some(level)
} else if flag!(
    "__loongarch_asx",
    GNU_STYLE_TARGET && cfg!(target_feature = "lasx")
) {
    Some(SK_CPU_LSX_LEVEL_LASX)
} else if flag!(
    "__loongarch_sx",
    GNU_STYLE_TARGET && cfg!(target_feature = "lsx")
) {
    Some(SK_CPU_LSX_LEVEL_LSX)
} else {
    None
};

// Source: include/private/SkFeatures.h:140-159
pub const SK_CPU_X64_LEVEL: Option<i32> = if X64_FIRST.is_some() {
    X64_FIRST
} else if flag!(
    "__AVX512F__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512f")
) && flag!(
    "__AVX512DQ__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512dq")
) && flag!(
    "__AVX512CD__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512cd")
) && flag!(
    "__AVX512BW__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512bw")
) && flag!(
    "__AVX512VL__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx512vl")
) {
    Some(SK_CPU_X64_LEVEL_ML4)
} else if flag!(
    "__AVX2__",
    GNU_STYLE_TARGET && cfg!(target_feature = "avx2")
) {
    Some(SK_CPU_X64_LEVEL_AVX2)
} else if flag!("__AVX__", GNU_STYLE_TARGET && cfg!(target_feature = "avx")) {
    Some(SK_CPU_X64_LEVEL_AVX)
} else if flag!("_M_X64", MSVC_TARGET && cfg!(target_arch = "x86_64"))
    || flag!("_M_AMD64", MSVC_TARGET && cfg!(target_arch = "x86_64"))
{
    Some(SK_CPU_X64_LEVEL_SSE2)
} else {
    let fp = integer!("_M_IX86_FP");
    if MSVC_TARGET
        && cfg!(target_arch = "x86")
        && !COMPLETE_NATIVE_INPUTS
        && option_env!("SK_FEATURES_INPUT__M_IX86_FP").is_none()
    {
        panic!("actual _M_IX86_FP value or explicit undefined required");
    }
    if let Some(fp) = fp {
        if fp >= 2 {
            Some(SK_CPU_X64_LEVEL_SSE2)
        } else if fp == 1 {
            Some(SK_CPU_X64_LEVEL_SSE1)
        } else {
            None
        }
    } else {
        None
    }
};

// Source: include/private/SkFeatures.h:162-166
const ARM32_DETECTED: bool = flag!("__arm__", GNU_STYLE_TARGET && cfg!(target_arch = "arm"))
    && (!APPLE || !target_conditional(option_env!("SK_FEATURES_INPUT_TARGET_IPHONE_SIMULATOR")));
pub const SK_CPU_ARM32: bool = flag!("SK_CPU_ARM32", false) || ARM32_DETECTED;
pub const SK_CPU_ARM64: bool = flag!("SK_CPU_ARM64", false)
    || (!ARM32_DETECTED
        && flag!(
            "__aarch64__",
            GNU_STYLE_TARGET && cfg!(target_arch = "aarch64")
        ));
// Source: include/private/SkFeatures.h:169-171
// The original tests __ARM_NEON; do not infer NEON solely from ARM64's name.
pub const SK_ARM_HAS_NEON: bool = flag!("SK_ARM_HAS_NEON", false)
    || flag!(
        "__ARM_NEON",
        GNU_STYLE_TARGET && cfg!(target_feature = "neon")
    );

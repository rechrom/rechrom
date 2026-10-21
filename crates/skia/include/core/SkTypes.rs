// Compile-time adapter for the resolved native preprocessor inputs. These
// environment entries are not inferred from the host: the// Exact integer transport for resolved native compile inputs, not a C/C++
// expression or floating-point parser. The build bridge supplies normalized
// decimal values/bits after resolving the native compilation environment.
const fn resolved_decimal_u32(text: &str) -> u32 {
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut value = 0u32;
    assert!(!bytes.is_empty(), "empty resolved native integer");
    while i < bytes.len() {
        assert!(
            bytes[i] >= b'0' && bytes[i] <= b'9',
            "resolved native integer must be decimal"
        );
        value = match value.checked_mul(10) {
            Some(v) => v,
            None => panic!("resolved u32 overflow"),
        };
        value = match value.checked_add((bytes[i] - b'0') as u32) {
            Some(v) => v,
            None => panic!("resolved u32 overflow"),
        };
        i += 1;
    }
    value
}
const fn resolved_decimal_i32(text: &str) -> i32 {
    let bytes = text.as_bytes();
    assert!(!bytes.is_empty(), "empty resolved native integer");
    let negative = bytes[0] == b'-';
    let mut i = if negative { 1 } else { 0 };
    assert!(i < bytes.len(), "missing resolved native integer digits");
    let mut value = 0i64;
    while i < bytes.len() {
        assert!(
            bytes[i] >= b'0' && bytes[i] <= b'9',
            "resolved native integer must be decimal"
        );
        value = value * 10 + (bytes[i] - b'0') as i64;
        assert!(value <= 2147483648, "resolved i32 overflow");
        i += 1;
    }
    if negative {
        (-value) as i32
    } else {
        assert!(value <= i32::MAX as i64, "resolved i32 overflow");
        value as i32
    }
}
// Source: include/core/SkTypes.h:80-82
pub const SK_ALLOW_STATIC_GLOBAL_INITIALIZERS: i32 =
    match option_env!("SKIA_RESOLVED_ALLOW_STATIC_GLOBAL_INITIALIZERS_I32") {
        Some(value) => resolved_decimal_i32(value),
        None => {
            assert!(
                option_env!("SK_ALLOW_STATIC_GLOBAL_INITIALIZERS").is_none(),
                "provide resolved SK_ALLOW_STATIC_GLOBAL_INITIALIZERS integer input"
            );
            0
        }
    };
// Source: include/core/SkTypes.h:84-86
pub const SK_GAMMA_EXPONENT: f32 = match option_env!("SKIA_RESOLVED_GAMMA_EXPONENT_BITS") {
    Some(bits) => f32::from_bits(resolved_decimal_u32(bits)),
    None => {
        assert!(
            option_env!("SK_GAMMA_EXPONENT").is_none(),
            "provide native-resolved SK_GAMMA_EXPONENT f32 bits"
        );
        f32::from_bits(0)
    }
};
// Source: include/core/SkTypes.h:88-94
pub const SK_GAMMA_CONTRAST: f32 = match option_env!("SKIA_RESOLVED_GAMMA_CONTRAST_BITS") {
    Some(bits) => f32::from_bits(resolved_decimal_u32(bits)),
    None => {
        assert!(
            option_env!("SK_GAMMA_CONTRAST").is_none(),
            "provide native-resolved SK_GAMMA_CONTRAST f32 bits"
        );
        f32::from_bits(0x3f000000)
    }
};
pub const NATIVE_HISTOGRAM_ENUMERATION_DEFINED: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_HISTOGRAM_ENUMERATION_DEFINED"),
    option_env!("SK_HISTOGRAM_ENUMERATION"),
);
pub const NATIVE_HISTOGRAM_BOOLEAN_DEFINED: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_HISTOGRAM_BOOLEAN_DEFINED"),
    option_env!("SK_HISTOGRAM_BOOLEAN"),
);
pub const NATIVE_HISTOGRAM_EXACT_LINEAR_DEFINED: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_HISTOGRAM_EXACT_LINEAR_DEFINED"),
    option_env!("SK_HISTOGRAM_EXACT_LINEAR"),
);
pub const NATIVE_HISTOGRAM_CUSTOM_EXACT_LINEAR_DEFINED: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_HISTOGRAM_CUSTOM_EXACT_LINEAR_DEFINED"),
    option_env!("SK_HISTOGRAM_CUSTOM_EXACT_LINEAR"),
);
pub const NATIVE_HISTOGRAM_MEMORY_KB_DEFINED: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_HISTOGRAM_MEMORY_KB_DEFINED"),
    option_env!("SK_HISTOGRAM_MEMORY_KB"),
);
pub const NATIVE_HISTOGRAM_CUSTOM_COUNTS_DEFINED: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_HISTOGRAM_CUSTOM_COUNTS_DEFINED"),
    option_env!("SK_HISTOGRAM_CUSTOM_COUNTS"),
);
pub const NATIVE_HISTOGRAM_CUSTOM_MICROSECONDS_TIMES_DEFINED: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_HISTOGRAM_CUSTOM_MICROSECONDS_TIMES_DEFINED"),
    option_env!("SK_HISTOGRAM_CUSTOM_MICROSECONDS_TIMES"),
);
// Source: include/core/SkTypes.h:96-106
// Native definedness OR is evaluated before fallback definitions are supplied.
pub const SK_HISTOGRAMS_ENABLED: i32 = if NATIVE_HISTOGRAM_ENUMERATION_DEFINED
    || NATIVE_HISTOGRAM_BOOLEAN_DEFINED
    || NATIVE_HISTOGRAM_EXACT_LINEAR_DEFINED
    || NATIVE_HISTOGRAM_CUSTOM_EXACT_LINEAR_DEFINED
    || NATIVE_HISTOGRAM_MEMORY_KB_DEFINED
    || NATIVE_HISTOGRAM_CUSTOM_COUNTS_DEFINED
    || NATIVE_HISTOGRAM_CUSTOM_MICROSECONDS_TIMES_DEFINED
{
    1
} else {
    0
};
// Source: include/core/SkTypes.h:108-110
// Undefined native fallback discards all tokens without evaluating arguments.
// A configured external provider is NOT replaced with a successful no-op:
// using that branch explicitly requires its actual provider translation.
macro_rules! SK_HISTOGRAM_BOOLEAN {
    ($($arguments:tt)*) => {{
        const {
            assert!(
                !$crate::include::core::SkTypes::NATIVE_HISTOGRAM_BOOLEAN_DEFINED,
                "configured native SK_HISTOGRAM_BOOLEAN provider is not yet translated"
            );
        }
    }};
}
pub(crate) use SK_HISTOGRAM_BOOLEAN;
// Source: include/core/SkTypes.h:112-114
// Undefined native fallback discards all tokens without evaluating arguments.
// A configured external provider is NOT replaced with a successful no-op:
// using that branch explicitly requires its actual provider translation.
macro_rules! SK_HISTOGRAM_ENUMERATION {
    ($($arguments:tt)*) => {{
        const {
            assert!(
                !$crate::include::core::SkTypes::NATIVE_HISTOGRAM_ENUMERATION_DEFINED,
                "configured native SK_HISTOGRAM_ENUMERATION provider is not yet translated"
            );
        }
    }};
}
pub(crate) use SK_HISTOGRAM_ENUMERATION;
// Source: include/core/SkTypes.h:116-118
// Undefined native fallback discards all tokens without evaluating arguments.
// A configured external provider is NOT replaced with a successful no-op:
// using that branch explicitly requires its actual provider translation.
macro_rules! SK_HISTOGRAM_EXACT_LINEAR {
    ($($arguments:tt)*) => {{
        const {
            assert!(
                !$crate::include::core::SkTypes::NATIVE_HISTOGRAM_EXACT_LINEAR_DEFINED,
                "configured native SK_HISTOGRAM_EXACT_LINEAR provider is not yet translated"
            );
        }
    }};
}
pub(crate) use SK_HISTOGRAM_EXACT_LINEAR;
// Source: include/core/SkTypes.h:120-123
// Undefined native fallback discards all tokens without evaluating arguments.
// A configured external provider is NOT replaced with a successful no-op:
// using that branch explicitly requires its actual provider translation.
macro_rules! SK_HISTOGRAM_CUSTOM_EXACT_LINEAR {
    ($($arguments:tt)*) => {{
        const {
            assert!(
                !$crate::include::core::SkTypes::NATIVE_HISTOGRAM_CUSTOM_EXACT_LINEAR_DEFINED,
                "configured native SK_HISTOGRAM_CUSTOM_EXACT_LINEAR provider is not yet translated"
            );
        }
    }};
}
pub(crate) use SK_HISTOGRAM_CUSTOM_EXACT_LINEAR;
// Source: include/core/SkTypes.h:125-127
// Undefined native fallback discards all tokens without evaluating arguments.
// A configured external provider is NOT replaced with a successful no-op:
// using that branch explicitly requires its actual provider translation.
macro_rules! SK_HISTOGRAM_MEMORY_KB {
    ($($arguments:tt)*) => {{
        const {
            assert!(
                !$crate::include::core::SkTypes::NATIVE_HISTOGRAM_MEMORY_KB_DEFINED,
                "configured native SK_HISTOGRAM_MEMORY_KB provider is not yet translated"
            );
        }
    }};
}
pub(crate) use SK_HISTOGRAM_MEMORY_KB;
// Source: include/core/SkTypes.h:129-131
// Undefined native fallback discards all tokens without evaluating arguments.
// A configured external provider is NOT replaced with a successful no-op:
// using that branch explicitly requires its actual provider translation.
macro_rules! SK_HISTOGRAM_CUSTOM_COUNTS {
    ($($arguments:tt)*) => {{
        const {
            assert!(
                !$crate::include::core::SkTypes::NATIVE_HISTOGRAM_CUSTOM_COUNTS_DEFINED,
                "configured native SK_HISTOGRAM_CUSTOM_COUNTS provider is not yet translated"
            );
        }
    }};
}
pub(crate) use SK_HISTOGRAM_CUSTOM_COUNTS;
// Source: include/core/SkTypes.h:133-135
// Undefined native fallback discards all tokens without evaluating arguments.
// A configured external provider is NOT replaced with a successful no-op:
// using that branch explicitly requires its actual provider translation.
macro_rules! SK_HISTOGRAM_CUSTOM_MICROSECONDS_TIMES {($($arguments:tt)*)=>{{
    const {assert!(!$crate::include::core::SkTypes::NATIVE_HISTOGRAM_CUSTOM_MICROSECONDS_TIMES_DEFINED,
        "configured native SK_HISTOGRAM_CUSTOM_MICROSECONDS_TIMES provider is not yet translated");}
}};}
pub(crate) use SK_HISTOGRAM_CUSTOM_MICROSECONDS_TIMES;
// Source: include/core/SkTypes.h:137-138
macro_rules! SK_HISTOGRAM_PERCENTAGE {
    ($name:expr,$percent:expr) => {
        $crate::include::core::SkTypes::SK_HISTOGRAM_EXACT_LINEAR!($name, $percent, 101)
    };
}
pub(crate) use SK_HISTOGRAM_PERCENTAGE;
// Source: include/core/SkTypes.h:140-146
pub const SK_FORCE_RASTER_PIPELINE_BLITTER: bool = option_env!("SK_FORCE_RASTER_PIPELINE_BLITTER")
    .is_some()
    || option_env!("SK_ENABLE_OPTIMIZE_SIZE").is_some();
pub const SK_DISABLE_SDF_TEXT: bool = option_env!("SK_DISABLE_SDF_TEXT").is_some()
    || option_env!("SK_ENABLE_OPTIMIZE_SIZE").is_some();
// Source: include/core/SkTypes.h:148-152
pub const SK_BUILD_FOR_FUZZER: bool = option_env!("SK_BUILD_FOR_FUZZER").is_some()
    || option_env!("SK_BUILD_FOR_LIBFUZZER").is_some()
    || option_env!("SK_BUILD_FOR_AFL_FUZZ").is_some();
// Compile-time adapter for the resolved native preprocessor inputs. These
// environment entries are not inferred from the host: the embedding must export
// effective user-config/command-line definitions to rustc. Absent overrides use
// the same native target defaults. Only resolved decimal shifts 0/16 are accepted.
const fn native_shift_input(value: &str) -> u32 {
    match value.as_bytes() {
        [b'0'] => 0,
        [b'1', b'6'] => 16,
        _ => panic!("SK_*32_SHIFT must be 0 or 16"),
    }
}
const NATIVE_BUILD_FOR_WIN: bool = option_env!("SK_BUILD_FOR_WIN").is_some()
    || (option_env!("SK_BUILD_FOR_ANDROID").is_none()
        && option_env!("SK_BUILD_FOR_IOS").is_none()
        && option_env!("SK_BUILD_FOR_UNIX").is_none()
        && option_env!("SK_BUILD_FOR_MAC").is_none()
        && crate::include::private::SkFeatures::SK_BUILD_FOR_WIN);
// Source: include/core/SkTypes.h:38-44
pub const SK_R32_SHIFT: u32 = match option_env!("SK_R32_SHIFT") {
    Some(value) => native_shift_input(value),
    None => {
        if NATIVE_BUILD_FOR_WIN {
            16
        } else {
            0
        }
    }
};
const _: () = assert!(SK_R32_SHIFT == 0 || SK_R32_SHIFT == 16, "");
// Source: include/core/SkTypes.h:46-50
pub const SK_B32_SHIFT: u32 = match option_env!("SK_B32_SHIFT") {
    Some(value) => native_shift_input(value),
    None => 16 - SK_R32_SHIFT,
};
const _: () = assert!(SK_B32_SHIFT == 16 - SK_R32_SHIFT, "");
// Source: include/core/SkTypes.h:52-53
// These native macros are unconditional after user config; there is no separate
// configurable G/A value in this source region.
pub const SK_G32_SHIFT: u32 = 8;
pub const SK_A32_SHIFT: u32 = 24;
// Native SkFeatures/SkLoadUserConfig supplies these effective macro presences.
// A defined macro counts as defined regardless of its text value, as in C++.
// Rust integration inputs for an embedding's resolved define/undef pair.
// The native Skia defaults remain available when no embedding resolution exists.
const fn resolved_presence(resolved: Option<&str>, raw: Option<&str>) -> bool {
    match resolved {
        Some(value) => match value.as_bytes() {
            [b'0'] => false,
            [b'1'] => true,
            _ => panic!("resolved macro presence must be 0 or 1"),
        },
        None => raw.is_some(),
    }
}
const USER_BENDIAN: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_CPU_BENDIAN"),
    option_env!("SK_CPU_BENDIAN"),
);
const USER_LENDIAN: bool = resolved_presence(
    option_env!("SKIA_RESOLVED_CPU_LENDIAN"),
    option_env!("SK_CPU_LENDIAN"),
);
const _: () = assert!(
    !(USER_BENDIAN && USER_LENDIAN),
    "cannot define both SK_CPU_LENDIAN and SK_CPU_BENDIAN"
);
const USER_ENDIAN_RESOLVED: bool = option_env!("SKIA_RESOLVED_CPU_BENDIAN").is_some()
    || option_env!("SKIA_RESOLVED_CPU_LENDIAN").is_some()
    || USER_BENDIAN
    || USER_LENDIAN;
const NATIVE_CPU_BENDIAN: bool = if USER_ENDIAN_RESOLVED {
    USER_BENDIAN
} else {
    crate::include::private::SkFeatures::SK_CPU_BENDIAN
};
const NATIVE_CPU_LENDIAN: bool = if USER_ENDIAN_RESOLVED {
    USER_LENDIAN
} else {
    crate::include::private::SkFeatures::SK_CPU_LENDIAN
};
const _: () = assert!(
    !(NATIVE_CPU_BENDIAN && NATIVE_CPU_LENDIAN),
    "cannot define both SK_CPU_LENDIAN and SK_CPU_BENDIAN"
);
const _: () = assert!(
    NATIVE_CPU_BENDIAN || NATIVE_CPU_LENDIAN,
    "must define either SK_CPU_LENDIAN or SK_CPU_BENDIAN"
);
// Source: include/core/SkTypes.h:58-70
// Token concatenation becomes a typed compile-time channel selector. Every
// comparison follows the selected native endian branch rather than host order.
#[derive(Clone, Copy)]
pub enum SkPMColorChannel {
    R,
    G,
    B,
    A,
}
const fn pm_channel_shift(channel: SkPMColorChannel) -> u32 {
    match channel {
        SkPMColorChannel::R => SK_R32_SHIFT,
        SkPMColorChannel::G => SK_G32_SHIFT,
        SkPMColorChannel::B => SK_B32_SHIFT,
        SkPMColorChannel::A => SK_A32_SHIFT,
    }
}
pub const fn SK_PMCOLOR_BYTE_ORDER(
    c0: SkPMColorChannel,
    c1: SkPMColorChannel,
    c2: SkPMColorChannel,
    c3: SkPMColorChannel,
) -> bool {
    if NATIVE_CPU_BENDIAN {
        pm_channel_shift(c3) == 0
            && pm_channel_shift(c2) == 8
            && pm_channel_shift(c1) == 16
            && pm_channel_shift(c0) == 24
    } else {
        pm_channel_shift(c0) == 0
            && pm_channel_shift(c1) == 8
            && pm_channel_shift(c2) == 16
            && pm_channel_shift(c3) == 24
    }
}
// Source: include/core/SkTypes.h:179-179
pub type SkUnichar = i32;
// Source: include/core/SkTypes.h:183-183
pub type SkGlyphID = u16;
// Source: include/core/SkTypes.h:187-187
pub const SK_InvalidGenID: u32 = 0;
// Source: include/core/SkTypes.h:191-191
pub const SK_InvalidUniqueID: u32 = 0;

// Source: include/core/SkTypes.h:159-165
// Numeric override wins even when its value is zero in a debug build.
pub const GR_CACHE_STATS: i32 = match option_env!("SKIA_RESOLVED_GR_CACHE_STATS_I32") {
    Some(value) => resolved_decimal_i32(value),
    None => {
        assert!(
            option_env!("GR_CACHE_STATS").is_none(),
            "provide resolved GR_CACHE_STATS integer input"
        );
        if cfg!(GR_CACHE_STATS = "1") {
            1
        } else if cfg!(GR_CACHE_STATS = "0") {
            0
        } else if option_env!("SK_DUMP_STATS").is_some()
            || cfg!(any(sk_debug, SK_DUMP_STATS, sk_dump_stats))
        {
            1
        } else {
            0
        }
    }
};
// Source: include/core/SkTypes.h:167-173
pub const GR_GPU_STATS: i32 = match option_env!("SKIA_RESOLVED_GR_GPU_STATS_I32") {
    Some(value) => resolved_decimal_i32(value),
    None => {
        assert!(
            option_env!("GR_GPU_STATS").is_none(),
            "provide resolved GR_GPU_STATS integer input"
        );
        if cfg!(GR_GPU_STATS = "1") {
            1
        } else if cfg!(GR_GPU_STATS = "0") {
            0
        } else if option_env!("SK_DUMP_STATS").is_some()
            || option_env!("GPU_TEST_UTILS").is_some()
            || cfg!(any(
                sk_debug,
                SK_DUMP_STATS,
                sk_dump_stats,
                GPU_TEST_UTILS,
                gpu_test_utils
            ))
        {
            1
        } else {
            0
        }
    }
};

// Copyright 2022 Google LLC. BSD-style license.
// Native preprocessing is transported from the actual compilation stream.
// No Rust debug_assertions/host heuristic overrides explicit native SK_RELEASE.
const COMPLETE_INPUTS: bool = option_env!("SK_LOAD_USER_CONFIG_COMPLETE_INPUTS").is_some();
const fn text_eq(text: &str, other: &[u8]) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != other.len() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != other[i] {
            return false;
        }
        i += 1;
    }
    true
}
const fn flag(input: Option<&str>, raw: Option<&str>, default: bool) -> bool {
    match input {
        Some(value) => !text_eq(value, b"undefined"),
        None => {
            if COMPLETE_INPUTS {
                false
            } else {
                raw.is_some() || default
            }
        }
    }
}
const fn definition(
    input: Option<&'static str>,
    raw: Option<&'static str>,
) -> Option<&'static str> {
    match input {
        Some(value) => {
            if text_eq(value, b"undefined") {
                None
            } else {
                Some(value)
            }
        }
        None => {
            if COMPLETE_INPUTS {
                None
            } else {
                raw
            }
        }
    }
}
// Source: include/private/SkLoadUserConfig.h:8-8
pub const PREVIOUSLY_LOADED: bool = flag(
    option_env!("SK_LOAD_USER_CONFIG_INPUT_SK_USER_CONFIG_WAS_LOADED"),
    option_env!("SK_USER_CONFIG_WAS_LOADED"),
    false,
);
// Rust cfg controls the actual macro definition items at this native guard.
// This checks only transport consistency, not a runtime Skia algorithm.
const _: () = assert!(
    PREVIOUSLY_LOADED == cfg!(sk_user_config_already_loaded),
    "sk_user_config_already_loaded cfg must match native loaded-guard presence"
);

// Source: include/private/SkLoadUserConfig.h:10-13
// SkFeatures supplies native pre-user-header defaults; the actual selected
// user include can override/undef them in the AFTER_INCLUDE stream below.
pub use super::SkLogPriority::SkLogPriority;

// Source: include/private/SkLoadUserConfig.h:15-19
// Match native preprocessing arity, including empty arguments, without parsing,
// resolving or evaluating any disabled argument tokens.
#[cfg(not(sk_user_config_already_loaded))]
#[doc(hidden)]
#[macro_export]
macro_rules! __sk_nothing_arguments {
    (@one;) => {{}};
    (@one; , $($rest:tt)*) => {compile_error!("SK_NOTHING_ARG1 requires one preprocessor argument")};
    (@one; $head:tt $($rest:tt)*) => {$crate::__sk_nothing_arguments!(@one; $($rest)*)};
    (@two;) => {compile_error!("SK_NOTHING_ARG2 requires two preprocessor arguments")};
    (@two; , $($rest:tt)*) => {$crate::__sk_nothing_arguments!(@one; $($rest)*)};
    (@two; $head:tt $($rest:tt)*) => {$crate::__sk_nothing_arguments!(@two; $($rest)*)};
    (@three;) => {compile_error!("SK_NOTHING_ARG3 requires three preprocessor arguments")};
    (@three; , $($rest:tt)*) => {$crate::__sk_nothing_arguments!(@two; $($rest)*)};
    (@three; $head:tt $($rest:tt)*) => {$crate::__sk_nothing_arguments!(@three; $($rest)*)};
}
#[cfg(not(sk_user_config_already_loaded))]
#[macro_export]
macro_rules! SK_NOTHING_ARG1 {($($arguments:tt)*)=>{$crate::__sk_nothing_arguments!(@one; $($arguments)*)};}
#[cfg(not(sk_user_config_already_loaded))]
#[macro_export]
macro_rules! SK_NOTHING_ARG2 {($($arguments:tt)*)=>{$crate::__sk_nothing_arguments!(@two; $($arguments)*)};}
#[cfg(not(sk_user_config_already_loaded))]
#[macro_export]
macro_rules! SK_NOTHING_ARG3 {($($arguments:tt)*)=>{$crate::__sk_nothing_arguments!(@three; $($arguments)*)};}

// Source: include/private/SkLoadUserConfig.h:21-34
#[derive(Clone, Copy)]
pub enum UserConfigInclude {
    AlreadyLoaded,
    Explicit {
        header_tokens: &'static str,
        resolved_path: Option<&'static str>,
    },
    Bazel {
        header: &'static str,
        resolved_path: Option<&'static str>,
    },
    Default {
        header: &'static str,
        resolved_path: Option<&'static str>,
    },
}
const USER_CONFIG_HEADER: Option<&str> = definition(
    option_env!("SK_LOAD_USER_CONFIG_INPUT_SK_USER_CONFIG_HEADER"),
    option_env!("SK_USER_CONFIG_HEADER"),
);
const BAZEL: bool = flag(
    option_env!("SK_LOAD_USER_CONFIG_INPUT_SK_USE_BAZEL_CONFIG_HEADER"),
    option_env!("SK_USE_BAZEL_CONFIG_HEADER"),
    false,
);
const RESOLVED_HEADER_PATH: Option<&str> = option_env!("SK_LOAD_USER_CONFIG_SELECTED_HEADER_PATH");
pub const USER_CONFIG_INCLUDE: UserConfigInclude = if PREVIOUSLY_LOADED {
    UserConfigInclude::AlreadyLoaded
} else if let Some(header_tokens) = USER_CONFIG_HEADER {
    UserConfigInclude::Explicit {
        header_tokens,
        resolved_path: RESOLVED_HEADER_PATH,
    }
} else if BAZEL {
    UserConfigInclude::Bazel {
        header: "SkUserConfig.h",
        resolved_path: RESOLVED_HEADER_PATH,
    }
} else {
    UserConfigInclude::Default {
        header: "include/config/SkUserConfig.h",
        resolved_path: RESOLVED_HEADER_PATH,
    }
};
// The build bridge resolves this actual C++ include. Rust does not include or
// approximate an external C++ config as a made-up runtime provider.
const CHROMIUM_CONFIG: bool = match option_env!("SK_LOAD_USER_CONFIG_IS_CHROMIUM_CONFIG") {
    Some(value) => {
        if text_eq(value, b"1") {
            true
        } else {
            assert!(
                text_eq(value, b"0"),
                "Chromium selected-header metadata must be 0/1"
            );
            false
        }
    }
    None => false,
};
// Source: skia/config/SkUserConfig.h:30-33
const DCHECK_ALWAYS_ON: bool = flag(
    option_env!("SK_LOAD_USER_CONFIG_AFTER_INCLUDE_DCHECK_ALWAYS_ON"),
    option_env!("DCHECK_ALWAYS_ON"),
    false,
);
const FORCE_CHROMIUM_DEBUG: bool = !PREVIOUSLY_LOADED && CHROMIUM_CONFIG && DCHECK_ALWAYS_ON;
const USER_DEBUG: bool = FORCE_CHROMIUM_DEBUG
    || flag(
        option_env!("SK_LOAD_USER_CONFIG_AFTER_INCLUDE_SK_DEBUG"),
        option_env!("SK_DEBUG"),
        false,
    );
const USER_RELEASE: bool = !FORCE_CHROMIUM_DEBUG
    && flag(
        option_env!("SK_LOAD_USER_CONFIG_AFTER_INCLUDE_SK_RELEASE"),
        option_env!("SK_RELEASE"),
        false,
    );

// Source: include/private/SkLoadUserConfig.h:36-43
pub const NDEBUG_DEFINED: bool = flag(
    option_env!("SK_LOAD_USER_CONFIG_AFTER_INCLUDE_NDEBUG"),
    option_env!("NDEBUG"),
    false,
);
const NEED_DEBUG_RELEASE_DEFAULT: bool = !PREVIOUSLY_LOADED && !USER_DEBUG && !USER_RELEASE;
pub const SK_DEBUG: bool = USER_DEBUG || (NEED_DEBUG_RELEASE_DEFAULT && !NDEBUG_DEFINED);
pub const SK_RELEASE: bool = USER_RELEASE || (NEED_DEBUG_RELEASE_DEFAULT && NDEBUG_DEFINED);
// Source: include/private/SkLoadUserConfig.h:45-49
const _: () = if !PREVIOUSLY_LOADED {
    assert!(
        !(SK_DEBUG && SK_RELEASE),
        "cannot define both SK_DEBUG and SK_RELEASE"
    );
    assert!(
        SK_DEBUG || SK_RELEASE,
        "must define either SK_DEBUG or SK_RELEASE"
    );
};

// Source: include/private/SkLoadUserConfig.h:51-55
pub const SK_CPU_LENDIAN: bool = flag(
    option_env!("SK_LOAD_USER_CONFIG_AFTER_INCLUDE_SK_CPU_LENDIAN"),
    option_env!("SK_CPU_LENDIAN"),
    super::SkFeatures::SK_CPU_LENDIAN,
);
pub const SK_CPU_BENDIAN: bool = flag(
    option_env!("SK_LOAD_USER_CONFIG_AFTER_INCLUDE_SK_CPU_BENDIAN"),
    option_env!("SK_CPU_BENDIAN"),
    super::SkFeatures::SK_CPU_BENDIAN,
);
const _: () = if !PREVIOUSLY_LOADED {
    assert!(
        !(SK_CPU_LENDIAN && SK_CPU_BENDIAN),
        "cannot define both SK_CPU_LENDIAN and SK_CPU_BENDIAN"
    );
    assert!(
        SK_CPU_LENDIAN || SK_CPU_BENDIAN,
        "must define either SK_CPU_LENDIAN or SK_CPU_BENDIAN"
    );
};
// Source: include/private/SkLoadUserConfig.h:57-61
pub const I_ACKNOWLEDGE_SKIA_DOES_NOT_SUPPORT_BIG_ENDIAN: bool = flag(
    option_env!("SK_LOAD_USER_CONFIG_AFTER_INCLUDE_I_ACKNOWLEDGE_SKIA_DOES_NOT_SUPPORT_BIG_ENDIAN"),
    option_env!("I_ACKNOWLEDGE_SKIA_DOES_NOT_SUPPORT_BIG_ENDIAN"),
    false,
);
const _: () = if !PREVIOUSLY_LOADED && SK_CPU_BENDIAN {
    assert!(I_ACKNOWLEDGE_SKIA_DOES_NOT_SUPPORT_BIG_ENDIAN,
        "The Skia team is not endian-savvy enough to support big-endian CPUs.\nIf you still want to use Skia,\nplease define I_ACKNOWLEDGE_SKIA_DOES_NOT_SUPPORT_BIG_ENDIAN.");
};
// Source: include/private/SkLoadUserConfig.h:63-64
pub const SK_USER_CONFIG_WAS_LOADED: bool = true;

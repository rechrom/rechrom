#[path = "build_support/native_ostream_abi.rs"]
mod native_ostream_abi;

// Embedding configuration only; no Skia algorithm/source-row credit.
// Chromium skia/BUILD.gn selects skia/config/SkUserConfig.h. Resolve that
// config's pixel definitions for the Cargo target, rather than the build host.
fn main() {
    // Native SkAssert.h permits a preexisting embedding SK_ABORT definition.
    // Chromium supplies SkAbort_FileLine; opt-in generic fallback is separate.
    println!("cargo:rustc-check-cfg=cfg(skia_use_default_abort)");
    println!("cargo:rustc-check-cfg=cfg(skia_use_default_debugf)");
    println!("cargo:rustc-check-cfg=cfg(skia_use_default_log)");
    println!("cargo:rustc-check-cfg=cfg(sk_debug)");
    println!("cargo:rustc-check-cfg=cfg(dcheck_always_on)");
    println!("cargo:rustc-check-cfg=cfg(SK_WIN_PANOSE)");
    let browser_cpu = std::env::var_os("CARGO_FEATURE_BROWSER_CPU").is_some();
    let chromium_embedding = std::env::var_os("CARGO_FEATURE_CHROMIUM_EMBEDDING").is_some();
    assert!(
        browser_cpu ^ chromium_embedding,
        "select exactly one Skia embedding: browser_cpu or chromium_embedding"
    );
    if browser_cpu {
        // These are upstream Skia's actual default diagnostic ports. This
        // embedding has no Chromium base::logging owner, so its FileLine
        // override symbols must not be selected.
        println!("cargo:rustc-cfg=skia_use_default_abort");
        println!("cargo:rustc-cfg=skia_use_default_debugf");
        println!("cargo:rustc-cfg=skia_use_default_log");
    }
    println!("cargo:rerun-if-env-changed=SK_WIN_PANOSE");
    if std::env::var_os("SK_WIN_PANOSE").is_some() {
        println!("cargo:rustc-cfg=SK_WIN_PANOSE");
    }
    for name in [
        "sk_native_nonclang_msvc",
        "sk_native_gnu_before_13",
        "sk_native_no_assume",
    ] {
        println!("cargo:rustc-check-cfg=cfg({name})");
    }
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS");
    let native = bind_native_features(&target_os);
    bind_codec_configuration(browser_cpu);
    if chromium_embedding {
        bind_native_alias_translation_units();
        bind_chromium_build_inputs();
        bind_native_location_inputs();
        bind_native_time_inputs();
        bind_native_numeric_inputs();
        bind_native_ostream_storage();
    }
    bind_native_primitive_archive(
        "LONG_DOUBLE",
        "sk_native_long_double_ffi",
        "skia_native_long_double",
    );
    bind_sktypes_configuration();
    let memory = bind_memory_configuration();
    for name in [
        "SK_R32_SHIFT",
        "SK_B32_SHIFT",
        "SK_CPU_BENDIAN",
        "SK_CPU_LENDIAN",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }

    // Embedding source: chromium/skia/config/SkUserConfig.h:79-84.
    // The Chromium Android target uses Skia's generic defaults. Every other
    // target gets the user-config definitions, overriding earlier definitions.
    if !native.android_defined {
        println!("cargo:rustc-env=SK_R32_SHIFT=16");
        println!("cargo:rustc-env=SK_B32_SHIFT=0");
    }

    // Embedding source: chromium/skia/config/SkUserConfig.h:86-89.
    // Represent the real #undef as an explicit resolved absence. A raw rustc
    // environment variable cannot express undef by being set to an empty string.
    if native.build_mac {
        println!("cargo:rustc-env=SKIA_RESOLVED_CPU_LENDIAN=1");
        println!("cargo:rustc-env=SKIA_RESOLVED_CPU_BENDIAN=0");
    } else {
        // Embedding source: chromium/skia/config/SkUserConfig.h:98-106.
        if native.build_unix && native.pre_user_big_endian {
            panic!("Chromium SkUserConfig.h rejects Unix big-endian pixel configuration");
        }
    }
    let user = bind_loaded_user_configuration(&native);
    bind_compiler_port_inputs(&target_os, user.debug, &memory);
}

// Actual SkLoadUserConfig.h input stream, after the selected user header.
// A defined-zero flag is defined. Neither the build host nor Rust's profile
// silently overrides an explicit native SK_RELEASE or NDEBUG input.
struct LoadedUserConfiguration {
    debug: bool,
}

fn bind_loaded_user_configuration(native: &NativeBindings) -> LoadedUserConfiguration {
    fn read(name: &str) -> Option<String> {
        println!("cargo:rerun-if-env-changed={name}");
        std::env::var(name).ok()
    }
    fn flag(prefix: &str, name: &str, complete: bool, fallback: bool) -> bool {
        let key = format!("{prefix}{name}");
        let supplied = read(&key).or_else(|| if complete { None } else { read(name) });
        let defined = supplied
            .as_deref()
            .map_or(!complete && fallback, |value| value != "undefined");
        println!(
            "cargo:rustc-env={key}={}",
            if defined { "1" } else { "undefined" }
        );
        defined
    }
    let complete = read("SK_LOAD_USER_CONFIG_COMPLETE_INPUTS").is_some();
    let pre = "SK_LOAD_USER_CONFIG_INPUT_";
    let post = "SK_LOAD_USER_CONFIG_AFTER_INCLUDE_";
    let loaded = flag(pre, "SK_USER_CONFIG_WAS_LOADED", complete, false);
    let bazel = flag(pre, "SK_USE_BAZEL_CONFIG_HEADER", complete, false);
    let header_key = "SK_LOAD_USER_CONFIG_INPUT_SK_USER_CONFIG_HEADER";
    let header = read(header_key)
        .or_else(|| {
            if complete {
                None
            } else {
                read("SK_USER_CONFIG_HEADER")
            }
        })
        .unwrap_or_else(|| {
            if complete || bazel {
                "undefined".into()
            } else {
                // Actual Chromium skia/BUILD.gn:64-69.
                "\"../../skia/config/SkUserConfig.h\"".into()
            }
        });
    println!("cargo:rustc-env={header_key}={header}");
    let chromium = read("SK_LOAD_USER_CONFIG_IS_CHROMIUM_CONFIG").map_or(
        header == "\"../../skia/config/SkUserConfig.h\"",
        |value| {
            assert!(
                value == "0" || value == "1",
                "selected-header metadata must be resolved 0/1"
            );
            value == "1"
        },
    );
    println!(
        "cargo:rustc-env=SK_LOAD_USER_CONFIG_IS_CHROMIUM_CONFIG={}",
        if chromium { 1 } else { 0 }
    );
    if let Some(path) = read("SK_LOAD_USER_CONFIG_SELECTED_HEADER_PATH") {
        println!("cargo:rustc-env=SK_LOAD_USER_CONFIG_SELECTED_HEADER_PATH={path}");
    } else if chromium {
        println!(
            "cargo:rustc-env=SK_LOAD_USER_CONFIG_SELECTED_HEADER_PATH=skia/config/SkUserConfig.h"
        );
    }
    let dcheck = flag(post, "DCHECK_ALWAYS_ON", complete, false);
    let supplied_debug = flag(post, "SK_DEBUG", complete, false);
    let supplied_release = flag(post, "SK_RELEASE", complete, false);
    let ndebug = flag(post, "NDEBUG", complete, false);
    let force_debug = !loaded && chromium && dcheck;
    let user_debug = force_debug || supplied_debug;
    let user_release = !force_debug && supplied_release;
    // Embedding source: skia/config/SkUserConfig.h:53-57. This include
    // selection precedes SkLoadUserConfig.h's later SK_DEBUG defaulting.
    // In particular, a later default debug flag cannot retroactively select
    // the debug mixin instead of the release header already included.
    let mixin_key = "SK_LOAD_USER_CONFIG_AFTER_INCLUDE_SK_REF_CNT_MIXIN_INCLUDE";
    let supplied_mixin = read(mixin_key).or_else(|| {
        if complete {
            None
        } else {
            read("SK_REF_CNT_MIXIN_INCLUDE")
        }
    });
    let mixin_header = supplied_mixin.unwrap_or_else(|| {
        if !loaded && chromium {
            if user_debug {
                "\"skia/config/sk_ref_cnt_ext_debug.h\"".into()
            } else {
                "\"skia/config/sk_ref_cnt_ext_release.h\"".into()
            }
        } else {
            "undefined".into()
        }
    });
    let mixin = match mixin_header.as_str() {
        "\"skia/config/sk_ref_cnt_ext_debug.h\"" => "chromium_debug",
        "\"skia/config/sk_ref_cnt_ext_release.h\"" => "chromium_release",
        "undefined" => "none",
        _ => panic!("unsupported actual SK_REF_CNT_MIXIN_INCLUDE: {mixin_header}"),
    };
    println!("cargo:rustc-env={mixin_key}={mixin_header}");
    println!("cargo:rustc-check-cfg=cfg(sk_ref_cnt_mixin, values(\"chromium_debug\", \"chromium_release\", \"none\"))");
    println!("cargo:rustc-cfg=sk_ref_cnt_mixin=\"{mixin}\"");
    let google3 = flag(post, "SK_BUILD_FOR_GOOGLE3", complete, false);
    println!("cargo:rustc-check-cfg=cfg(sk_build_for_google3)");
    if google3 {
        println!("cargo:rustc-cfg=sk_build_for_google3");
    }
    let default_needed = !loaded && !user_debug && !user_release;
    let debug = user_debug || (default_needed && !ndebug);
    let release = user_release || (default_needed && ndebug);
    if !loaded {
        assert!(
            !(debug && release),
            "cannot define both SK_DEBUG and SK_RELEASE"
        );
        assert!(
            debug || release,
            "must define either SK_DEBUG or SK_RELEASE"
        );
    }
    // These are effective post-user-header flags, not compiler byte order.
    let post_endian = |name: &str, fallback: bool| {
        let key = format!("{post}{name}");
        // Legacy raw flags were already applied by SkFeatures. The Chromium
        // Mac user header then defines little and explicitly undefines big;
        // do not resurrect that earlier raw big flag after the #undef.
        let defined = read(&key)
            .as_deref()
            .map_or(!complete && fallback, |value| value != "undefined");
        println!(
            "cargo:rustc-env={key}={}",
            if defined { "1" } else { "undefined" }
        );
        defined
    };
    let little = post_endian(
        "SK_CPU_LENDIAN",
        (chromium && native.build_mac) || native.pre_user_little_endian,
    );
    let big = post_endian(
        "SK_CPU_BENDIAN",
        !(chromium && native.build_mac) && native.pre_user_big_endian,
    );
    flag(
        post,
        "I_ACKNOWLEDGE_SKIA_DOES_NOT_SUPPORT_BIG_ENDIAN",
        complete,
        false,
    );
    if !loaded {
        assert!(
            !(little && big),
            "cannot define both SK_CPU_LENDIAN and SK_CPU_BENDIAN"
        );
        assert!(
            little || big,
            "must define either SK_CPU_LENDIAN or SK_CPU_BENDIAN"
        );
    }
    println!(
        "cargo:rustc-env=SKIA_RESOLVED_CPU_LENDIAN={}",
        if little { 1 } else { 0 }
    );
    println!(
        "cargo:rustc-env=SKIA_RESOLVED_CPU_BENDIAN={}",
        if big { 1 } else { 0 }
    );
    for (name, enabled) in [
        ("sk_user_config_already_loaded", loaded),
        ("sk_debug", debug),
        ("sk_release", release),
        ("sk_native_ndebug", ndebug),
        ("dcheck_always_on", dcheck),
    ] {
        println!("cargo:rustc-check-cfg=cfg({name})");
        if enabled {
            println!("cargo:rustc-cfg={name}");
        }
    }
    LoadedUserConfiguration { debug }
}

// Transport actual resolved user-config values; never evaluate an arbitrary
// C++ macro expression with a made-up Rust expression parser. This bridge is
// embedding input, not implementation credit for Skia's source algorithms.
fn bind_sktypes_configuration() {
    fn input(name: &str) -> Option<String> {
        println!("cargo:rerun-if-env-changed={name}");
        std::env::var(name).ok()
    }
    fn decimal_i32(raw: &str, resolved: &str) {
        let original = input(raw);
        let value = input(resolved).or_else(|| original.filter(|value| value != "undefined"));
        if let Some(value) = value {
            let value: i32 = value.parse().unwrap_or_else(|_| {
                panic!("{raw} requires its actual resolved signed decimal value in {resolved}")
            });
            println!("cargo:rustc-env={resolved}={value}");
        }
    }
    decimal_i32(
        "SK_ALLOW_STATIC_GLOBAL_INITIALIZERS",
        "SKIA_RESOLVED_ALLOW_STATIC_GLOBAL_INITIALIZERS_I32",
    );
    decimal_i32("GR_CACHE_STATS", "SKIA_RESOLVED_GR_CACHE_STATS_I32");
    decimal_i32("GR_GPU_STATS", "SKIA_RESOLVED_GR_GPU_STATS_I32");
    for (raw, resolved) in [
        ("SK_GAMMA_EXPONENT", "SKIA_RESOLVED_GAMMA_EXPONENT_BITS"),
        ("SK_GAMMA_CONTRAST", "SKIA_RESOLVED_GAMMA_CONTRAST_BITS"),
    ] {
        let original = input(raw);
        if let Some(value) = input(resolved) {
            let bits: u32 = value.parse().unwrap_or_else(|_| {
                panic!("{resolved} must be the actual resolved f32 bit pattern as decimal u32")
            });
            println!("cargo:rustc-env={resolved}={bits}");
        } else if original
            .as_deref()
            .is_some_and(|value| value != "undefined")
        {
            panic!("{raw} requires its actual resolved f32 bit pattern in {resolved}");
        }
    }
    // Chromium SkUserConfig.h:26 includes skia/ext/skia_histogram.h, which
    // actually defines all seven macros. Explicit resolved final presences
    // describe another effective embedding configuration; text "0" is absence
    // only in this resolved presence transport, not C++ #ifdef semantics.
    for suffix in [
        "BOOLEAN",
        "ENUMERATION",
        "EXACT_LINEAR",
        "CUSTOM_EXACT_LINEAR",
        "MEMORY_KB",
        "CUSTOM_COUNTS",
        "CUSTOM_MICROSECONDS_TIMES",
    ] {
        let name = format!("SKIA_RESOLVED_HISTOGRAM_{suffix}_DEFINED");
        let value = input(&name).unwrap_or_else(|| "1".into());
        assert!(
            value == "0" || value == "1",
            "{name} must be resolved 0 or 1"
        );
        println!("cargo:rustc-env={name}={value}");
    }
}

// Embedding transport only, not another runtime feature detector. Cargo's
// selected target supplies the compiler-predefine equivalents used by the
// source-aligned SkFeatures constants. Never use this build program's host cfg.
struct NativeBindings {
    android_defined: bool,
    build_mac: bool,
    build_unix: bool,
    pre_user_big_endian: bool,
    pre_user_little_endian: bool,
}

fn bind_native_features(target_os: &str) -> NativeBindings {
    use std::collections::BTreeMap;
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo target arch");
    let vendor = std::env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or_default();
    let environment = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let abi = std::env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default();
    let features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let endian = std::env::var("CARGO_CFG_TARGET_ENDIAN").expect("Cargo target endian");
    let has = |name: &str| features.split(',').any(|feature| feature == name);
    let complete = std::env::var_os("SK_FEATURES_COMPLETE_NATIVE_INPUTS").is_some();
    println!("cargo:rerun-if-env-changed=SK_FEATURES_COMPLETE_NATIVE_INPUTS");
    println!("cargo:rerun-if-env-changed=SK_FEATURES_CXX_COMPILER");
    for name in "SK_BUILD_FOR_ANDROID SK_BUILD_FOR_IOS SK_BUILD_FOR_WIN SK_BUILD_FOR_UNIX SK_BUILD_FOR_MAC SK_BUILD_FOR_WASM __APPLE__ _WIN32 __SYMBIAN32__ ANDROID __ANDROID__ __EMSCRIPTEN__ linux __linux __FreeBSD__ __OpenBSD__ __sun __NetBSD__ __DragonFly__ __Fuchsia__ __GLIBC__ __GNU__ __unix__ TARGET_OS_IPHONE TARGET_IPHONE_SIMULATOR SK_RESTRICT __clang__ SK_CPU_BENDIAN SK_CPU_LENDIAN __BYTE_ORDER__ __ORDER_BIG_ENDIAN__ __ORDER_LITTLE_ENDIAN__ __sparc __sparc__ _POWER __powerpc__ __ppc__ __hppa __PPC__ __PPC64__ _MIPSEB __ARMEB__ __s390__ __sh__ __BIG_ENDIAN__ __ia64 __i386 _M_IX86 __x86_64__ _M_X64 _M_AMD64 _M_IX86_FP _MSC_VER __loongarch__ __loongarch64 __powerpc64__ SK_CPU_X86 SK_CPU_LOONGARCH SK_CPU_PPC SK_CPU_X64_LEVEL SK_CPU_LSX_LEVEL __AVX512F__ __AVX512DQ__ __AVX512CD__ __AVX512BW__ __AVX512VL__ __AVX2__ __AVX__ __SSE4_2__ __SSE4_1__ __SSSE3__ __SSE3__ __SSE2__ __loongarch_asx __loongarch_sx __arm__ __aarch64__ SK_CPU_ARM32 SK_CPU_ARM64 SK_ARM_HAS_NEON __ARM_NEON __ARM_VFPV3__".split_whitespace() {
        println!("cargo:rerun-if-env-changed=SK_FEATURES_INPUT_{name}");
    }
    // Chromium build/config/BUILDCONFIG.gn:138-140. MIPS CPU spelling is a GN
    // input and cannot be recovered from Rust's architecture name alone.
    let compiler = std::env::var("SK_FEATURES_CXX_COMPILER").unwrap_or_else(|_| {
        if complete {
            return "other".into();
        }
        assert!(
            !(target_os == "linux" && matches!(arch.as_str(), "mips" | "mips64")),
            "MIPS needs the actual Chromium C++ compiler input"
        );
        "clang".into()
    });
    println!("cargo:rustc-env=SK_FEATURES_CXX_COMPILER={compiler}");
    let gnu = matches!(compiler.as_str(), "clang" | "gcc");
    let msvc = compiler == "msvc";
    let mut inputs: BTreeMap<String, String> = std::env::vars()
        .filter_map(|(name, value)| {
            name.strip_prefix("SK_FEATURES_INPUT_")
                .map(|name| (name.to_owned(), value))
        })
        .collect();
    // Preserve existing explicit native inputs, including defined-zero. The
    // prefix transport takes precedence over the older direct macro transport.
    for name in [
        "SK_BUILD_FOR_ANDROID",
        "SK_BUILD_FOR_IOS",
        "SK_BUILD_FOR_WIN",
        "SK_BUILD_FOR_UNIX",
        "SK_BUILD_FOR_MAC",
        "SK_BUILD_FOR_WASM",
        "SK_CPU_BENDIAN",
        "SK_CPU_LENDIAN",
        "SK_CPU_X64_LEVEL",
        "SK_CPU_LSX_LEVEL",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        if let Ok(value) = std::env::var(name) {
            inputs.entry(name.into()).or_insert(value);
        }
    }
    // Actual SDK TargetConditionals.h:231-248,524-525: the macOS target's
    // non-iPhone/non-simulator branch. Other Apple SDK targets require their
    // own actual input, rather than inferring a simulator from the build host.
    if target_os == "macos" && !complete {
        inputs
            .entry("TARGET_OS_IPHONE".into())
            .or_insert_with(|| "0".into());
        inputs
            .entry("TARGET_IPHONE_SIMULATOR".into())
            .or_insert_with(|| "0".into());
    }
    // Embedding source: build/config/android/BUILD.gn:25-26. This is the
    // Chromium ANDROID define, distinct from compiler-defined __ANDROID__.
    if target_os == "android" && !complete {
        inputs.entry("ANDROID".into()).or_insert_with(|| "1".into());
    }
    for (name, value) in &inputs {
        println!("cargo:rerun-if-env-changed=SK_FEATURES_INPUT_{name}");
        println!("cargo:rustc-env=SK_FEATURES_INPUT_{name}={value}");
    }
    let flag = |name: &str, fallback: bool| {
        inputs
            .get(name)
            .map_or(!complete && fallback, |value| value != "undefined")
    };
    let integer = |name: &str| {
        inputs.get(name).and_then(|value| {
            if value == "undefined" {
                None
            } else {
                Some(
                    value
                        .parse::<i32>()
                        .expect("resolved native signed decimal input"),
                )
            }
        })
    };
    let conditional = |name: &str| {
        if !inputs.contains_key(name) && !complete && flag("__APPLE__", vendor == "apple") {
            panic!("actual Apple SDK TargetConditionals input required: {name}");
        }
        integer(name).unwrap_or(0) != 0
    };
    let overrides = [
        "SK_BUILD_FOR_ANDROID",
        "SK_BUILD_FOR_IOS",
        "SK_BUILD_FOR_WIN",
        "SK_BUILD_FOR_UNIX",
        "SK_BUILD_FOR_MAC",
    ]
    .map(|name| flag(name, false));
    let selected = if overrides.iter().any(|value| *value) {
        "override"
    } else if flag("_WIN32", target_os == "windows") || flag("__SYMBIAN32__", false) {
        "win"
    } else if flag("ANDROID", false) || flag("__ANDROID__", target_os == "android") {
        "android"
    } else if flag("__EMSCRIPTEN__", target_os == "emscripten") {
        "wasm"
    } else if [
        ("linux", false),
        ("__linux", target_os == "linux"),
        ("__FreeBSD__", target_os == "freebsd"),
        ("__OpenBSD__", target_os == "openbsd"),
        ("__sun", matches!(target_os, "solaris" | "illumos")),
        ("__NetBSD__", target_os == "netbsd"),
        ("__DragonFly__", target_os == "dragonfly"),
        ("__Fuchsia__", target_os == "fuchsia"),
        ("__GLIBC__", false),
        ("__GNU__", false),
        ("__unix__", false),
    ]
    .iter()
    .any(|(name, fallback)| flag(name, *fallback))
    {
        "unix"
    } else if conditional("TARGET_OS_IPHONE") || conditional("TARGET_IPHONE_SIMULATOR") {
        "ios"
    } else {
        "mac"
    };
    let build_win = overrides[2] || selected == "win";
    let build_ios = overrides[1] || selected == "ios";
    bind_cfstr_configuration(build_ios);
    let build_mac = overrides[4] || selected == "mac";
    let build_unix = overrides[3] || selected == "unix";
    // The same pre-user-config source condition as SkFeatures.h:49-66.
    // Explicit defined-zero endian flags still suppress automatic selection.
    let byte_order_match = |big: bool| {
        if let Some(raw) = integer("__BYTE_ORDER__") {
            let name = if big {
                "__ORDER_BIG_ENDIAN__"
            } else {
                "__ORDER_LITTLE_ENDIAN__"
            };
            assert!(
                !gnu || complete || inputs.contains_key(name),
                "actual {name} input required with explicit __BYTE_ORDER__"
            );
            raw == integer(name).unwrap_or(0)
        } else if flag("__BYTE_ORDER__", gnu) {
            endian == if big { "big" } else { "little" }
        } else {
            false
        }
    };
    let big_override = flag("SK_CPU_BENDIAN", false);
    let little_override = flag("SK_CPU_LENDIAN", false);
    let pre_user_big_endian = big_override
        || (!little_override
            && if byte_order_match(true) {
                true
            } else if byte_order_match(false) {
                false
            } else {
                [
                    ("__sparc", false),
                    (
                        "__sparc__",
                        gnu && matches!(arch.as_str(), "sparc" | "sparc64"),
                    ),
                    ("_POWER", false),
                    (
                        "__powerpc__",
                        gnu && matches!(arch.as_str(), "powerpc" | "powerpc64"),
                    ),
                    ("__ppc__", false),
                    ("__hppa", false),
                    ("__PPC__", false),
                    ("__PPC64__", false),
                    ("_MIPSEB", false),
                    ("__ARMEB__", false),
                    ("__s390__", gnu && arch == "s390x"),
                ]
                .into_iter()
                .any(|(name, fallback)| flag(name, fallback))
                    || ((flag("__sh__", false) || flag("__ia64", false))
                        && flag("__BIG_ENDIAN__", false))
            });
    let avx512 = ["avx512f", "avx512dq", "avx512cd", "avx512bw", "avx512vl"]
        .iter()
        .all(|feature| {
            flag(
                &format!("__{}__", feature.to_uppercase()),
                gnu && has(feature),
            )
        });
    let first = integer("SK_CPU_X64_LEVEL").or_else(|| {
        if avx512 {
            Some(60)
        } else {
            [
                ("__AVX2__", "avx2", 52),
                ("__AVX__", "avx", 51),
                ("__SSE4_2__", "sse4.2", 42),
                ("__SSE4_1__", "sse4.1", 41),
                ("__SSSE3__", "ssse3", 31),
                ("__SSE3__", "sse3", 30),
                ("__SSE2__", "sse2", 20),
            ]
            .into_iter()
            .find_map(|(name, feature, level)| flag(name, gnu && has(feature)).then_some(level))
        }
    });
    let level = first.or_else(|| {
        if flag("_M_X64", msvc && arch == "x86_64") || flag("_M_AMD64", msvc && arch == "x86_64") {
            Some(20)
        } else {
            match integer("_M_IX86_FP") {
                Some(n) if n >= 2 => Some(20),
                Some(1) => Some(10),
                _ => None,
            }
        }
    });
    let arm32_detected = flag("__arm__", gnu && arch == "arm")
        && (!flag("__APPLE__", vendor == "apple") || !conditional("TARGET_IPHONE_SIMULATOR"));
    let arm32 = flag("SK_CPU_ARM32", false) || arm32_detected;
    let arm64 = flag("SK_CPU_ARM64", false)
        || (!arm32_detected && flag("__aarch64__", gnu && arch == "aarch64"));
    let loongarch = flag("SK_CPU_LOONGARCH", false)
        || flag("__loongarch__", gnu && arch == "loongarch64")
        || flag("__loongarch64", gnu && arch == "loongarch64");
    // clang-cl defines _MSC_VER as well as __clang__. Compiler family alone
    // therefore does not select the native stage ABI.
    let msc_ver = flag(
        "_MSC_VER",
        msvc || (compiler == "clang" && environment == "msvc"),
    );
    for (name, enabled) in [
        ("SK_BUILD_FOR_IOS", build_ios),
        ("sk_features_build_for_mac", build_mac),
        ("sk_features_build_for_ios", build_ios),
        ("sk_features_build_for_win", build_win),
        ("sk_features_x64_ge_sse2", level.unwrap_or(0) >= 20),
        ("skrp_native_target_inputs_configured", true),
        ("skrp_target_arm32_defined", arm32),
        ("skrp_target_msc_ver_defined", msc_ver),
        (
            "skrp_target_x86_64_defined",
            flag("__x86_64__", gnu && arch == "x86_64"),
        ),
        ("skrp_target_arm64_defined", arm64),
        ("skrp_target_loongarch_defined", loongarch),
        (
            "skrp_target_c_abi_aapcs_vfp",
            arch == "arm" && abi == "eabihf",
        ),
        ("sk_native_clang", flag("__clang__", compiler == "clang")),
        (
            "sk_native_nonclang_msvc",
            msc_ver && !flag("__clang__", compiler == "clang"),
        ),
        ("sk_native_arm_vfpv3", flag("__ARM_VFPV3__", false)),
    ] {
        println!("cargo:rustc-check-cfg=cfg({name})");
        if enabled {
            println!("cargo:rustc-cfg={name}");
        }
    }
    NativeBindings {
        android_defined: flag("ANDROID", false),
        build_mac,
        build_unix,
        pre_user_big_endian,
        pre_user_little_endian: little_override || (!big_override && !pre_user_big_endian),
    }
}

// Transport the selected SDK/compiler CFSTR expansion. This is a resolved
// native ABI input, not a choice inferred from the Cargo host or profile.
// The builtin mode denotes Objective-C runtime, unsigned isa, Mach-O and
// __CONSTANT_CFSTRINGS__ defined; function denotes the SDK's nonbuiltin branch.
fn bind_cfstr_configuration(build_ios: bool) {
    println!("cargo:rustc-check-cfg=cfg(sk_cfstr_mode, values(\"builtin_objc_unsigned_isa\", \"function\"))");
    println!("cargo:rerun-if-env-changed=SKIA_RESOLVED_CFSTR_MODE");
    let mode = std::env::var("SKIA_RESOLVED_CFSTR_MODE").ok();
    if !build_ios {
        return;
    }
    match mode.as_deref() {
        Some("builtin_objc_unsigned_isa") | Some("function") => {
            println!("cargo:rustc-cfg=sk_cfstr_mode=\"{}\"", mode.unwrap());
        }
        _ => panic!("iOS requires SKIA_RESOLVED_CFSTR_MODE from the actual SDK/compiler CFSTR expansion; unsupported runtime/isa layouts must not select an approximate ABI"),
    }
}

// Source: src/codec/SkCodec.cpp:34-140. The native decoder list is selected by
// actual preprocessor definedness, including defined-zero flags. Missing
// configuration is explicitly distinguishable from a genuinely empty native
// decoder selection; the codec module must not treat it as an empty registry.
fn bind_codec_configuration(browser_cpu: bool) {
    println!("cargo:rustc-check-cfg=cfg(sk_codec_native_inputs_configured)");
    println!("cargo:rustc-check-cfg=cfg(cxxbridge1)");
    println!("cargo:rerun-if-env-changed=SK_CODEC_COMPLETE_NATIVE_INPUTS");
    let complete = std::env::var("SK_CODEC_COMPLETE_NATIVE_INPUTS")
        .ok()
        .or_else(|| browser_cpu.then(|| "1".to_owned()));
    if let Some(value) = complete.as_deref() {
        assert!(
            value == "1",
            "SK_CODEC_COMPLETE_NATIVE_INPUTS must be the resolved complete-input marker 1"
        );
    }
    for name in [
        "SK_DISABLE_LEGACY_INIT_DECODERS",
        "SK_CODEC_DECODES_AVIF",
        "SK_CODEC_DECODES_BMP_WITH_RUST",
        "SK_CODEC_DECODES_BMP",
        "SK_CODEC_DECODES_GIF",
        "SK_HAS_WUFFS_LIBRARY",
        "SK_CODEC_DECODES_ICO_WITH_RUST",
        "SK_CODEC_DECODES_ICO",
        "SK_CODEC_DECODES_JPEG_WITH_RUST",
        "SK_CODEC_DECODES_JPEG",
        "SK_CODEC_DECODES_JPEGXL",
        "SK_CODEC_DECODES_PNG_WITH_LIBPNG",
        "SK_CODEC_DECODES_PNG_WITH_RUST",
        "SK_CODEC_DECODES_RAW",
        "SK_CODEC_DECODES_WBMP",
        "SK_CODEC_DECODES_WEBP",
        "SK_CODEC_COLOR_PROFILE_PARSE_WITH_RUST",
    ] {
        println!("cargo:rustc-check-cfg=cfg({name})");
        let key = format!("SK_CODEC_INPUT_{name}");
        println!("cargo:rerun-if-env-changed={key}");
        if complete.is_some() {
            let value = std::env::var(&key).unwrap_or_else(|_| {
                if browser_cpu {
                    // This embedding intentionally has no Skia decoder list:
                    // image_decoder owns encoded bytes and supplies decoded
                    // images to the raster boundary.
                    "undefined".to_owned()
                } else {
                    panic!("complete codec inputs require actual {key}, with undefined for an absent macro")
                }
            });
            println!("cargo:rustc-env={key}={value}");
            if value != "undefined" {
                println!("cargo:rustc-cfg={name}");
                if name == "SK_CODEC_COLOR_PROFILE_PARSE_WITH_RUST" {
                    // Actual rust/icc/BUILD.bazel ffi_rs rustc_flags.
                    println!("cargo:rustc-cfg=cxxbridge1");
                }
            }
        }
    }
    if complete.is_some() {
        println!("cargo:rustc-cfg=sk_codec_native_inputs_configured");
    }
}

// Native memory-port compiler/platform macro transport. These values are
// embedding inputs and contribute no Skia implementation credit.
struct NativeMemoryBindings {
    bionic: bool,
    android_framework: bool,
}

fn bind_memory_configuration() -> NativeMemoryBindings {
    fn input(name: &str) -> Option<String> {
        let key = format!("SK_MEMORY_INPUT_{name}");
        println!("cargo:rerun-if-env-changed={key}");
        println!("cargo:rerun-if-env-changed={name}");
        std::env::var(&key)
            .ok()
            .or_else(|| std::env::var(name).ok())
    }
    let mut bionic = false;
    let mut android_framework = false;
    for (name, cfg) in [
        ("__BIONIC__", "sk_bionic"),
        (
            "SK_BUILD_FOR_ANDROID_FRAMEWORK",
            "sk_build_for_android_framework",
        ),
        ("SK_BUILD_FOR_AFL_FUZZ", "SK_BUILD_FOR_AFL_FUZZ"),
    ] {
        let defined = input(name).is_some_and(|value| value != "undefined");
        if name == "__BIONIC__" {
            bionic = defined;
        }
        if name == "SK_BUILD_FOR_ANDROID_FRAMEWORK" {
            android_framework = defined;
        }
        println!("cargo:rustc-check-cfg=cfg({cfg})");
        if defined {
            println!("cargo:rustc-cfg={cfg}");
        }
    }
    println!("cargo:rustc-check-cfg=cfg(sk_android_api_at_least_17)");
    if let Some(value) = input("__ANDROID_API__").filter(|value| value != "undefined") {
        let value: i32 = value
            .parse()
            .expect("actual resolved __ANDROID_API__ decimal value");
        if value >= 17 {
            println!("cargo:rustc-cfg=sk_android_api_at_least_17");
        }
    }
    NativeMemoryBindings {
        bionic,
        android_framework,
    }
}

// Supply the actual selected compiler/SDK macro values used by the translated
// native memory port. This transports inputs, never probes a compiler or SDK.
fn bind_compiler_port_inputs(target_os: &str, debug: bool, memory: &NativeMemoryBindings) {
    fn input(name: &str) -> Option<String> {
        println!("cargo:rerun-if-env-changed={name}");
        std::env::var(name).ok()
    }
    fn macro_input(name: &str) -> Option<String> {
        input(&format!("SK_MEMORY_INPUT_{name}"))
            .or_else(|| input(name))
            .filter(|value| value != "undefined")
    }
    let mut generated =
        String::from("// Actual resolved native compiler/SDK inputs; no source credit.\n");
    if target_os == "windows" && debug {
        let raw = macro_input("FAST_FAIL_FATAL_APP_EXIT");
        let resolved = input("SKIA_RESOLVED_FAST_FAIL_FATAL_APP_EXIT_U32");
        // Source: src/ports/SkMemory_malloc.cpp:27-29. Only actual absence
        // selects 7. Defined zero is retained; arbitrary C++ expressions need
        // their genuine resolved u32 result rather than an invented parser.
        let code = match resolved.or(raw) {
            Some(value) => value.parse::<u32>().expect(
                "FAST_FAIL_FATAL_APP_EXIT needs actual resolved decimal u32 in SKIA_RESOLVED_FAST_FAIL_FATAL_APP_EXIT_U32"),
            None => 7,
        };
        generated.push_str(&format!("pub const FAST_FAIL_FATAL_APP_EXIT:u32={code};\n"));
    }
    if memory.android_framework && memory.bionic {
        let raw = macro_input("M_THREAD_DISABLE_MEM_INIT");
        let resolved = input("SKIA_RESOLVED_M_THREAD_DISABLE_MEM_INIT_I32");
        // This macro must exist in the ACTUAL selected Bionic SDK. The primary
        // header is evidence, not a fallback for a different or older SDK.
        let value = resolved.or(raw).expect(
            "actual Bionic SDK M_THREAD_DISABLE_MEM_INIT is required in SKIA_RESOLVED_M_THREAD_DISABLE_MEM_INIT_I32");
        let option = value
            .parse::<i32>()
            .expect("M_THREAD_DISABLE_MEM_INIT needs its actual resolved signed decimal i32 value");
        generated.push_str(&format!(
            "pub const M_THREAD_DISABLE_MEM_INIT:core::ffi::c_int={option};\n"
        ));
    }
    let destination = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo OUT_DIR"))
        .join("sk_native_compiler_port_inputs.rs");
    std::fs::write(destination, generated).expect("write actual native compiler port input module");
}

// Actual base/debug/alias.cc and PA alias.cc are deliberately separate TUs.
// Keep their bodies opaque to callers even when the main Rust crate uses LTO.
// This only writes the build recipe; no compiler/archiver probe is performed.
fn bind_native_alias_translation_units() {
    use std::path::PathBuf;
    use std::process::Command;
    println!("cargo:rustc-check-cfg=cfg(skia_alias_separate_tu)");
    println!("cargo:rustc-check-cfg=cfg(pa_cpp_abi, values(\"itanium\", \"msvc\"))");
    for key in [
        "SKIA_LINK_NATIVE_ALIAS",
        "SKIA_NATIVE_ALIAS_ABI",
        "SKIA_NATIVE_ALIAS_ARCHIVER",
        "SKIA_NATIVE_ALIAS_ARCHIVER_STYLE",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    match std::env::var("SKIA_LINK_NATIVE_ALIAS").ok().as_deref() {
        None | Some("0") => return,
        Some("1") => {}
        _ => panic!("SKIA_LINK_NATIVE_ALIAS is an explicit resolved 0/1 build selection"),
    }
    let abi = std::env::var("SKIA_NATIVE_ALIAS_ABI")
        .expect("actual selected native C++ ABI required for Alias exports");
    assert!(
        abi == "itanium",
        "Alias export binding for this actual C++ ABI remains untranslated"
    );
    println!("cargo:rustc-cfg=pa_cpp_abi=\"itanium\"");
    let target = std::env::var("TARGET").expect("Cargo selected target");
    let rustc = std::env::var_os("RUSTC").expect("Cargo Rust compiler");
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    let mut objects = Vec::new();
    for (name, source) in [
        ("skia_base_alias", "base/debug/alias.rs"),
        ("skia_pa_alias", "base/allocator/partition_allocator/src/partition_alloc/partition_alloc_base/debug/alias.rs"),
    ] {
        println!("cargo:rerun-if-changed={source}");
        let object = output.join(format!("{name}.o"));
        let status = Command::new(&rustc)
            .args(["--crate-type=lib", "--edition=2024", "--crate-name", name,
                "--target", &target, "--emit=obj", "--cfg", "skia_alias_separate_tu",
                "--cfg", "pa_cpp_abi=\"itanium\"", "-C", "lto=off",
                "-C", "embed-bitcode=no", "-C", "codegen-units=1", "-C", "opt-level=2"])
            .arg(source).arg("-o").arg(&object)
            .status().expect("invoke selected Rust compiler for separate Alias TU");
        assert!(status.success(), "separate Alias translation unit did not compile");
        objects.push(object);
    }
    let archiver = std::env::var_os("SKIA_NATIVE_ALIAS_ARCHIVER")
        .expect("actual target native-object archiver required; no host-tool inference");
    let style = std::env::var("SKIA_NATIVE_ALIAS_ARCHIVER_STYLE")
        .expect("actual target archiver command-line style required");
    let archive = output.join(if style == "msvc" {
        "skia_native_alias.lib"
    } else {
        "libskia_native_alias.a"
    });
    let mut command = Command::new(archiver);
    match style.as_str() {
        "gnu" => {
            command.arg("crs").arg(&archive);
        }
        "msvc" => {
            command
                .arg("/NOLOGO")
                .arg(format!("/OUT:{}", archive.display()));
        }
        _ => panic!("actual archiver style must be gnu or msvc"),
    }
    let status = command
        .args(&objects)
        .status()
        .expect("archive native Alias objects");
    assert!(
        status.success(),
        "Alias native object archive was not produced"
    );
    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=static=skia_native_alias");
}

// These are resolved native-TU preprocessor inputs. Presence, including defined
// zero, is distinct from absence. Never use Rust's host/profile as a substitute.
fn bind_native_numeric_inputs() {
    println!("cargo:rustc-check-cfg=cfg(base_numeric_native_inputs_configured)");
    let pairs = [
        ("__asmjs__", "base_numeric_asmjs"),
        ("__wasm__", "base_numeric_wasm"),
        ("__ARMEL__", "base_numeric_armel"),
        ("__clang__", "base_numeric_compiler_clang"),
        ("__GNUC__", "base_numeric_compiler_gcc"),
        ("_MSC_VER", "base_numeric_compiler_msvc"),
        ("NDEBUG", "base_numeric_ndebug"),
    ];
    for (_, cfg) in pairs {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }
    println!("cargo:rerun-if-env-changed=BASE_NUMERIC_COMPLETE_NATIVE_INPUTS");
    match std::env::var("BASE_NUMERIC_COMPLETE_NATIVE_INPUTS")
        .ok()
        .as_deref()
    {
        None | Some("0") => return,
        Some("1") => {}
        _ => panic!("BASE_NUMERIC_COMPLETE_NATIVE_INPUTS must be explicit 0 or 1"),
    }
    let mut present = std::collections::BTreeMap::new();
    for (native, cfg) in pairs {
        let key = format!("BASE_NUMERIC_INPUT_{native}");
        println!("cargo:rerun-if-env-changed={key}");
        let value = std::env::var(&key).unwrap_or_else(|_| {
            panic!("missing actual numeric native input {native}; use undefined for absence")
        });
        let defined = value != "undefined";
        if defined {
            println!("cargo:rustc-cfg={cfg}");
        }
        println!("cargo:rustc-env={key}={value}");
        present.insert(native, defined);
    }
    if present["__ARMEL__"] {
        assert!(
            std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("arm"),
            "actual __ARMEL__ inline assembly requires the Rust ARM32 target"
        );
    }
    if present["_MSC_VER"] {
        assert!(bind_native_primitive_archive("DEBUGBREAK", "base_numeric_native_debugbreak_linked", "skia_native_debugbreak"),
            "the actual MSVC-first numeric failure branch requires its true __debugbreak bridge archive");
    }
    // Keep the separate compiler-presence inputs: each native header owns its
    // own precedence (notably _MSC_VER before __clang__ in HandleFailure).
    let optimized = !present["__asmjs__"] && !present["__wasm__"];
    if optimized && (present["__clang__"] || present["__GNUC__"]) {
        assert!(
            bind_native_primitive_archive(
                "OVERFLOW",
                "base_numeric_native_overflow_linked",
                "skia_native_overflow"
            ),
            "the actual numeric builtin branch requires its real target-native bridge archive"
        );
    }
    println!("cargo:rustc-cfg=base_numeric_native_inputs_configured");
}

// Link an artifact built from the supplied genuine compiler primitive TU.
// This does not compile/probe a host toolchain or infer a native ABI. The
// embedding supplies an archive made with its actual target and compiler flags.
// Runtime library specifications are supplied one cargo link-lib value per line.
fn bind_native_primitive_archive(kind: &str, cfg: &str, library: &str) -> bool {
    use std::path::PathBuf;
    println!("cargo:rustc-check-cfg=cfg({cfg})");
    let key = |suffix: &str| format!("SKIA_NATIVE_{kind}_{suffix}");
    for suffix in ["ARCHIVE", "ABI_TARGET", "LIBRARIES"] {
        println!("cargo:rerun-if-env-changed={}", key(suffix));
    }
    let Some(archive) = std::env::var_os(key("ARCHIVE")) else {
        return false;
    };
    let selected_target = std::env::var(key("ABI_TARGET"))
        .expect("actual native bridge Cargo target correspondence is required");
    assert_eq!(
        selected_target,
        std::env::var("TARGET").expect("Cargo selected target"),
        "native bridge must be built for the selected target, never the build host"
    );
    let archive = PathBuf::from(archive);
    assert!(
        archive.is_file(),
        "selected native primitive archive is missing"
    );
    let libraries =
        PathBuf::from(std::env::var_os(key("LIBRARIES")).expect(
            "actual target runtime-library list required; explicitly empty file is permitted",
        ));
    println!("cargo:rerun-if-changed={}", archive.display());
    println!("cargo:rerun-if-changed={}", libraries.display());
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    let destination = output.join(if msvc {
        format!("{library}.lib")
    } else {
        format!("lib{library}.a")
    });
    std::fs::copy(&archive, &destination)
        .expect("retain genuine native primitive archive for static linking");
    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=static={library}");
    for dependency in std::fs::read_to_string(libraries)
        .expect("read actual target runtime libraries")
        .lines()
    {
        if !dependency.is_empty() {
            println!("cargo:rustc-link-lib={dependency}");
        }
    }
    println!("cargo:rustc-cfg={cfg}");
    true
}

// Build the actual SDK TU only when the embedding supplies genuine metadata,
// target compiler and its original argv. No measurements or host defaults.
fn bind_native_ostream_storage() {
    use std::{path::PathBuf, process::Command};
    let read = |suffix: &str| {
        let key = format!("SKIA_NATIVE_OSTREAM_{suffix}");
        println!("cargo:rerun-if-env-changed={key}");
        std::env::var_os(key)
    };
    let metadata = read("ABI_METADATA_PATH").map(PathBuf::from);
    println!("cargo:rustc-check-cfg=cfg(sk_native_ostream_abi_configured)");
    let Some(metadata) = metadata else {
        return;
    };
    let target = std::env::var("TARGET").expect("Cargo selected target");
    let domain = read("ABI_DOMAIN")
        .expect("actual native C++ SDK/ABI identity required")
        .into_string()
        .expect("UTF-8 native ABI identity");
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    let prepared =
        native_ostream_abi::prepare_native_ostream_abi(Some(&metadata), &output, &target, &domain)
            .expect("prepare genuine supplied native stream ABI")
            .expect("explicit metadata must prepare an actual ABI instance");
    let compiler = read("CXX").expect("actual selected target C++ compiler required");
    let style =
        read("COMPILER_STYLE").expect("explicit actual native compiler argv style required");
    let arguments =
        PathBuf::from(read("CXX_ARGS").expect("actual native compiler argv file required"));
    println!("cargo:rerun-if-changed={}", arguments.display());
    println!("cargo:rerun-if-changed=interop/native_ostream.cpp");
    let mut command = Command::new(compiler);
    // One complete argv value per line: spaces are preserved, no shell splitting.
    for argument in std::fs::read_to_string(arguments)
        .expect("read actual native compiler argv")
        .lines()
    {
        command.arg(argument);
    }
    let msvc = style == "msvc";
    let object = output.join(if msvc {
        "skia_native_ostream.obj"
    } else {
        "skia_native_ostream.o"
    });
    if style == "gnu" {
        command
            .arg("-I")
            .arg(prepared.include_dir())
            .arg("-c")
            .arg("interop/native_ostream.cpp")
            .arg("-o")
            .arg(&object);
    } else if msvc {
        command
            .arg(format!("/I{}", prepared.include_dir().display()))
            .arg("/c")
            .arg("interop/native_ostream.cpp")
            .arg(format!("/Fo{}", object.display()));
    } else {
        panic!("actual native compiler style must be gnu or msvc");
    }
    let status = command
        .status()
        .expect("compile actual target native stream TU");
    assert!(
        status.success(),
        "native stream operations and exact SDK static_asserts did not compile"
    );
    let archiver = read("ARCHIVER").expect("actual target native archiver required");
    let archive_style = read("ARCHIVER_STYLE").expect("actual native archiver argv style required");
    let archive = output.join(if archive_style == "msvc" {
        "skia_native_ostream.lib"
    } else {
        "libskia_native_ostream.a"
    });
    let mut command = Command::new(archiver);
    if archive_style == "gnu" {
        command.arg("crs").arg(&archive);
    } else if archive_style == "msvc" {
        command
            .arg("/NOLOGO")
            .arg(format!("/OUT:{}", archive.display()));
    } else {
        panic!("actual archiver style must be gnu or msvc");
    }
    assert!(
        command
            .arg(&object)
            .status()
            .expect("archive actual native stream object")
            .success(),
        "native stream archive creation failed"
    );
    let libraries = PathBuf::from(
        read("LIBRARIES").expect("explicit actual C++ runtime library argv file required"),
    );
    println!("cargo:rerun-if-changed={}", libraries.display());
    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=static=skia_native_ostream");
    for library in std::fs::read_to_string(libraries)
        .expect("read actual C++ runtime libraries")
        .lines()
    {
        if !library.is_empty() {
            println!("cargo:rustc-link-lib={library}");
        }
    }
    // SAFETY: the exact canonical TU was compiled above with this very prepared
    // include directory. A failed native compile or archive aborts before here.
    // Its actual generated static_asserts guard both SDK object dimensions and
    // that same successfully compiled object is retained for static linkage.
    unsafe {
        prepared.publish_after_successful_native_compile();
    }
}

// Exact resolved Chromium BUILDFLAG values, separate from Rust target/host or
// Skia's own preprocessor input stream. These are embedding inputs, no algorithms.
fn bind_chromium_build_inputs() {
    println!("cargo:rustc-check-cfg=cfg(base_native_build_inputs_configured)");
    let inputs = [
        ("IS_ANDROID", "base_is_android"),
        ("IS_APPLE", "base_is_apple"),
        ("IS_BSD", "base_is_bsd"),
        ("IS_CASTOS", "base_is_castos"),
        ("IS_CAST_ANDROID", "base_is_cast_android"),
        ("IS_CHROMEOS", "base_is_chromeos"),
        ("IS_FUCHSIA", "base_is_fuchsia"),
        ("IS_IOS", "base_is_ios"),
        ("IS_LINUX", "base_is_linux"),
        ("IS_MAC", "base_is_mac"),
        ("IS_POSIX", "base_is_posix"),
        ("IS_WIN", "base_is_win"),
    ];
    for (_, cfg) in inputs {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }
    println!("cargo:rustc-check-cfg=cfg(chromium_is_chromeos)");
    println!("cargo:rerun-if-env-changed=BASE_COMPLETE_NATIVE_BUILD_INPUTS");
    match std::env::var("BASE_COMPLETE_NATIVE_BUILD_INPUTS")
        .ok()
        .as_deref()
    {
        None | Some("0") => return,
        Some("1") => {}
        _ => panic!("BASE_COMPLETE_NATIVE_BUILD_INPUTS is explicit 0/1"),
    }
    for (native, cfg) in inputs {
        let key = format!("BASE_BUILDFLAG_{native}");
        println!("cargo:rerun-if-env-changed={key}");
        let value = std::env::var(&key)
            .unwrap_or_else(|_| panic!("actual native BUILDFLAG {native} required"));
        match value.as_str() {
            "0" => {}
            "1" => {
                println!("cargo:rustc-cfg={cfg}");
                if native == "IS_CHROMEOS" {
                    println!("cargo:rustc-cfg=chromium_is_chromeos");
                }
            }
            _ => panic!("BUILDFLAG inputs must be resolved 0/1 expressions, not guessed presence"),
        }
        println!("cargo:rustc-env={key}={value}");
    }
    let mut diagnostics = std::collections::BTreeMap::new();
    for native in [
        "OFFICIAL_BUILD",
        "NDEBUG",
        "DCHECK_ALWAYS_ON",
        "FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION",
    ] {
        let key = format!("BASE_NATIVE_PREDEF_{native}");
        println!("cargo:rerun-if-env-changed={key}");
        let value = std::env::var(&key).expect(
            "actual diagnostic native preprocessor presence required; undefined means absent",
        );
        let defined = value != "undefined";
        diagnostics.insert(native, defined);
        if native != "NDEBUG" {
            println!("cargo:rustc-check-cfg=cfg({native})");
            if defined {
                println!("cargo:rustc-cfg={native}");
            }
        }
    }
    println!("cargo:rustc-check-cfg=cfg(base_dcheck_is_on)");
    if !diagnostics["NDEBUG"] || diagnostics["DCHECK_ALWAYS_ON"] {
        println!("cargo:rustc-cfg=base_dcheck_is_on");
    }
    for native in ["EXPENSIVE_DCHECKS_ARE_ON", "ENABLE_CHECK_ELISION_WARNING"] {
        let key = format!("BASE_NATIVE_RESOLVED_{native}");
        println!("cargo:rerun-if-env-changed={key}");
        println!("cargo:rustc-check-cfg=cfg({native})");
        match std::env::var(&key)
            .expect("actual generated native diagnostic macro 0/1 expansion required")
            .as_str()
        {
            "0" => {}
            "1" => println!("cargo:rustc-cfg={native}"),
            _ => panic!("diagnostic function-macro expansion must be resolved 0/1"),
        }
    }
    // Both translated source groups currently share this crate cfg. A native
    // build with different NDEBUG per TU requires an explicit per-TU mapping,
    // not silently combining the two incompatible inputs.
    let skia_complete = std::env::var("SK_LOAD_USER_CONFIG_COMPLETE_INPUTS").is_ok();
    let skia_ndebug = std::env::var("SK_LOAD_USER_CONFIG_AFTER_INCLUDE_NDEBUG")
        .or_else(|e| {
            if skia_complete {
                Err(e)
            } else {
                std::env::var("NDEBUG")
            }
        })
        .map_or(false, |v| v != "undefined");
    assert_eq!(diagnostics["NDEBUG"],skia_ndebug,
        "supply consistent actual native NDEBUG inputs for this shared Rust cfg; no host/profile fallback");
    println!("cargo:rustc-cfg=base_native_build_inputs_configured");
}

fn bind_native_location_inputs() {
    for cfg in [
        "native_compiler_gcc",
        "native_compiler_msvc",
        "native_compiler_other",
        "native_location_raw_ptr_noop",
    ] {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }
    for key in [
        "BASE_LOCATION_COMPLETE_NATIVE_INPUTS",
        "BASE_LOCATION_COMPILER_DOMAIN",
        "BASE_LOCATION_RAW_PTR_DOMAIN",
        "BASE_LOCATION_NATIVE_FILE",
        "BASE_LOCATION_NATIVE_THIS_FILE_PATH",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    match std::env::var("BASE_LOCATION_COMPLETE_NATIVE_INPUTS")
        .ok()
        .as_deref()
    {
        None | Some("0") => return,
        Some("1") => {}
        _ => panic!("BASE_LOCATION_COMPLETE_NATIVE_INPUTS is explicit 0/1"),
    }
    let compiler = std::env::var("BASE_LOCATION_COMPILER_DOMAIN")
        .expect("actual COMPILER_GCC/MSVC/other branch from native build_config.h required");
    match compiler.as_str() {
        "gcc" => {
            assert!(bind_native_primitive_archive("LOCATION", "native_location_extract_linked", "skia_native_location"),
                "actual GCC-family Location requires its true extract-return-address helper archive");
            println!("cargo:rustc-cfg=native_compiler_gcc");
        }
        "msvc" => println!("cargo:rustc-cfg=native_compiler_msvc"),
        "other" => println!("cargo:rustc-cfg=native_compiler_other"),
        _ => panic!("actual native Location compiler domain must be gcc, msvc or other"),
    }
    // This selection is an explicit native type/config assertion by the embedder.
    // Instrumented/zero-on-move domains remain unsupported until real producers
    // are connected. They cannot silently select plain pointer semantics.
    let pointer = std::env::var("BASE_LOCATION_RAW_PTR_DOMAIN")
        .expect("actual raw_ptr backend/type domain required");
    match pointer.as_str() {
        "noop" => println!("cargo:rustc-cfg=native_location_raw_ptr_noop"),
        "instrumented" | "zero_on_move" => {}
        _ => panic!("actual native raw_ptr domain must be explicit, never inferred from host"),
    }
    for key in [
        "BASE_LOCATION_NATIVE_FILE",
        "BASE_LOCATION_NATIVE_THIS_FILE_PATH",
    ] {
        let value =
            std::env::var(key).expect("original Location native __FILE__/suffix input required");
        assert!(
            !value.is_empty()
                && !value.contains('\0')
                && !value.contains('\n')
                && !value.contains('\r'),
            "native Location source metadata must be one real nonempty source string"
        );
        println!("cargo:rustc-env={key}={value}");
    }
}

fn bind_native_time_inputs() {
    for cfg in [
        "base_native_time_inputs_configured",
        "base_posix_monotonic_clock",
        "base_posix_thread_cputime",
    ] {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }
    println!("cargo:rerun-if-env-changed=BASE_TIME_COMPLETE_NATIVE_INPUTS");
    match std::env::var("BASE_TIME_COMPLETE_NATIVE_INPUTS")
        .ok()
        .as_deref()
    {
        None | Some("0") => return,
        Some("1") => {}
        _ => panic!("BASE_TIME_COMPLETE_NATIVE_INPUTS is explicit 0/1"),
    }
    // Source: base/time/time_now_posix.cc:50-52,136-137. Android/BSD
    // alternatives remain in the translated source's original cfg branches.
    for (native, cfg) in [
        ("_POSIX_MONOTONIC_CLOCK", "base_posix_monotonic_clock"),
        ("_POSIX_THREAD_CPUTIME", "base_posix_thread_cputime"),
    ] {
        let key = format!("BASE_TIME_INPUT_{native}");
        println!("cargo:rerun-if-env-changed={key}");
        let value = std::env::var(&key).expect(
            "actual resolved unistd native feature input required; use undefined for absence",
        );
        if value != "undefined" {
            let value = value
                .parse::<i64>()
                .expect("supply resolved signed integer native macro value");
            if value >= 0 {
                println!("cargo:rustc-cfg={cfg}");
            }
        }
    }
    println!("cargo:rustc-cfg=base_native_time_inputs_configured");
}

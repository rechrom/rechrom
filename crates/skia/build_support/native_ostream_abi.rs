//! Root-owned build integration fragment. This module never compiles/probes a
//! target, infers a host layout, or enables a stream configuration by itself.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub struct PreparedNativeOStreamAbi {
    include_dir: PathBuf,
    metadata_path: PathBuf,
    target: String,
    native_domain: String,
}
impl PreparedNativeOStreamAbi {
    /// Add to the actual native_ostream.cpp compiler's include search path.
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn native_domain(&self) -> &str {
        &self.native_domain
    }
    /// # Safety
    /// The root build has just successfully compiled the exact canonical
    /// interop/native_ostream.cpp TU with this include_dir's generated header,
    /// metadata target/domain, original base C++ runtime/SDK/ABI flags and no
    /// exceptions. Its real sizeof/align static_asserts passed in that native
    /// compilation, and that same object will be linked into this Rust target.
    /// Merely generating files, parsing metadata or compiling another TU does
    /// not meet this contract. There is no successful verification token here.
    pub unsafe fn publish_after_successful_native_compile(&self) {
        println!("cargo:rustc-cfg=sk_native_ostream_abi_configured");
        println!("cargo:rerun-if-changed={}", self.metadata_path.display());
    }
}

/// Prepare literal Rust storage and C++ static-assert input from an explicitly
/// supplied, already genuine native ABI record. `target` is root's actual TARGET,
/// never HOST. `native_domain` is the actual native build identity supplied by
/// root, never a compiler/stdlib guess made by this helper. None means unsupported
/// and writes no files/publishes no cfg, even if stale output files exist.
pub fn prepare_native_ostream_abi(
    metadata_path: Option<&Path>,
    out_dir: &Path,
    target: &str,
    native_domain: &str,
) -> Result<Option<PreparedNativeOStreamAbi>, String> {
    println!("cargo:rustc-check-cfg=cfg(sk_native_ostream_abi_configured)");
    let Some(metadata_path) = metadata_path else {
        return Ok(None);
    };
    if target.is_empty() || native_domain.is_empty() {
        return Err("actual TARGET and native ABI domain must both be supplied".into());
    }
    let contents = fs::read_to_string(metadata_path)
        .map_err(|e| format!("read supplied native ABI metadata: {e}"))?;
    let fields = parse_record(&contents)?;
    if fields["format"] != "rechrom.native_ostream_abi.v1" {
        return Err("unsupported supplied metadata format".into());
    }
    if fields["target_triple"] != target {
        return Err("supplied metadata belongs to another target; no host fallback".into());
    }
    if fields["native_abi_domain"] != native_domain {
        return Err("supplied metadata belongs to another C++ SDK/ABI domain".into());
    }
    let stream_size = decimal(&fields, "ostringstream_size")?;
    let stream_align = decimal(&fields, "ostringstream_align")?;
    let string_size = decimal(&fields, "std_string_size")?;
    let string_align = decimal(&fields, "std_string_align")?;
    let bidirectional_size = decimal(&fields, "stringstream_size")?;
    let bidirectional_align = decimal(&fields, "stringstream_align")?;
    storage_shape(stream_size, stream_align)?;
    storage_shape(string_size, string_align)?;
    storage_shape(bidirectional_size, bidirectional_align)?;
    let origin = &fields["metadata_origin"];
    let sdk = &fields["sdk_definition_identity"];
    let flags = &fields["native_cxx_flags_identity"];
    let rust = format!(
        r#"// Generated from supplied genuine native ABI metadata, not a probe/default.
pub const TARGET_TRIPLE:&str={target:?};
pub const NATIVE_ABI_DOMAIN:&str={native_domain:?};
pub const METADATA_ORIGIN:&str={origin:?};
pub const SDK_DEFINITION_IDENTITY:&str={sdk:?};
pub const NATIVE_CXX_FLAGS_IDENTITY:&str={flags:?};
#[repr(C,align({stream_align}))]
pub struct SuppliedOStringStreamStorage {{bytes:[core::mem::MaybeUninit<u8>;{stream_size}]}}
unsafe impl super::OStringStreamStorage for SuppliedOStringStreamStorage {{
    const NATIVE_SIZE:usize={stream_size};const NATIVE_ALIGN:usize={stream_align};
}}
const _:[();{stream_size}]=[();core::mem::size_of::<SuppliedOStringStreamStorage>()];
const _:[();{stream_align}]=[();core::mem::align_of::<SuppliedOStringStreamStorage>()];
#[repr(C,align({string_align}))]
pub struct SuppliedStdStringStorage {{bytes:[core::mem::MaybeUninit<u8>;{string_size}]}}
unsafe impl super::StdStringStorage for SuppliedStdStringStorage {{
    const NATIVE_SIZE:usize={string_size};const NATIVE_ALIGN:usize={string_align};
}}
const _:[();{string_size}]=[();core::mem::size_of::<SuppliedStdStringStorage>()];
const _:[();{string_align}]=[();core::mem::align_of::<SuppliedStdStringStorage>()];
#[repr(C,align({bidirectional_align}))]
pub struct SuppliedStringStreamStorage {{bytes:[core::mem::MaybeUninit<u8>;{bidirectional_size}]}}
unsafe impl super::StringStreamStorage for SuppliedStringStreamStorage {{
    const NATIVE_SIZE:usize={bidirectional_size};const NATIVE_ALIGN:usize={bidirectional_align};
}}
const _:[();{bidirectional_size}]=[();core::mem::size_of::<SuppliedStringStreamStorage>()];
const _:[();{bidirectional_align}]=[();core::mem::align_of::<SuppliedStringStreamStorage>()];
"#
    );
    let header = format!(
        r#"// Generated from the same explicit SDK ABI record as the Rust storage.
#ifndef RECHROM_NATIVE_OSTREAM_ABI_H
#define RECHROM_NATIVE_OSTREAM_ABI_H
#define RECHROM_NATIVE_OSTREAM_ABI_METADATA_AVAILABLE 1
#define RECHROM_NATIVE_OSTRINGSTREAM_SIZE {stream_size}
#define RECHROM_NATIVE_OSTRINGSTREAM_ALIGN {stream_align}
#define RECHROM_NATIVE_STD_STRING_SIZE {string_size}
#define RECHROM_NATIVE_STD_STRING_ALIGN {string_align}
#define RECHROM_NATIVE_STRINGSTREAM_SIZE {bidirectional_size}
#define RECHROM_NATIVE_STRINGSTREAM_ALIGN {bidirectional_align}
#define RECHROM_NATIVE_OSTREAM_TARGET {target}
#define RECHROM_NATIVE_OSTREAM_ABI_DOMAIN {domain}
#define RECHROM_NATIVE_OSTREAM_METADATA_ORIGIN {origin}
#define RECHROM_NATIVE_OSTREAM_SDK_IDENTITY {sdk}
#define RECHROM_NATIVE_OSTREAM_CXX_FLAGS_IDENTITY {flags}
#endif
"#,
        target = c_string(target),
        domain = c_string(native_domain),
        origin = c_string(origin),
        sdk = c_string(sdk),
        flags = c_string(flags)
    );
    fs::create_dir_all(out_dir).map_err(|e| format!("create output directory: {e}"))?;
    fs::write(out_dir.join("rechrom_native_ostream_abi_storage.rs"), rust)
        .map_err(|e| format!("write Rust ABI storage: {e}"))?;
    fs::write(out_dir.join("rechrom_native_ostream_abi.h"), header)
        .map_err(|e| format!("write C++ ABI input: {e}"))?;
    Ok(Some(PreparedNativeOStreamAbi {
        include_dir: out_dir.into(),
        metadata_path: metadata_path.into(),
        target: target.into(),
        native_domain: native_domain.into(),
    }))
}

fn parse_record(contents: &str) -> Result<BTreeMap<String, String>, String> {
    const KEYS: [&str; 12] = [
        "format",
        "target_triple",
        "native_abi_domain",
        "metadata_origin",
        "sdk_definition_identity",
        "native_cxx_flags_identity",
        "ostringstream_size",
        "ostringstream_align",
        "std_string_size",
        "std_string_align",
        "stringstream_size",
        "stringstream_align",
    ];
    let mut fields = BTreeMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err("metadata entry must be key=value".into());
        };
        let key = key.trim();
        let value = value.trim();
        if !KEYS.contains(&key) {
            return Err(format!("unknown native ABI metadata key {key}"));
        }
        if value.is_empty() || value.chars().any(|c| c.is_control()) {
            return Err(format!("empty/control-bearing native ABI value {key}"));
        }
        if fields.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("duplicate native ABI key {key}"));
        }
    }
    for key in KEYS {
        if !fields.contains_key(key) {
            return Err(format!("missing supplied native ABI field {key}"));
        }
    }
    Ok(fields)
}
fn decimal(fields: &BTreeMap<String, String>, key: &str) -> Result<u64, String> {
    let value = &fields[key];
    if !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{key} must be supplied decimal integer metadata"));
    }
    let n = value
        .parse::<u64>()
        .map_err(|_| format!("{key} is outside the literal emitter domain"))?;
    if n == 0 {
        return Err(format!("{key} must be nonzero"));
    }
    Ok(n)
}
fn storage_shape(size: u64, align: u64) -> Result<(), String> {
    // Rust's repr(align) accepts power-of-two integer literals up to 2^29. This
    // checks literal representability; it does not infer any target type layout.
    if !align.is_power_of_two() || align > (1u64 << 29) || size % align != 0 {
        return Err(
            "supplied native layout is not representable by this exact literal Rust storage".into(),
        );
    }
    Ok(())
}
fn c_string(value: &str) -> String {
    let mut out = String::from("\"");
    for byte in value.bytes() {
        match byte {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            0x20..=0x7e => out.push(byte as char),
            // Fixed-width octal avoids consuming a following hex digit.
            _ => out.push_str(&format!("\\{:03o}", byte)),
        }
    }
    out.push('"');
    out
}

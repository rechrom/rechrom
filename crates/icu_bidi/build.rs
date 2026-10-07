use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};

fn pkg_config(args: &[&str], extra_path: Option<&PathBuf>) -> Option<Output> {
    let mut command = Command::new("pkg-config");
    command.args(args);
    if let Some(extra_path) = extra_path {
        let mut paths = vec![extra_path.clone()];
        if let Some(existing) = env::var_os("PKG_CONFIG_PATH") {
            paths.extend(env::split_paths(&existing));
        }
        command.env("PKG_CONFIG_PATH", env::join_paths(paths).ok()?);
    }
    command
        .output()
        .ok()
        .filter(|output| output.status.success())
}

fn homebrew_icu_pkgconfig() -> Option<PathBuf> {
    for root in ["/opt/homebrew/opt", "/usr/local/opt"] {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("icu4c"))
            .map(|entry| entry.path().join("lib/pkgconfig"))
            .filter(|path| path.join("icu-uc.pc").exists())
            .collect();
        paths.sort();
        if let Some(path) = paths.pop() {
            return Some(path);
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=ICU_PKG_CONFIG_PATH");
    let explicit_path = env::var_os("ICU_PKG_CONFIG_PATH").map(PathBuf::from);
    let extra_path = if explicit_path.is_some() {
        explicit_path
    } else if pkg_config(&["--modversion", "icu-uc"], None).is_some() {
        None
    } else {
        homebrew_icu_pkgconfig()
    };
    let version = pkg_config(&["--modversion", "icu-uc"], extra_path.as_ref())
        .expect("ICU uc development package is required by icu_bidi");
    let version = String::from_utf8(version.stdout).expect("ICU version is not UTF-8");
    let major = version.trim().split('.').next().unwrap_or_default();
    assert!(major.chars().all(|ch| ch.is_ascii_digit()) && !major.is_empty());
    println!("cargo:rustc-env=ICU_MAJOR={major}");
    let flags = pkg_config(&["--libs", "icu-uc"], extra_path.as_ref())
        .expect("ICU uc linker flags are required");
    let flags = String::from_utf8(flags.stdout).expect("ICU flags are not UTF-8");
    for flag in flags.split_whitespace() {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(name) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={name}");
        }
    }
}

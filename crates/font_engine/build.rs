use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // HarfBuzz is still a native dependency. Resolve the installed development
    // package explicitly, never an unrelated sibling checkout's static archive.
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=HARFBUZZ_PKG_CONFIG_PATH");
    let mut command = Command::new("pkg-config");
    command.args(["--libs", "harfbuzz"]);
    let extra = env::var_os("HARFBUZZ_PKG_CONFIG_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            (env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos"))
                .then(|| {
                    [
                        "/opt/homebrew/opt/harfbuzz/lib/pkgconfig",
                        "/usr/local/opt/harfbuzz/lib/pkgconfig",
                    ]
                    .into_iter()
                    .map(PathBuf::from)
                    .find(|path| path.join("harfbuzz.pc").exists())
                })
                .flatten()
        });
    if let Some(extra) = extra {
        let mut paths = vec![extra];
        if let Some(existing) = env::var_os("PKG_CONFIG_PATH") {
            paths.extend(env::split_paths(&existing));
        }
        command.env(
            "PKG_CONFIG_PATH",
            env::join_paths(paths).expect("invalid HarfBuzz package path"),
        );
    }
    let output = command
        .output()
        .expect("pkg-config is required to locate HarfBuzz");
    assert!(
        output.status.success(),
        "HarfBuzz development package is required: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let flags = String::from_utf8(output.stdout).expect("HarfBuzz linker flags are not UTF-8");
    for flag in flags.split_whitespace() {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(name) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={name}");
        }
    }
}

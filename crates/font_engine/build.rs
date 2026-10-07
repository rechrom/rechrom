use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let archive = manifest.join("../../../layoutng/build/libharfbuzz.a");
    if archive.exists() {
        println!(
            "cargo:rustc-link-search=native={}",
            archive.parent().unwrap().display()
        );
        println!("cargo:rustc-link-lib=static=harfbuzz");
    } else {
        println!("cargo:rustc-link-lib=dylib=harfbuzz");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        // The current archive contains C++ objects; test binaries and
        // standalone examples need its standard-library symbols too.
        println!("cargo:rustc-link-lib=c++");
    }
}

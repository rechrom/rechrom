use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let text = env::var_os("CARGO_FEATURE_TEXT").is_some();
    let source_replay = env::var_os("CARGO_FEATURE_SOURCE_REPLAY").is_some();
    if !text && !source_replay {
        return;
    }
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let skia = manifest.join("../../../layoutng/deps/skia");
    let archive = skia.join("lib/libskia.a");
    assert!(archive.exists(), "Skia archive is required for text replay");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut objects = Vec::new();
    let static_lib = output.join("liblayoutng_text_replay.a");
    for (enabled, source, object_name) in [
        (text, "src/text_replay.cc", "text_replay.o"),
        (source_replay, "src/native_canvas.cc", "native_canvas.o"),
    ] {
        if !enabled {
            continue;
        }
        let object = output.join(object_name);
        let status = Command::new("clang++")
            .arg("-std=c++20")
            .arg("-fexceptions")
            .arg("-DSK_A32_SHIFT=24")
            .arg("-DSK_R32_SHIFT=16")
            .arg("-DSK_G32_SHIFT=8")
            .arg("-DSK_B32_SHIFT=0")
            .arg(format!("-I{}", skia.display()))
            .args(["-c", source, "-o"])
            .arg(&object)
            .status()
            .expect("clang++ is required for Skia replay");
        assert!(
            status.success(),
            "Skia C++ bridge failed to compile: {source}"
        );
        println!("cargo:rerun-if-changed={source}");
        objects.push(object);
    }
    let status = Command::new("ar")
        .arg("rcs")
        .arg(&static_lib)
        .args(&objects)
        .status()
        .expect("ar is required for Skia text replay");
    assert!(status.success(), "Skia text replay archive failed");
    println!("cargo:rustc-link-search=native={}", output.display());
    println!(
        "cargo:rustc-link-search=native={}",
        skia.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=layoutng_text_replay");
    println!("cargo:rustc-link-lib=static=skia");
    println!("cargo:rustc-link-lib=static=skcms");
    for framework in [
        "ApplicationServices",
        "CoreFoundation",
        "CoreGraphics",
        "CoreText",
        "Foundation",
        "ImageIO",
    ] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
    println!("cargo:rustc-link-lib=c++");
}

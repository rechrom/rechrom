fn main() {
    println!("cargo:rustc-check-cfg=cfg(rechrom_libjpeg)");
    println!("cargo:rerun-if-changed=src/jpeg_scaled_decode.c");
    if let Ok(library) = pkg_config::Config::new().statik(true).probe("libjpeg") {
        let mut build = cc::Build::new();
        build.file("src/jpeg_scaled_decode.c");
        for include in library.include_paths {
            build.include(include);
        }
        if std::env::var("CARGO_CFG_TARGET_ENDIAN").as_deref() == Ok("little")
            && std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("android")
        {
            build.define("RECHROM_N32_BGRA", None);
        }
        build.compile("image_decoder_jpeg_scaled");
        println!("cargo:rustc-cfg=rechrom_libjpeg");
    }
    println!("cargo:rerun-if-changed=src/macos_video_frame_decoder.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/macos_video_frame_decoder.m")
            .flag("-fobjc-arc")
            .flag("-fblocks")
            .compile("image_decoder_macos_video_frame");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}

fn main() {
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

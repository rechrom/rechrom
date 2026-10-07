fn main() {
    println!("cargo:rerun-if-changed=src/devtools_window.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/devtools_window.m")
            .flag("-fobjc-arc")
            .compile("browser_devtools_window");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=WebKit");
    }
}

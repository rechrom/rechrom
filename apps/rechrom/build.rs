fn main() {
    println!("cargo:rerun-if-changed=src/devtools_window.m");
    println!("cargo:rerun-if-changed=src/native_drag.m");
    println!("cargo:rerun-if-changed=src/native_kill_ring.m");
    println!("cargo:rerun-if-changed=src/native_keyboard_modifiers.m");
    println!("cargo:rerun-if-changed=src/native_clipboard.m");
    println!("cargo:rerun-if-changed=src/native_drag_win.cpp");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        cc::Build::new()
            .cpp(true)
            .file("src/native_drag_win.cpp")
            .std("c++17")
            .compile("rechrom_native_drag");
        for library in ["ole32", "uuid", "shell32", "shlwapi", "gdi32", "user32"] {
            println!("cargo:rustc-link-lib={library}");
        }
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/devtools_window.m")
            .file("src/native_drag.m")
            .file("src/native_kill_ring.m")
            .file("src/native_keyboard_modifiers.m")
            .file("src/native_clipboard.m")
            .flag("-fobjc-arc")
            .compile("browser_devtools_window");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=WebKit");
        println!("cargo:rustc-link-lib=framework=UniformTypeIdentifiers");
    }
}

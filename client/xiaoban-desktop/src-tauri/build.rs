fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rerun-if-changed=macos/computer-use-indicator.m");
        cc::Build::new()
            .file("macos/computer-use-indicator.m")
            .flag("-fobjc-arc")
            .compile("xiaoban_computer_use_indicator");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
    }
    tauri_build::build()
}

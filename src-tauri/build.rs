fn main() {
    #[cfg(target_os = "macos")]
    configure_macos_swift_runtime_rpath();

    tauri_build::build()
}

#[cfg(target_os = "macos")]
fn configure_macos_swift_runtime_rpath() {
    // ScreenCaptureKit bindings link against Swift runtime symbols that are provided
    // by the system Swift runtime on modern macOS releases. Point @rpath there instead
    // of the Xcode toolchain to avoid loading duplicate Swift standard libraries.
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
}

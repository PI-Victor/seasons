fn main() {
    #[cfg(target_os = "macos")]
    configure_macos_swift_runtime_rpath();

    tauri_build::build()
}

#[cfg(target_os = "macos")]
fn configure_macos_swift_runtime_rpath() {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");

    let mut candidates = Vec::<PathBuf>::new();

    if let Some(developer_dir) = std::env::var_os("DEVELOPER_DIR").filter(|value| !value.is_empty())
    {
        candidates.extend(swift_runtime_candidates(Path::new(&developer_dir)));
    }

    if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    candidates.extend(swift_runtime_candidates(Path::new(trimmed)));
                }
            }
        }
    }

    candidates.sort();
    candidates.dedup();

    for candidate in candidates {
        if candidate.join("libswift_Concurrency.dylib").exists() {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", candidate.display());
            return;
        }
    }

    println!(
        "cargo:warning=Could not locate the macOS Swift runtime. ScreenCaptureKit may fail at launch."
    );
}

#[cfg(target_os = "macos")]
fn swift_runtime_candidates(developer_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    vec![
        developer_dir.join("Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx"),
        developer_dir.join("Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx"),
        developer_dir.join("usr/lib/swift/macosx"),
        developer_dir.join("usr/lib/swift-5.5/macosx"),
    ]
}

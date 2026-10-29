fn main() {
    println!("cargo:rerun-if-changed=resources/computer-use");
    println!("cargo:rerun-if-changed=../../../libs/agent-tools/computer-use/runtime-lock.json");
    // A direct release cargo build must not silently use a stale/foreign runtime either.
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        let target = std::env::var("TARGET").expect("Cargo TARGET");
        let status = std::process::Command::new("node")
            .args(["../../../libs/agent-tools/computer-use/scripts/prepare-runtime.mjs", "--check", "--target", &target, "--output", "resources/computer-use"])
            .status().expect("Build-time Node.js is required to verify the bundled Computer Use engine");
        assert!(status.success(), "Missing or mismatched Computer Use engine; run npm run computer-use:prepare for the release target first");
    }
    tauri_build::build()
}

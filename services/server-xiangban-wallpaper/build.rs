fn main() {
    // rust-embed requires the folder to exist at compile time, even before the frontend is built.
    let _ = std::fs::create_dir_all("web/dist");
    println!("cargo:rerun-if-changed=web/dist");
}

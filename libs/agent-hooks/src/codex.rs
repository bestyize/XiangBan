//! Codex hook installation is intentionally unsupported until its contract is verified.
use std::io;
/// Returns an explicit unsupported error without reading or modifying configuration.
pub fn plan_install_codex() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Codex hook configuration and trust are not verified",
    ))
}

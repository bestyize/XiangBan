//! Adapters for locally installed coding agents.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod claude;
pub mod codex;
pub mod model;
mod process;
pub use model::*;

#[cfg(test)]
mod test_support {
    use std::path::PathBuf;
    pub fn fake(name: &str) -> PathBuf {
        let exe = std::env::current_exe().unwrap();
        exe.parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(format!("fake-{name}{}", std::env::consts::EXE_SUFFIX))
    }
}

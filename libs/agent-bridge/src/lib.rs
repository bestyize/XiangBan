//! Adapters for locally installed coding agents.
#![warn(missing_docs)]

pub mod claude;
pub mod codex;
pub mod environment;
pub mod injector;
pub mod model;
pub mod observer;
mod process;
pub use environment::*;
pub use injector::*;
pub use model::*;
pub use observer::*;

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

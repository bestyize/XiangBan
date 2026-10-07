//! Startup gate, independent of both Tauri windows and the frontend.
#[cfg(windows)]
pub mod windows;

// Deliberately no persisted "installed" flag: check the real runtime every launch.
pub fn runtime_ready(version: Option<&str>) -> bool {
    version
        .and_then(|value| value.split('.').next()?.parse::<u32>().ok())
        .is_some_and(|major| major >= 109)
}

pub fn prepare(
    mut detect: impl FnMut() -> Option<String>,
    install: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if runtime_ready(detect().as_deref()) {
        return Ok(());
    }
    install()?;
    if runtime_ready(detect().as_deref()) {
        Ok(())
    } else {
        Err(
            "必要组件安装后仍不可用。请检查系统安装权限，或安装微软官方 WebView2 运行时后重试。"
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn installed_runtime_never_runs_installer() {
        prepare(|| Some("140.0.1.2".into()), || panic!("must not download")).unwrap();
    }

    #[test]
    fn install_is_followed_by_real_detection() {
        let installed = Cell::new(false);
        prepare(
            || installed.get().then(|| "140.0.1.2".into()),
            || {
                installed.set(true);
                Ok(())
            },
        )
        .unwrap();
        // A subsequent launch is silent, and removing the runtime is detected again.
        prepare(|| Some("140.0.1.2".into()), || panic!("already installed")).unwrap();
        assert!(prepare(|| None, || Ok(())).is_err());
    }

    #[test]
    fn success_exit_code_alone_does_not_allow_startup() {
        assert!(prepare(|| Some("0.0.0.0".into()), || Ok(())).is_err());
        assert!(prepare(|| Some("108.0.1.2".into()), || Ok(())).is_err());
        assert!(!runtime_ready(Some("not-a-version")));
    }

    #[test]
    fn installer_failure_is_returned_for_retry() {
        assert_eq!(
            prepare(|| None, || Err("offline".into())),
            Err("offline".into())
        );
    }
}

use std::path::Path;
use ::windows::{
    core::{w, HSTRING},
    Storage::StorageFile,
    System::UserProfile::LockScreen,
    Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
};

pub(super) fn apply_picture(path: &Path) -> ::windows::core::Result<()> {
    // This runs on a blocking worker, with initialization scoped to that thread.
    unsafe { RoInitialize(RO_INIT_MULTITHREADED)? };
    let result = (|| {
        let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path.as_os_str()))?.join()?;
        LockScreen::SetImageFileAsync(&file)?.join()
    })();
    unsafe { RoUninitialize() };
    result
}

pub(super) fn open_settings() -> Result<(), String> {
    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            w!("open").as_ptr(),
            w!("ms-settings:lockscreen").as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        )
    };
    if result as isize <= 32 {
        return Err("无法打开系统锁屏设置".into());
    }
    Ok(())
}

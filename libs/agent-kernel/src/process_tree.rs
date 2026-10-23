//! Terminate the group created for one managed command, including silent descendants.
pub fn terminate(pid: u32) {
    #[cfg(unix)]
    unsafe {
        libc::killpg(pid as libc::pid_t, libc::SIGKILL);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::{Command, Stdio};
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .creation_flags(0x08000000)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

use std::process::{Child, Command};
/// Owns termination even if supervision cannot be spawned. Never silently fall back when a
/// Windows job cannot be attached; the child is killed and start returns an error.
pub struct ChildTree {
    pub(crate) child: Child,
    armed: bool,
    #[cfg(windows)]
    job: Job,
}

impl ChildTree {
    pub fn spawn(command: &mut Command) -> Result<Self, String> {
        #[cfg(windows)]
        let job = Job::new()?;
        #[allow(unused_mut)]
        let mut child = command
            .spawn()
            .map_err(|e| format!("cannot start process: {e}"))?;
        #[cfg(windows)]
        if let Err(error) = job.attach(&child) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        Ok(Self {
            child,
            armed: true,
            #[cfg(windows)]
            job,
        })
    }

    pub fn terminate(&mut self) {
        if !self.armed {
            return;
        }
        self.armed = false;
        #[cfg(unix)]
        unsafe {
            libc::killpg(self.child.id() as libc::pid_t, libc::SIGKILL);
        }
        #[cfg(windows)]
        self.job.terminate();
        let _ = self.child.kill();
    }
}

impl Drop for ChildTree {
    fn drop(&mut self) {
        self.terminate();
        let _ = self.child.wait();
    }
}

#[cfg(windows)]
struct Job(usize);

#[cfg(windows)]
impl Job {
    fn new() -> Result<Self, String> {
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() {
                return Err(format!(
                    "cannot create process job: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let job = Self(handle as usize);
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const _,
                std::mem::size_of_val(&limits) as u32,
            ) == 0
            {
                return Err(format!(
                    "cannot configure process job: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(job)
        }
    }
    fn attach(&self, child: &Child) -> Result<(), String> {
        use std::os::windows::io::AsRawHandle;
        unsafe {
            if windows_sys::Win32::System::JobObjects::AssignProcessToJobObject(
                self.0 as _,
                child.as_raw_handle() as _,
            ) == 0
            {
                return Err(format!(
                    "cannot attach process job: {}",
                    std::io::Error::last_os_error()
                ));
            }
        }
        Ok(())
    }
    fn terminate(&self) {
        unsafe {
            windows_sys::Win32::System::JobObjects::TerminateJobObject(self.0 as _, 1);
        }
    }
}
#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0 as _);
        }
    }
}

impl std::ops::Deref for ChildTree {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.child
    }
}
impl std::ops::DerefMut for ChildTree {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}

use std::ptr::null_mut;
use anyhow::Result;
use windows::{Wdk::System::Threading::NtQueryInformationProcess, Win32::{
    Foundation::BOOL,
    System::{Diagnostics::Debug::CheckRemoteDebuggerPresent, Threading::GetCurrentProcess},
}};

pub struct DebugPort {}

impl DebugPort {
    pub fn check_remote_debugger_present() -> Result<bool> {
        let hprocess = unsafe { GetCurrentProcess() };
        let debug_port: *mut BOOL = null_mut();
        unsafe { CheckRemoteDebuggerPresent(hprocess, debug_port) }?;
        Ok(unsafe {*debug_port}.as_bool())
    }
}

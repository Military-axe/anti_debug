use crate::util::BeingDebug;
use anyhow::Result;
use log::{debug, warn};
use std::ffi::c_void;
use windows::{
    Wdk::System::Threading::{NtQueryInformationProcess, ProcessDebugPort},
    Win32::{
        Foundation::{BOOL, HANDLE, NTSTATUS, STATUS_SUCCESS},
        System::{Diagnostics::Debug::CheckRemoteDebuggerPresent, Threading::GetCurrentProcess},
    },
};

pub struct DebugPort {}

impl BeingDebug for DebugPort {
    fn is_being_debug(&self) -> bool {
        Self::nt_query_debug_port()
    }
}

impl DebugPort {
    /// 检查当前进程是否被远程调试
    ///
    /// 通过调用CheckRemoteDebuggerPresentAPI来判断是否有调试端口
    ///
    /// # 返回值
    ///
    /// - `Err`: CheckRemoteDebuggerPresent API报错
    /// - `Ok(true)`: 调试器端口存在
    /// - `Ok(false)`: 调试器端口不存在
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let is_debug = check_remote_debugger_present().unwarp();
    /// match is_debug {
    ///     true => println!("process is being debugged"),
    ///     false => println!("process is no being debugged")
    /// }
    /// ```
    pub fn check_remote_debugger_present() -> Result<bool> {
        let hprocess: HANDLE = unsafe { GetCurrentProcess() };
        let mut debug_port: BOOL = Default::default();
        unsafe { CheckRemoteDebuggerPresent(hprocess, &mut debug_port) }?;
        Ok(debug_port.as_bool())
    }

    /// 检查当前进程是否被调试
    ///
    /// 通过NtQueryInformationProcess API检查ProcessDebugPort类型
    /// 来判断进程是否被开启了调试端口
    ///
    /// # 返回值
    ///
    /// - `true`: 当前进程被开启了调试端口
    /// - `false`: 当前进程未被开启调试端口
    ///
    /// # 示例
    ///
    /// ```ignore
    /// match nt_query_debug_port() {
    ///     true => println!("process is being debugged"),
    ///     false => println!("process is no being debugged")
    /// }
    /// ```
    pub fn nt_query_debug_port() -> bool {
        let hprocess: HANDLE = unsafe { GetCurrentProcess() };
        let mut is_debug_port: u32 = Default::default();
        let debug_port_ref: *mut c_void = &mut is_debug_port as *mut _ as *mut c_void;
        let mut ret_length: u32 = Default::default();
        let status: NTSTATUS = unsafe {
            NtQueryInformationProcess(
                hprocess,
                ProcessDebugPort,
                debug_port_ref,
                4,
                &mut ret_length,
            )
        };

        if status != STATUS_SUCCESS {
            warn!(
                "NtQueryInformationProcess query ProcessDebugPort failed; error code: {:?}",
                ProcessDebugPort
            );
        }

        debug!("debug port: {}", is_debug_port);
        match is_debug_port {
            0 => false,
            _ => true,
        }
    }
}

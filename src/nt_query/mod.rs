use crate::util::BeingDebug;
use anyhow::{Error, Result};
use log::{debug, warn};
use std::{mem::size_of_val, ptr::addr_of_mut};
use windows::{
    Wdk::System::Threading::{
        NtQueryInformationProcess, ProcessDebugObjectHandle, ProcessDebugPort,
    },
    Win32::{
        Foundation::{GetLastError, BOOL, HANDLE, NTSTATUS, STATUS_PORT_NOT_SET, STATUS_SUCCESS},
        System::{Diagnostics::Debug::CheckRemoteDebuggerPresent, Threading::GetCurrentProcess},
    },
};

pub struct DebugPort {}

impl BeingDebug for DebugPort {
    fn is_being_debug(&self) -> bool {
        let status = Self::nt_query_debug_port();
        match status {
            Err(_) => false,
            Ok(x) => x,
        }
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
    /// - `Err`: NtQueryInformationProcess API
    /// - `true`: 当前进程被开启了调试端口
    /// - `false`: 当前进程未被开启调试端口
    ///
    /// # 示例
    ///
    /// ```ignore
    /// match nt_query_debug_port().unwarp() {
    ///     true => println!("process is being debugged"),
    ///     false => println!("process is no being debugged")
    /// }
    /// ```
    pub fn nt_query_debug_port() -> Result<bool> {
        let hprocess: HANDLE = unsafe { GetCurrentProcess() };
        let mut is_debug_port: u64 = Default::default();
        let mut ret_length: u32 = Default::default();
        let status: NTSTATUS = unsafe {
            NtQueryInformationProcess(
                hprocess,
                ProcessDebugPort,
                addr_of_mut!(is_debug_port).cast(),
                u32::try_from(size_of_val(&is_debug_port)).expect("u32::try_from failed!"),
                &mut ret_length,
            )
        };

        if status != STATUS_SUCCESS {
            warn!(
                "NtQueryInformationProcess query ProcessDebugPort failed; error code: {:?}",
                status
            );
            return Err(Error::msg(
                "NtQueryInformationProcess query ProcessDebugPort failed",
            ));
        }

        debug!("debug port: {}", is_debug_port);
        match is_debug_port {
            0 => Ok(false),
            _ => Ok(true),
        }
    }
}

pub struct DebugObject {}

impl BeingDebug for DebugObject {
    fn is_being_debug(&self) -> bool {
        let status = Self::nt_query_debug_object();
        match status {
            Err(_) => false,
            Ok(x) => x,
        }
    }
}

impl DebugObject {
    pub fn nt_query_debug_object() -> Result<bool> {
        let hprocess: HANDLE = unsafe { GetCurrentProcess() };
        let mut debug_objetct_handle: HANDLE = Default::default();
        let mut ret_length: u32 = Default::default();
        let status: NTSTATUS = unsafe {
            NtQueryInformationProcess(
                hprocess,
                ProcessDebugObjectHandle,
                addr_of_mut!(debug_objetct_handle.0).cast(),
                u32::try_from(size_of_val(&debug_objetct_handle)).expect("u32::try_from failed"),
                &mut ret_length,
            )
        };

        // NtQueryInformationProcess的processinformation值是ProcessDebugObjectHandle时
        // 返回值为STATUS_PORT_NOT_SET表示进程未被调试
        if status == STATUS_PORT_NOT_SET {
            debug!("debug port not set");
            return Ok(false)
        }

        if status != STATUS_SUCCESS {
            unsafe {
                warn!(
                    "NtQueryInformationProcess query ProcessDebugObjectHandle failed; error code: {:?}; GetLastError: {:?}",
                    status, GetLastError()
                );
            }

            return Err(Error::msg(
                "NtQueryInformationProcess query ProcessDebugObjectHandle failed",
            ));
        }

        debug!("debug_object_handle ==> {:?}", debug_objetct_handle);

        // match debug_objetct_handle {
        //     0 => Ok(false),
        //     _ => Ok(true),
        // }
        Ok(!debug_objetct_handle.is_invalid())
    }
}

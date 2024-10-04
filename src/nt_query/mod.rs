use crate::util::BeingDebug;
use anyhow::{Error, Result};
use log::{debug, warn};
use std::{mem::size_of_val, ptr::addr_of_mut};
use windows::{
    Wdk::System::Threading::{
        NtQueryInformationProcess, ProcessDebugFlags, ProcessDebugObjectHandle, ProcessDebugPort,
        PROCESSINFOCLASS,
    },
    Win32::{
        Foundation::{BOOL, HANDLE, NTSTATUS, STATUS_PORT_NOT_SET, STATUS_SUCCESS},
        System::{Diagnostics::Debug::CheckRemoteDebuggerPresent, Threading::GetCurrentProcess},
    },
};

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

#[derive(PartialEq, Clone, Debug)]
pub enum QueryType {
    DebugPort = ProcessDebugPort.0 as isize,
    DebugObject = ProcessDebugObjectHandle.0 as isize,
    DebugFlags = ProcessDebugFlags.0 as isize,
}

impl Into<i32> for QueryType {
    fn into(self) -> i32 {
        self as i32
    }
}

pub struct NtQueryDebug {}

impl BeingDebug for NtQueryDebug {
    fn is_being_debug(&self) -> bool {
        let hprocess: HANDLE = unsafe { GetCurrentProcess() };
        Self::check_debug_flags(hprocess)
            && Self::check_debug_object(hprocess)
            && Self::check_debug_port(hprocess)
    }
}

impl NtQueryDebug {
    pub fn nt_query_core(hprocess: HANDLE, query_type: QueryType) -> Result<u64> {
        debug!("process handle ==> {:?}; query type ==> {:?}", hprocess, query_type);
        let mut ret_length: u32 = Default::default();
        let process_information_class = PROCESSINFOCLASS(query_type as i32);
        let mut process_information: u64 = Default::default();
        let status: NTSTATUS = unsafe {
            NtQueryInformationProcess(
                hprocess,
                process_information_class,
                addr_of_mut!(process_information).cast(),
                u32::try_from(size_of_val(&process_information)).expect("u32::try_from failed!"),
                &mut ret_length,
            )
        };

        if status == STATUS_PORT_NOT_SET
            && process_information_class.0 == QueryType::DebugObject.into()
        {
            return Ok(process_information);
        }

        if status != STATUS_SUCCESS {
            warn!("NtQueryInformationProcess failed; error code: {:?}", status);
            return Err(Error::msg("NtQueryInformationProcess failed"));
        }

        Ok(process_information)
    }

    pub fn check_debug_port(hprocess: HANDLE) -> bool {
        match Self::nt_query_core(hprocess, QueryType::DebugPort) {
            Err(_) => false,
            Ok(x) => match x {
                0 => false,
                _ => true,
            },
        }
    }

    pub fn check_debug_object(hprocess: HANDLE) -> bool {
        match Self::nt_query_core(hprocess, QueryType::DebugObject) {
            Err(_) => false,
            Ok(x) => match x {
                0 => false,
                _ => true,
            },
        }
    }

    pub fn check_debug_flags(hprocess: HANDLE) -> bool {
        match Self::nt_query_core(hprocess, QueryType::DebugFlags) {
            Err(_) => false,
            Ok(x) => match x {
                0 => false,
                _ => true,
            },
        }
    }
}

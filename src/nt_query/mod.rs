use crate::util::BeingDebug;
use anyhow::Result;
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

/// nt_query下的查询调试信息方法类型，
/// 不同类型表示查询进程是否被调试的不同特征
/// 
/// - `DebugPort`: 调试器端口
/// - `DebugObject`: 调试器对象句柄
/// - `DebugFlags`: 调试器标志符
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
    /// 查询指定进程的相关调试信息
    /// 
    /// 传入进程句柄和需要查询的调试信息的方法类型(QueryType)
    /// - `QueryType::DebugPort`，NtQueryInformationProcess返回值为0则没有被调试
    /// - `QueryType::DebugObject`，NtQueryInformationProcess返回值为0则没有被调试
    /// - `QueryType::DebugFlags`，NtQueryInformationProcess返回值为0则没有被调试
    /// 
    /// # 参数
    /// 
    /// - `hprocess`：进程句柄
    /// - `query_type`：查询类型
    ///
    /// # 返回值
    /// 
    /// - `true`: 进程被调试
    /// - `false`: 进程未被调试
    /// 
    /// # 示例
    /// 
    /// ```ignore
    /// let hprocess = unsafe {GetCurrentProcess()};
    /// let result = NtQueryDebug::nt_query_core(hprocess, QueryType::DebugObject).unwarp();
    /// assert_eq!(result, 0);
    /// ```
    pub fn nt_query_core(hprocess: HANDLE, query_type: QueryType) -> bool {
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

        if status != STATUS_SUCCESS && status != STATUS_PORT_NOT_SET {
            // 查询失败，则默认返回false
            warn!("NtQueryInformationProcess failed; error code: {:?}", status);
            return false;
        }

        match process_information {
            0 => false,
            _ => true
        }
    }

    pub fn check_debug_port(hprocess: HANDLE) -> bool {
        Self::nt_query_core(hprocess, QueryType::DebugPort)
    }

    pub fn check_debug_object(hprocess: HANDLE) -> bool {
        Self::nt_query_core(hprocess, QueryType::DebugObject)
    }

    pub fn check_debug_flags(hprocess: HANDLE) -> bool {
        Self::nt_query_core(hprocess, QueryType::DebugFlags)
    }
}

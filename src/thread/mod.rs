use anyhow::{Error, Result};
use log::warn;
use std::ptr::null;
use windows::{
    Wdk::System::Threading::{ThreadHideFromDebugger, ZwSetInformationThread},
    Win32::{
        Foundation::{HANDLE, NTSTATUS, STATUS_SUCCESS},
        System::Threading::GetCurrentThread,
    },
};

/// 禁止指定线程调试事件，如果已经在调试中，则会关闭调试
///
/// # 返回值
///
/// - `Err`: ZwSetInformationThread API调用失败
/// - `Ok(())`: 成功禁止线程调试
pub fn disable_thread_debug(hthread: HANDLE) -> Result<()> {
    let status: NTSTATUS =
        unsafe { ZwSetInformationThread(hthread, ThreadHideFromDebugger, null(), 0) };

    if status != STATUS_SUCCESS {
        warn!("ZwSetInformationThread failed; error code: {:?}", status);
        return Err(Error::msg("ZwSetInformationThread failed"));
    }

    Ok(())
}

/// 禁止当前线程调试事件生成，如果已经在调试中，则会关闭调试
///
/// # 返回值
///
/// - `Err`: disable_thread_debug 函数调用失败
/// - `Ok(())`: 成功禁止当前线程调试
pub fn disable_current_thread_debug() -> Result<()> {
    let hthread: HANDLE = unsafe { GetCurrentThread() };
    disable_thread_debug(hthread)
}

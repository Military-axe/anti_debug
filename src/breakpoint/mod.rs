use crate::util::BeingDebug;
use anyhow::Result;
use log::debug;
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Diagnostics::Debug::{GetThreadContext, CONTEXT},
        Threading::GetCurrentThread,
    },
};

impl BeingDebug for CONTEXT {
    fn is_being_debug(&self) -> bool {
        if self.Dr0 != 0 || self.Dr1 != 0 || self.Dr2 != 0 || self.Dr3 != 0 {
            true
        } else {
            false
        }
    }
}

/// 检测当前线程的Context，判断是否被设置硬件断点
///
/// # 返回值
///
/// - `Err`: GetThreadContext API报错返回
/// - `Ok(true)`: 设置了硬件断点
/// - `Ok(false)`: 未设置硬件断点
///
/// # 示例
///
/// ```ignore
/// if is_hardware_breakpoint_set().unwarp() {
///     println!("Set hardware breakpoint");
/// } else {
///     println!("Do not set hardware breakpoint");
/// }
/// ```
pub fn is_hardware_breakpoint_set() -> Result<bool> {
    let mut context: CONTEXT = CONTEXT::default();
    let thread_hanle: HANDLE = unsafe { GetCurrentThread() };
    unsafe { GetThreadContext(thread_hanle, &mut context) }?;

    debug!(
        "Thread Context ==> Dr0: {}; Dr1: {}; Dr2: {}; Dr3: {}",
        context.Dr0, context.Dr1, context.Dr2, context.Dr3
    );

    Ok(context.is_being_debug())
}

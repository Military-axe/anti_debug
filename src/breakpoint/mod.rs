use crate::util::BeingDebug;
use anyhow::Result;
use log::debug;
use windows::Win32::{
    Foundation::HANDLE,
    System::Diagnostics::Debug::{GetThreadContext, SetThreadContext, CONTEXT},
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

pub struct HardwareBreakPoint {}

impl HardwareBreakPoint {
    /// 检测指定线程的Context，判断是否被设置硬件断点
    ///
    /// # 参数
    ///
    /// - `thread_handle`: 线程句柄
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
    pub fn is_hardware_breakpoint_set(thread_hanle: HANDLE) -> Result<bool> {
        let mut context: CONTEXT = CONTEXT::default();
        unsafe { GetThreadContext(thread_hanle, &mut context) }?;

        debug!(
            "Thread Context ==> Dr0: {}; Dr1: {}; Dr2: {}; Dr3: {}",
            context.Dr0, context.Dr1, context.Dr2, context.Dr3
        );

        Ok(context.is_being_debug())
    }

    /// 清除指定线程的所有硬件断点
    ///
    /// # 参数
    ///
    /// - `thread_handle`: 线程句柄
    ///
    /// # 返回值
    ///
    /// - `Err`: GetThreadContext/SetThreadContext失败
    /// - `Ok(())`: 清空硬件断点成功
    pub fn clean_hardware_breakpoint(thread_hanle: HANDLE) -> Result<()> {
        let mut context: CONTEXT = CONTEXT::default();
        unsafe { GetThreadContext(thread_hanle, &mut context) }?;

        context.Dr0 = 0;
        context.Dr1 = 0;
        context.Dr2 = 0;
        context.Dr3 = 0;

        unsafe { SetThreadContext(thread_hanle, &mut context) }?;

        Ok(())
    }
}

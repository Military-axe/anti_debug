use crate::util::BeingDebug;
use anyhow::{Error, Result};
use log::{debug, error};
use std::{arch::asm, ptr};
use windows::Win32::{
    Foundation::HANDLE,
    System::{Diagnostics::Debug::IsDebuggerPresent, Memory::GetProcessHeap},
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct WinPeb {
    pub reverse1: [u8; 2],
    pub being_debugged: u8,

    #[cfg(target_pointer_width = "32")]
    pub reverse2: [u8; 0x15],
    #[cfg(target_pointer_width = "64")]
    pub reverse2: [u8; 0x2d],

    pub process_heap: *const WinProcessHeap,

    #[cfg(target_pointer_width = "32")]
    pub reverse2: [u8; 0x4c],
    #[cfg(target_pointer_width = "64")]
    pub reverse3: [u8; 0x84],

    pub nt_global_flag: u32,
}

impl AsRef<WinPeb> for u64 {
    fn as_ref(&self) -> &WinPeb {
        unsafe { &*(*self as *const WinPeb) }
    }
}

impl Default for WinPeb {
    fn default() -> Self {
        Self {
            reverse1: Default::default(),
            being_debugged: Default::default(),

            #[cfg(target_pointer_width = "32")]
            reverse2: [u8; 0x15],
            #[cfg(target_pointer_width = "64")]
            reverse2: [0; 0x2d],

            process_heap: ptr::null(),

            #[cfg(target_pointer_width = "32")]
            reverse2: [u8; 0x4c],
            #[cfg(target_pointer_width = "64")]
            reverse3: [0; 0x84],

            nt_global_flag: Default::default(),
        }
    }
}

impl BeingDebug for WinPeb {
    fn is_being_debug(&self) -> bool {
        if self.being_debugged == 0 && self.nt_global_flag != 0x70 {
            false
        } else {
            true
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct WinProcessHeap {
    #[cfg(target_pointer_width = "32")]
    pub reverse1: [u8; 0x40],

    #[cfg(target_pointer_width = "64")]
    pub reverse1: [u8; 0x70],

    pub flags: u32,
    pub force_flags: u32,
}

impl Default for WinProcessHeap {
    fn default() -> Self {
        Self {
            #[cfg(target_pointer_width = "32")]
            reverse1: [0; 0x40],
            #[cfg(target_pointer_width = "64")]
            reverse1: [0; 0x70],

            flags: Default::default(),
            force_flags: Default::default(),
        }
    }
}

impl BeingDebug for WinProcessHeap {
    fn is_being_debug(&self) -> bool {
        if self.flags == 2 && self.force_flags == 0 {
            false
        } else {
            true
        }
    }
}

impl AsRef<WinProcessHeap> for u64 {
    fn as_ref(&self) -> &WinProcessHeap {
        unsafe { &*(self as *const _ as *const WinProcessHeap) }
    }
}

impl AsRef<WinProcessHeap> for HANDLE {
    fn as_ref(&self) -> &WinProcessHeap {
        let value = self.0;
        unsafe { &*(value as *const WinProcessHeap) }
    }
}

impl AsRef<WinProcessHeap> for WinPeb {
    fn as_ref(&self) -> &WinProcessHeap {
        unsafe { &*self.process_heap }
    }
}

impl WinPeb {
    /// 获取进程的PEB地址
    ///
    /// 64位程序获取gs:[0x60]的值，32位程序则获取fs:[0x30]的值
    ///
    /// # 返回值
    ///
    /// 返回一个u64类型的值，这个值就是PEB块的首地址
    pub fn get_peb_address() -> u64 {
        let mut peb_address: u64;

        #[cfg(target_pointer_width = "64")]
        unsafe {
            asm!("mov {}, gs:[0x60]", out(reg) peb_address);
        };

        #[cfg(target_pointer_width = "32")]
        unsafe {
            asm!("mov {}, fs:[0x30]", out(reg) peb_address);
        };

        debug!("peb address ==> {:#x}", peb_address);

        return peb_address;
    }

    /// 检测进程是否被调试
    ///
    /// 通过调用IsDebuggerPresent Win API来判断是否被调试
    ///
    /// # 返回值
    ///
    /// - `true`: 进程未被调试
    /// - `false`：进程正在被调试
    ///
    /// # 示例
    ///
    /// ```ignore
    /// match peb_being_debugged() {
    ///     true => println!("process is not being debugged"),
    ///     false => println!("process is being debugged")
    /// }
    /// ```
    pub fn peb_being_debugged() -> bool {
        return unsafe { IsDebuggerPresent().into() };
    }

    /// 获取peb中指定属性的值来判断进程是否被调试
    ///
    /// peb_being_debugged_asm通过汇编代码检测peb结构体中的BeingDebugged属性值
    /// 如果值不为0则认为正在被调试，返回true，否则返回false
    ///
    /// # 返回值
    ///
    /// - `false`: 进程未被调试
    /// - `true`：进程正在被调试
    ///
    /// # 示例
    ///
    /// ```ignore
    /// match peb_being_debugged_asm() {
    ///     true => println!("process is not being debugged"),
    ///     false => println!("process is being debugged")
    /// }
    /// ```
    pub fn peb_being_debugged_asm() -> bool {
        let peb_address: u64 = Self::get_peb_address();
        let peb_ref: &WinPeb = peb_address.as_ref();

        debug!("PEB.BeingDebugged ==> {:#x}", peb_ref.being_debugged);

        peb_ref.is_being_debug()
    }

    /// 获取peb中指定属性的值来判断进程是否被调试
    ///
    /// peb_nt_global_flag_asm通过汇编代码检测peb结构体中的NtGlobalFlag属性值
    /// 如果值为0x70则认为正在被调试，返回true，否则返回false
    ///
    /// # 返回值
    ///
    /// - `false`: 进程未被调试
    /// - `true`：进程正在被调试
    ///
    /// # 示例
    ///
    /// ```ignore
    /// match peb_nt_global_flag_asm() {
    ///     true => println!("process is not being debugged"),
    ///     false => println!("process is being debugged")
    /// }
    /// ```
    pub fn peb_nt_global_flag_asm() -> bool {
        let peb_address: u64 = Self::get_peb_address();
        let peb_ref: &WinPeb = peb_address.as_ref();

        debug!("PEB.NtGlobalFlag ==> {:#x}", peb_ref.nt_global_flag);

        peb_ref.is_being_debug()
    }

    /// 获取peb.processheap中的flags和force_flags的值来判断进程是否被调试
    ///
    /// # 返回值
    ///
    /// - `Err`: PEB.ProcessHeap的值是null，这是不正常的
    /// - `Ok(false)`: 进程未被调试
    /// - `Ok(true)`：进程正在被调试
    ///
    /// # 示例
    ///
    /// ```ignore
    /// match peb_process_heap_asm() {
    ///     true => println!("process is not being debugged"),
    ///     false => println!("process is being debugged")
    /// }
    /// ```
    pub fn peb_process_heap_asm() -> Result<bool> {
        let peb_address: u64 = Self::get_peb_address();
        let peb_ref: &WinPeb = peb_address.as_ref();

        if peb_ref.process_heap.is_null() {
            error!("PEB.ProcessHeap value: Null is invalid");
            return Err(Error::msg("PEB.ProcessHeap value: Null is invalid"));
        }

        let process_ref: &WinProcessHeap = peb_ref.as_ref();

        debug!(
            "Process Heap address ==> {:#x}",
            process_ref as *const _ as u64
        );
        debug!(
            "HEAP.flags ==> {:?}; HEAP.force_flags ==> {:?}",
            process_ref.flags, process_ref.force_flags
        );

        Ok(process_ref.is_being_debug())
    }

    /// 通过检测ProcessHeap中属性值来判断进程是否被调试
    ///
    /// 使用GetProcessHeap API获取ProcessHeap，通过判断ProcessHeap.flags的值
    /// 是否大于2，或者ProcessHeap.force_flags地址是否不为0来判断是否被调试
    ///
    /// # 返回值
    ///
    /// - `Err`: GetProcessHeap API调用异常
    /// - `Ok(true)`: 进程被调试
    /// - `Ok(false)`: 进程未被调试
    ///
    /// # 注意事项
    ///
    /// 如果进程后运行起来，调试器再attach，此函数无法检测出来进程是否被调试
    ///
    /// # 示例
    ///
    /// ```rust
    /// match peb_process_heap() {
    ///     Err(error_msg) => println!("GetProcessHeap error; {:?}", error_msg),
    ///     Ok(is_being_debug) => match is_being_debug {
    ///         true => println!("process is not being debugged"),
    ///         false => println!("process is being debugged"),
    ///     }
    /// }
    /// ```
    pub fn peb_process_heap() -> Result<bool> {
        let heap_handle: HANDLE = unsafe { GetProcessHeap() }?;
        let process_ref: &WinProcessHeap = heap_handle.as_ref();

        debug!("heap handle ==> {:?}", heap_handle);
        debug!(
            "HEAP.flags ==> {:?}; HEAP.force_flags ==> {:?}",
            process_ref.flags, process_ref.force_flags
        );

        Ok(process_ref.is_being_debug())
    }
}

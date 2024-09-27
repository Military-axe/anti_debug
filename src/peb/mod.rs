use log::debug;
use std::{arch::asm, ptr};
use windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent;

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
    /// - `true`: 进程未被调试
    /// - `false`：进程正在被调试
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
        let peb_raw_ptr: *const WinPeb = peb_address as *const WinPeb;
        let peb_ref: &WinPeb = unsafe { &*peb_raw_ptr };

        debug!("PEB.BeingDebugged ==> {:#x}", peb_ref.being_debugged);

        match peb_ref.being_debugged {
            0 => false,
            _ => true,
        }
    }

    /// 获取peb中指定属性的值来判断进程是否被调试
    ///
    /// peb_nt_global_flag_asm通过汇编代码检测peb结构体中的NtGlobalFlag属性值
    /// 如果值为0x70则认为正在被调试，返回true，否则返回false
    ///
    /// # 返回值
    ///
    /// - `true`: 进程未被调试
    /// - `false`：进程正在被调试
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
        let peb_raw_ptr: *const WinPeb = peb_address as *const WinPeb;
        let peb_ref: &WinPeb = unsafe { &*peb_raw_ptr };

        debug!("PEB.NtGlobalFlag ==> {:#x}", peb_ref.nt_global_flag);

        match peb_ref.nt_global_flag {
            0x70 => true,
            _ => false,
        }
    }

    /// 获取peb.processheap中的flags和force_flags的值来判断进程是否被调试
    ///
    /// # 返回值
    ///
    /// - `true`: 进程未被调试
    /// - `false`：进程正在被调试
    ///
    /// # 示例
    ///
    /// ```ignore
    /// match peb_process_heap_asm() {
    ///     true => println!("process is not being debugged"),
    ///     false => println!("process is being debugged")
    /// }
    /// ```
    pub fn peb_process_heap_asm() -> bool {
        let peb_address: u64 = Self::get_peb_address();
        let peb_raw_ptr: *const WinPeb = peb_address as *const WinPeb;
        let peb_ref: &WinPeb = unsafe { &*peb_raw_ptr };
        let process_ref: &WinProcessHeap = unsafe { &*peb_ref.process_heap };

        debug!(
            "HEAP.flags ==> {:?}; HEAP.force_flags ==> {:?}",
            process_ref.flags, process_ref.force_flags
        );

        if process_ref.flags > 2 || process_ref.force_flags != 0 {
            false
        } else {
            true
        }
    }
}

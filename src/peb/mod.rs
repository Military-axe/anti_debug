use log::debug;
use std::arch::asm;
use windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent;

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
    let peb_address: u64 = get_peb_address();

    let mut peb_being_debugged_value: u64;
    unsafe {
        asm!(
            "add {1}, 0x2",
            "movzx {0}, byte ptr [{1}]",
            out(reg) peb_being_debugged_value,
            in(reg) peb_address,
        );
    };

    debug!("PEB.BeingDebugged ==> {:#x}", peb_being_debugged_value);

    match peb_being_debugged_value {
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
    let peb_address: u64 = get_peb_address();
    let mut peb_nt_global_flag_value: u64;

    #[cfg(target_pointer_width = "32")]
    let mut nt_global_flag_offset: u64 = 0x68;

    #[cfg(target_pointer_width = "64")]
    let nt_global_flag_offset: u64 = 0xbc;

    // peb_nt_global_flag_value = [nt_global_flag_offset + peb_address]
    unsafe {
        asm!(
            "add {0}, {2}",
            "mov {1:e}, [{0}]",
            in(reg) peb_address,
            out(reg) peb_nt_global_flag_value,
            in(reg) nt_global_flag_offset,
        );
    };

    debug!("PEB.NtGlobalFlag ==> {:#x}", peb_nt_global_flag_value );

    match peb_nt_global_flag_value {
        0x70 => true,
        _ => false,
    }
}

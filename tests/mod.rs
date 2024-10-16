use anti_debug::{breakpoint, nt_query, peb::*, thread, util::BeingDebug};
use windows::Win32::System::Threading::GetCurrentThread;

#[test]
pub fn peb_being_debugged_test() {
    assert_eq!(WinPeb::peb_being_debugged(), false);
}

#[test]
pub fn peb_being_debugged_asm_test() {
    assert_eq!(WinPeb::peb_being_debugged_asm(), false);
}

#[test]
pub fn peb_nt_global_flag_asm_test() {
    assert_eq!(WinPeb::peb_nt_global_flag_asm(), false);
}

#[test]
pub fn peb_process_heap_asm_test() {
    assert_eq!(
        WinPeb::peb_process_heap_asm().expect("PEB.ProcessHeap value is invalid"),
        false
    );
}

#[test]
pub fn peb_process_heap_test() {
    assert_eq!(
        WinPeb::peb_process_heap().expect("GetProcessHeap error"),
        false
    );
}

#[test]
pub fn hardware_breakpoint_test() {
    let hthread = unsafe {GetCurrentThread()};
    assert_eq!(
        breakpoint::HardwareBreakPoint::is_hardware_breakpoint_set(hthread).expect("GetThreadContext error"),
        false
    );

    assert!(breakpoint::HardwareBreakPoint::clean_hardware_breakpoint(hthread).is_ok());
}

#[test]
pub fn nt_query_debug_test() {
    let anti = nt_query::NtQueryDebug {};
    assert_eq!(anti.is_being_debug(), false)
}

#[test]
pub fn honey_thread_test() {
    let mut t = thread::HoneyThread::default();
    t.set_honey_thread_current_process().unwrap();
    assert_eq!(
        t.check().unwrap(),
        false
    )
}

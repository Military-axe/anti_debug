use crate::{breakpoint, nt_query, peb::*, thread, util::BeingDebug};

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
    assert_eq!(
        breakpoint::is_hardware_breakpoint_set().expect("GetThreadContext error"),
        false
    );
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

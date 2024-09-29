use crate::{breakpoint, peb::*};

#[test]
pub fn peb_being_debugged_test() {
    assert_eq!(WinPeb::peb_being_debugged(), true);
}

#[test]
pub fn peb_being_debugged_asm_test() {
    assert_eq!(WinPeb::peb_being_debugged_asm(), true);
}

#[test]
pub fn peb_nt_global_flag_asm_test() {
    assert_eq!(WinPeb::peb_nt_global_flag_asm(), true);
}

#[test]
pub fn peb_process_heap_asm_test() {
    assert_eq!(
        WinPeb::peb_process_heap_asm().expect("PEB.ProcessHeap value is invalid"),
        true
    );
}

#[test]
pub fn peb_process_heap_test() {
    assert_eq!(
        WinPeb::peb_process_heap().expect("GetProcessHeap error"),
        true
    );
}

#[test]
pub fn hardware_breakpoint_test() {
    assert_eq!(breakpoint::is_hardware_breakpoint_set().expect("GetThreadContext error"), false);
}
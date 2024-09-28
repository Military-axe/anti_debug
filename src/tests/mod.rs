use crate::peb::*;

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
    assert_eq!(WinPeb::peb_process_heap_asm(), true);
}

#[test]
pub fn peb_process_heap_test() {
    assert_eq!(
        WinPeb::peb_process_heap().expect("GetProcessHeap error"),
        true
    );
}

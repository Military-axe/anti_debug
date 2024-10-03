use crate::{breakpoint, nt_query, peb::*};

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
pub fn check_remote_debugger_present_test() {
    assert_eq!(
        nt_query::DebugPort::check_remote_debugger_present()
            .expect("CheckRemoteDebuggerPresent error"),
        false
    );
}

#[test]
pub fn nt_query_debug_port_test() {
    assert_eq!(nt_query::DebugPort::nt_query_debug_port(), false);
}

use crate::peb::*;

#[test]
pub fn peb_being_debugged_test() {
    match WinPeb::peb_being_debugged() {
        true => println!("peb_being_debugged: true"),
        false => println!("peb_being_debugged: false"),
    }
}

#[test]
pub fn peb_being_debugged_asm_test() {
    match WinPeb::peb_being_debugged_asm() {
        true => println!("peb_being_debugged_asm: true"),
        false => println!("peb_being_debugged_asm: false"),
    }
}

#[test]
pub fn peb_nt_global_flag_asm_test() {
    match WinPeb::peb_nt_global_flag_asm() {
        true => println!("peb_being_debugged_asm: true"),
        false => println!("peb_being_debugged_asm: false"),
    }
}

#[test]
pub fn peb_process_heap_asm_test() {
    match WinPeb::peb_process_heap_asm() {
        true => println!("peb_being_debugged_asm: true"),
        false => println!("peb_being_debugged_asm: false"),
    }
}

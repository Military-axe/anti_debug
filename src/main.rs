use std::env::set_var;

use log::debug;

pub mod peb;
pub mod util;
pub mod breakpoint;
pub mod exception;
pub mod nt_query;
pub mod thread;
#[cfg(test)]
pub mod tests;

fn main() {
    #[cfg(debug_assertions)]
    set_var("RUST_LOG", "debug");

    #[cfg(not(debug_assertions))]
    set_var("RUST_LOG", "warn");

    env_logger::init();

    let _ = peb::WinPeb::peb_being_debugged();
    let _ = peb::WinPeb::peb_being_debugged_asm();
    let _ = peb::WinPeb::peb_nt_global_flag_asm();
    let _ = peb::WinPeb::peb_process_heap_asm();
    let _ = peb::WinPeb::peb_process_heap();
    let _ = breakpoint::is_hardware_breakpoint_set();
    let _ = thread::disable_current_thread_debug();
    let mut t = thread::HoneyThread::default();
    t.set_honey_thread_current_process().unwrap();
    t.check().unwrap();

    util::pause();

    debug!("Anti Debug End");
}

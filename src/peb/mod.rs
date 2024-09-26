use windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent;


pub fn peb_being_debugged() -> bool {
    return unsafe { IsDebuggerPresent().into() };
}

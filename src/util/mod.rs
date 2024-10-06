use std::io::{self, Write};

pub trait BeingDebug{
    fn is_being_debug(&self) -> bool; 
}

pub fn pause() {
    print!("Press Enter to continue...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
use crate::peb::*;

#[test]
pub fn peb_being_debugged_test() {
    match peb_being_debugged() {
        true => println!("peb_being_debugged: true"),
        false => println!("peb_being_debugged: false"),
    }
}
use crate::constants::{LOGO, VERSION, COPYRIGHT};

pub fn print_logo() {
    println!("{}", LOGO);
    println!("Lycrex Tool Version: {}", VERSION);
    // println!("Author: {}", AUTHOR);
    println!("{}", COPYRIGHT);
} 
pub mod constants;
pub mod print;
// pub mod utils;
#[cfg(feature = "win-memory")]
pub mod memory;

pub use constants::*;
pub use print::*;
// pub use utils::*;
#[cfg(feature = "win-memory")]
pub use memory::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_logo() {
        print_logo();
    }
}

#![no_std]

mod funcs;
mod types;

pub use funcs::*;
pub use types::*;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[path = "raw/x86_64.rs"]
pub mod raw;

#[cfg(all(target_os = "linux", target_arch = "x86"))]
#[path = "raw/x86.rs"]
pub mod raw;

#[cfg(all(target_os = "linux", target_arch = "arm"))]
#[path = "raw/arm.rs"]
pub mod raw;

#[cfg(all(target_os = "linux", target_arch = "riscv64"))]
#[path = "raw/riscv64.rs"]
pub mod raw;

#[cfg(test)]
mod tests;

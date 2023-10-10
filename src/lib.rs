#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]

extern crate alloc;

pub mod global_allocator;
pub mod flash;
pub mod utils;

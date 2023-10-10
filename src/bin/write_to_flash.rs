#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use defmt::info;
use defmt_rtt as _;
use rp_pico::entry;
use panic_probe as _;
use app::{global_allocator, utils};

const TEST_ADDR: u32 = 0x10300000;

#[entry]
unsafe fn main() -> ! {
    global_allocator::init_allocator();

    // Generate a 8K long string composed of "Hello, World!"
    let base_string = String::from("Hello, World!");
    let repetitions = (8 * 1024) / base_string.len();
    let long_string = base_string.repeat(repetitions);

    if let Err(e) = utils::write_bytes_to_flash(TEST_ADDR, long_string.as_bytes()) {
        info!("Error: {}", e);
    } else {
        info!("String written successfully!");
    }

    loop {}
}

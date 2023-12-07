#![no_std]
#![no_main]

extern crate alloc;

use defmt::info;
use rp_pico::entry;
use panic_probe as _;
use defmt_rtt as _;
use app::{global_allocator, utils};

#[entry]
unsafe fn main() -> ! {
    global_allocator::init_allocator();
    if let Some(read_string) = utils::read_string_from_flash(utils::EDS_DATA_ADDRESS) {
        info!("Read string: {}", read_string.as_str());
        info!("Length of the string: {}", read_string.len());
    } else {
        panic!("Error reading string from flash.");
    }

    loop {}
}

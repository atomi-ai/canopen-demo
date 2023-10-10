#![no_std]
#![no_main]

use bsp::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;
use app::flash::flash;

const FLASH_BASE_ADDR: u32 = 0x10000000;
// this point to 3M address, because the flash size is 4M, and the 7M addr
// will be round as 3M based on this.
const TEST_ADDR: u32 = 0x10700000;

#[entry]
unsafe fn main() -> ! {
    let flash_ptr = TEST_ADDR as *const [u8; 4096];
    let mut curr = *flash_ptr;

    info!("Program start");
    info!("Contents start with {=[u8]}", curr[0..7]);

    let mut to_write = *flash_ptr;
    to_write[0] = ((to_write[0] as u32 + 1) % 0x100) as u8;
    info!("Contents start with {=[u8]}", to_write[0..7]);

    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    cortex_m::interrupt::free(|_cs| {
        flash::flash_range_erase_and_program(TEST_ADDR - FLASH_BASE_ADDR, &to_write, true);
    });
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    curr = *flash_ptr;
    info!("Contents start with {=[u8]}", curr[0..7]);

    loop {}
}
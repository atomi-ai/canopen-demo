// TODO(zephyr): Remove the file

#![no_std]
#![no_main]

use bsp::entry;
use core::cell::UnsafeCell;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use app::flash::flash;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[repr(C, align(4096))]
struct FlashBlock {
    data: UnsafeCell<[u8; 4096]>,
}

impl FlashBlock {
    #[inline(never)]
    fn addr(&self) -> u32 {
        &self.data as *const _ as u32
    }

    #[inline(never)]
    fn read(&self) -> &[u8; 4096] {
        // Make sure the compiler can't know that
        // we actually access a specific static
        // variable, to avoid unexpected optimizations
        //
        // (Don't try this with strict provenance.)
        let addr = self.addr();

        unsafe { &*(&*(addr as *const Self)).data.get() }
    }

    unsafe fn write_flash(&self, data: &[u8; 4096]) {
        let addr = self.addr() - 0x10000000;
        defmt::assert!(addr & 0xfff == 0);

        cortex_m::interrupt::free(|_cs| {
            flash::flash_range_erase_and_program(addr, data, true);
        });
    }
}

// TODO safety analysis - this is probably not sound
unsafe impl Sync for FlashBlock {}

const TEST_ADDR: u32 = 0x10700000;

fn test_block() -> &'static FlashBlock {
    unsafe { &*(TEST_ADDR as *const FlashBlock) }
}
//
// const INIT_DATA: u8 = 0x59u8;
// #[link_section = ".flash_data"]
// static TEST: FlashBlock = FlashBlock {
//     data: UnsafeCell::new([INIT_DATA; 4096]),
// };

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    // add some delay to give an attached debug probe time to parse the
    // defmt RTT header. Reading that header might touch flash memory, which
    // interferes with flash write operations.
    // https://github.com/knurling-rs/defmt/pull/683
    delay.delay_ms(10);

    let _pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let psm = pac.PSM;

    // Reset core1 so it's guaranteed to be running
    // ROM code, waiting for the wakeup sequence
    psm.frce_off.modify(|_, w| w.proc1().set_bit());
    while !psm.frce_off.read().proc1().bit_is_set() {
        cortex_m::asm::nop();
    }
    psm.frce_off.modify(|_, w| w.proc1().clear_bit());

    let jedec_id: u32 = unsafe { cortex_m::interrupt::free(|_cs| flash::flash_jedec_id(true)) };
    info!("JEDEC ID {:x}", jedec_id);
    let mut unique_id = [0u8; 8];
    unsafe { cortex_m::interrupt::free(|_cs| flash::flash_unique_id(&mut unique_id, true)) };
    info!("Unique ID {:x}", unique_id);

    let mut current_data: [u8; 4096] = *test_block().read();
    info!("Addr of flash block is {:x}", test_block().addr());
    info!("Contents start with {=[u8]}", current_data[0..7]);

    current_data[0] = current_data[0].wrapping_add(1);
    current_data[1] = current_data[1].wrapping_add(2);
    current_data[2] = current_data[2].wrapping_add(3);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    unsafe { test_block().write_flash(&current_data) };
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

    let read_data: [u8; 4096] = *test_block().read();
    info!("Contents start with {=[u8]}", read_data[0..7]);
    info!("Contents start with {=[u8]}", current_data[0..7]);

    loop {}
}

// End of file
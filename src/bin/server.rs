//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
// TODO(zephyr): Reorganize imports.
#![no_std]
#![no_main]

extern crate alloc;
extern crate alloc_cortex_m;

// reserved
use defmt_rtt as _;
use panic_probe as _;

use bsp::entry;
use bsp::hal::{
    clocks::{Clock, init_clocks_and_plls},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use defmt::*;
use embedded_can::Frame;
use rp_pico as bsp;

use app::global_allocator;
use app::utils::{EDS_DATA_ADDRESS, read_string_from_flash};
use can2040;
use canopen_rust::node;

//
// #[defmt::timestamp]
// fn timestamp() -> defmt::Timestamp {
//     // Implement your timestamp logic here, or leave it empty if not required.
//     defmt::Timestamp::from_ticks(0)
// }

#[defmt::panic_handler]
fn panic() -> ! {
    loop {}
}

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);
    global_allocator::init_allocator();

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog, )
        .ok()
        .unwrap();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // init the correct led_pin for Pico Pi.
    let mut led_pin = pins.led.into_push_pull_output();

    let mut can_bus = can2040::initialize_cbus(&mut core);

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let eds_content = read_string_from_flash(EDS_DATA_ADDRESS).unwrap();
    let mut node = node::Node::new(0x2, eds_content.as_str(), can_bus).expect("TODO: panic message");
    node.init().expect("TODO: panic message");

    loop {
        let free_bytes = global_allocator::ALLOCATOR.free();
        // info!("Free bytes in heap: {}", free_bytes);
        node.process_one_frame();
        // delay.delay_ms(500);
    }
}

// End of file
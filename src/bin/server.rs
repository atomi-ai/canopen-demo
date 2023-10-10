//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
// TODO(zephyr): Reorganize imports.
#![no_std]
#![no_main]

use alloc::boxed::Box;
use core::cell::RefCell;
use can2040;

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;

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
use canopen::node;
use can2040::CanFrame;
use embedded_can::{Frame, Id};

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

extern crate alloc_cortex_m;
extern crate alloc;

use embedded_can::StandardId;
use fugit::{Duration, MicrosDurationU32};
use rp2040_hal::pac::interrupt;
use rp2040_hal::Timer;
use rp2040_hal::timer::{Alarm, ScheduleAlarmError};
use app::{global_allocator, utils};
use app::utils::{EDS_DATA_ADDRESS, read_string_from_flash};

#[entry]
fn main() -> ! {
    // defmt::rtt::Logger::init();
    // rtt_target::rtt_init_print!();

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

    use embedded_can::blocking::Can;

    let mut can_bus = can2040::initialize_cbus(&mut core);

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let eds_content = read_string_from_flash(EDS_DATA_ADDRESS).unwrap();
    let mut node = node::Node::new(0x2, eds_content.as_str(), Box::new(can_bus));
    node.init();

    loop {
        let free_bytes = global_allocator::ALLOCATOR.free();
        info!("Free bytes in heap: {}", free_bytes);
        node.process_one_frame();
    }

    // let mut t: u8 = 0xfe;
    // loop {
    //     info!("on!");
    //     let free_bytes = global_allocator::ALLOCATOR.free();
    //     info!("Free bytes in heap: {}", free_bytes);
    //
    //     led_pin.set_high().unwrap();
    //
    //     // main logic here.
    //     let id = Id::Standard(StandardId::new(0x607).unwrap());
    //     t = if t == 0xff { 0x0 } else { t + 1 };
    //     let frame = CanFrame::new(id, &[0x55, 0x66, 0x77, t]).unwrap();
    //     debug!("try to transmit frame {:?}", frame);
    //     can_bus.transmit(&frame);
    //     can_bus.receive();
    //
    //
    //     delay.delay_ms(500);
    //     info!("off!");
    //     led_pin.set_low().unwrap();
    //     delay.delay_ms(500);
    // }
}

// End of file
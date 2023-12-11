//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
// TODO(zephyr): Reorganize imports.
#![no_std]
#![no_main]

extern crate alloc;
extern crate alloc_cortex_m;

use core::cell::RefCell;
// reserved
use defmt_rtt as _;
use panic_probe as _;

use cortex_m_rt::entry;
use critical_section::Mutex;
use defmt::*;
use embedded_can::Frame;
use fugit::MicrosDurationU32;
use rp2040_hal::{Clock, pac, Sio, Timer, Watchdog};
use rp2040_hal::clocks::init_clocks_and_plls;
use rp2040_hal::pac::interrupt;
use rp2040_hal::timer::{Alarm, Alarm0};
use rp_pico::Pins;

use app::global_allocator;
use app::utils::{EDS_DATA_ADDRESS, log_free_bytes, read_string_from_flash};
use can2040;
use canopen_rust::node;

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

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // init the correct led_pin for Pico Pi.
    let mut led_pin = pins.led.into_push_pull_output();

    let mut can_bus = can2040::initialize_cbus(&mut core);

    // TODO(zephyr): Debug memory usage in object_directory. TOO HIGH MEMORY USAGE now.
    let eds_content = read_string_from_flash(EDS_DATA_ADDRESS).unwrap();
    let mut node = node::Node::new(0x2, eds_content.as_str(), can_bus).expect("TODO: panic message");
    node.init().expect("TODO: panic message");
    //
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    // let mut alarm = timer.alarm_0().unwrap();
    // critical_section::with(|cs| {
    //     let _ = alarm.schedule(TIMER_INTERVAL_US);
    //     alarm.enable_interrupt();
    //     unsafe {
    //         CS_VARS.borrow(cs).replace(Some((timer, alarm)));
    //     }
    // });
    // unsafe {
    //     pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    // }

    let mut next_rounded = u64::MAX;
    loop {
        // let free_bytes = global_allocator::ALLOCATOR.free();
        // info!("Free bytes in heap: {}", free_bytes);
        node.event_timer_callback();
        node.process_one_frame();

        let curr = timer.get_counter().ticks();

        if curr > next_rounded {
            error!("Missed one round, {}", curr);
        }
        let rounded : u64 = (curr + TIMER_INTERVAL_TICKS) / TIMER_INTERVAL_TICKS * TIMER_INTERVAL_TICKS;
        trace!("time log, curr = {}, rounded = {}, next_rounded = {}", curr, rounded, next_rounded);
        next_rounded = rounded + TIMER_INTERVAL_TICKS;
        delay.delay_us((rounded - curr) as u32);
    }
}

const TIMER_INTERVAL_US: MicrosDurationU32 = MicrosDurationU32::millis(20);
const TIMER_INTERVAL_TICKS: u64 = TIMER_INTERVAL_US.ticks() as u64;
//
// type CsVars = (
//     Timer,
//     Alarm0,
// );
// static mut CS_VARS: Mutex<RefCell<Option<CsVars>>> = Mutex::new(RefCell::new(None));

//
// #[interrupt]
// fn TIMER_IRQ_0() {
//     info!("TIMER_IRQ_0: 0");
//     critical_section::with(|cs| {
//         let cs_vars = unsafe { CS_VARS.borrow(cs).take() };
//         if let Some((mut timer, mut alarm)) = cs_vars {
//             alarm.clear_interrupt();
//             let cur = timer.get_counter().ticks();
//             let next_rounded = (cur + TIMER_INTERVAL_TICKS) / TIMER_INTERVAL_TICKS * TIMER_INTERVAL_TICKS;
//             let next_trigger = fugit::Instant::<u64, 1, 1_000_000>::from_ticks(next_rounded);
//             info!("xfguo: counter = {:?},  next triggering = {:?}", cur, next_rounded);
//             alarm.schedule_at(next_trigger).unwrap();
//             unsafe {
//                 CS_VARS.borrow(cs).replace_with(|_| Some((timer, alarm)));
//             }
//         }
//     });
//     info!("TIMER_IRQ_0: 9");
// }

// End of file
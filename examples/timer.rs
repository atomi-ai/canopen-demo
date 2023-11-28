#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;
use fugit::MicrosDurationU32;
use rp2040_hal::pac::interrupt;
use rp2040_hal::{pac, clocks::{init_clocks_and_plls, Clock}, sio::Sio, watchdog::Watchdog, timer::Timer, entry};
use panic_probe as _;
use rp2040_hal::timer::{Alarm, Alarm0};


#[defmt::panic_handler]
fn panic() -> ! {
    loop {}
}

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

const TIMER_INTERVAL_US: MicrosDurationU32 = MicrosDurationU32::millis(500);
const TIMER_INTERVAL_TICKS: u64 = TIMER_INTERVAL_US.ticks() as u64;

type CsVars = (
    Timer,
    Alarm0,
);
static mut CS_VARS: Mutex<RefCell<Option<CsVars>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);
    let clocks = init_clocks_and_plls(
        12_000_000u32,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,)
        .ok()
        .unwrap();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut alarm = timer.alarm_0().unwrap();
    critical_section::with(|cs| {
        let _ = alarm.schedule(TIMER_INTERVAL_US);
        alarm.enable_interrupt();
        unsafe {
            CS_VARS.borrow(cs).replace(Some((timer, alarm)));
        }
    });
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    }

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    loop {
        info!("in loop");
        delay.delay_ms(5000);
    }
}

// Trigger every 500ms.
#[interrupt]
fn TIMER_IRQ_0() {
    info!("TIMER_IRQ_0: 0");
    critical_section::with(|cs| {
        let cs_vars = unsafe { CS_VARS.borrow(cs).take() };
        if let Some((mut timer, mut alarm)) = cs_vars {
            alarm.clear_interrupt();
            let cur = timer.get_counter().ticks();
            let next_rounded = (cur + TIMER_INTERVAL_TICKS) / TIMER_INTERVAL_TICKS * TIMER_INTERVAL_TICKS;
            let next_trigger = fugit::Instant::<u64, 1, 1_000_000>::from_ticks(next_rounded);
            info!("xfguo: counter = {:?},  next triggering = {:?}", cur, next_rounded);
            alarm.schedule_at(next_trigger).unwrap();
            unsafe {
                CS_VARS.borrow(cs).replace_with(|_| Some((timer, alarm)));
            }
        }
    });
    info!("TIMER_IRQ_0: 9");
}

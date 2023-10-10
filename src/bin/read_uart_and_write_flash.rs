#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use app::{global_allocator, utils};

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;
use bsp::{entry, hal};
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use hal::uart::{DataBits, StopBits, UartConfig};
use rp_pico::hal::uart::UartPeripheral;
use fugit::RateExtU32;
use embedded_hal::serial::Read;

#[entry]
fn main() -> ! {
    info!("Store_to_littlefs starts");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);
    global_allocator::init_allocator();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
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

    let uart_pins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        pins.gpio0.into_function(),
        // UART RX (characters received by RP2040) on pin 2 (GPIO1)
        pins.gpio1.into_function(),
    );
    let mut uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(115200_u32.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    info!("Wait for size from UART");
    // Read size from UART. We expect the size to be sent as 4 bytes.
    let mut size_bytes = [0u8; 4];
    for byte in size_bytes.iter_mut() {
        *byte = loop {
            if let Some(b) = uart.read().ok() {
                break b;
            }
        };
    }
    let size = u32::from_le_bytes(size_bytes);
    info!("Data size is {}", size);

    // Now read the bytes as per the size from UART.
    let mut buffer = Vec::new();
    for _ in 0..size {
        let byte = loop {
            if let Some(b) = uart.read().ok() {
                break b;
            }
        };
        buffer.push(byte);
    }
    info!("buffer is: {}", buffer[..20]);


    // write this to the flash using the write function.
    if let Err(e) = utils::write_bytes_to_flash(utils::EDS_DATA_ADDRESS, &buffer) {
        info!("Error: {}", e);
    } else {
        info!("Data written successfully!");
    }

    loop {}
}

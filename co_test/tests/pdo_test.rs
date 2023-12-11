use socketcan::Socket;

use canopen_rust::error::ErrorCode;
use co_test::pdo_func::{rpdo_event_driven_mode, rpdo_sync_mode, tpdo_event_driven_mode, tpdo_sync_mode};
use co_test::util::VCAN0_INTERFACE;

use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_tpdo_event_driven_mode() -> Result<(), ErrorCode> {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");

    tpdo_event_driven_mode(&s, VCAN0_INTERFACE)
}

#[test]
fn test_tpdo_sync_mode() -> Result<(), ErrorCode> {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");

    tpdo_sync_mode(&s, VCAN0_INTERFACE)
}

#[test]
fn test_rpdo_event_driven_mode() -> Result<(), ErrorCode> {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");

    rpdo_event_driven_mode(&s)
}

#[test]
fn test_rpdo_sync_mode() -> Result<(), ErrorCode> {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");

    rpdo_sync_mode(&s)
}
use socketcan::{CanSocket, Socket};

use co_test::restore_func::{error_restore_params, restore_all_params, restore_application_params, restore_communication_params};
use co_test::util::VCAN0_INTERFACE;

use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_restore_all_params() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");
    restore_all_params(&s);
}

#[test]
fn test_restore_communication_params() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");
    restore_communication_params(&s);
}

#[test]
fn test_restore_application_params() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");
    restore_application_params(&s);
}

#[test]
fn test_error_restore_params() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");

    error_restore_params(&s, 1);
    error_restore_params(&s, 2);
    error_restore_params(&s, 3);
}

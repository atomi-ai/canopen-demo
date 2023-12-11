use socketcan::Socket;

use co_test::emergency_func::emergency_basic;
use co_test::util::VCAN0_INTERFACE;

use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_emergency_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");

    emergency_basic(&s);
}
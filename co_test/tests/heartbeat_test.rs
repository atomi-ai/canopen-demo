use socketcan::Socket;

use co_test::heartbeat_func::heartbeat_basic;
use co_test::util::VCAN0_INTERFACE;

use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_heartbeat_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");
    heartbeat_basic(&s);
}
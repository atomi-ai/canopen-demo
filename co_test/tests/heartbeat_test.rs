use socketcan::Socket;
use co_test::util::{exp, INTERFACE_NAME, send};
use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_heartbeat_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, 0x602, 0x2B_17_10_00_64_00_00_00, 8);
    exp(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);
    for _ in 0..3 {
        exp(&s, 0x702, 0x0, 1);
    }
}
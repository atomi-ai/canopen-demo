use socketcan::Socket;
use co_test::util::{expf, INTERFACE_NAME, sendf};
use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_emergency_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    // Disable TPDOs
    sendf(&s, 0x602, 0x23_00_18_01_82_01_00_C0, 8);
    expf(&s, 0x582, 0x60_00_18_01_00_00_00_00, 8);
    sendf(&s, 0x602, 0x23_01_18_01_82_02_00_C0, 8);
    expf(&s, 0x582, 0x60_01_18_01_00_00_00_00, 8);

    // Set state to pre-operational and set 0x1800 related.
    sendf(&s, 0x000, 0x80_02, 2);
    // Write value C0000202h to object 1400h:01h
    sendf(&s, 0x602, 0x23_00_14_01_02_02_00_C0, 8);
    expf(&s, 0x582, 0x60_00_14_01_00_00_00_00, 8);
    // Write value 0Ah to object 1400h:02h
    sendf(&s, 0x602, 0x2F_00_14_02_0A_00_00_00, 8);
    expf(&s, 0x582, 0x60_00_14_02_00_00_00_00, 8);
    // Write value 0h to object 1400h:05h
    sendf(&s, 0x602, 0x2B_00_14_05_00_00_00_00, 8);
    expf(&s, 0x582, 0x60_00_14_05_00_00_00_00, 8);
    // Write value 40000202h to object 1400h:01h, enable the RPDO object
    sendf(&s, 0x602, 0x23_00_14_01_02_02_00_40, 8);
    expf(&s, 0x582, 0x60_00_14_01_00_00_00_00, 8);

    sendf(&s, 0x602, 0x40_00_16_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x4F_00_16_00_02_00_00_00, 8);
    sendf(&s, 0x602, 0x40_00_16_01_00_00_00_00, 8);
    expf(&s, 0x582, 0x43_00_16_01_08_01_00_62, 8);
    sendf(&s, 0x602, 0x40_00_16_02_00_00_00_00, 8);
    expf(&s, 0x582, 0x43_00_16_02_08_02_00_62, 8);

    sendf(&s, 0x602, 0x2F_00_62_01_00_00_00_00, 8);
    expf(&s, 0x582, 0x60_00_62_01_00_00_00_00, 8);
    sendf(&s, 0x602, 0x2F_00_62_02_00_00_00_00, 8);
    expf(&s, 0x582, 0x60_00_62_02_00_00_00_00, 8);

    // Set device to Operational
    sendf(&s, 0x000, 0x01_02, 2);

    sendf(&s, 0x202, 0x0A, 1);
    expf(&s, 0x082, 0x10_82_00_02_02_00_00_00, 8);
    expf(&s, 0x082, 0x00_00_00_02_02_00_00_00, 8);

    sendf(&s, 0x202, 0x0B, 1);
    expf(&s, 0x082, 0x10_82_00_02_02_00_00_00, 8);
    expf(&s, 0x082, 0x00_00_00_02_02_00_00_00, 8);

    sendf(&s, 0x602, 0x40_03_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x4F_03_10_00_02_00_00_00, 8);
    sendf(&s, 0x602, 0x40_03_10_01_00_00_00_00, 8);
    expf(&s, 0x582, 0x43_03_10_01_10_82_00_00, 8);
    sendf(&s, 0x602, 0x40_03_10_02_00_00_00_00, 8);
    expf(&s, 0x582, 0x43_03_10_02_10_82_00_00, 8);
    sendf(&s, 0x602, 0x40_01_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x4F_01_10_00_00_00_00_00, 8);
}
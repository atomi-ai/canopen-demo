/*
The file document was separated from sdo_test.rs.

Since the block size on the server side can be modified by incoming
requests, I've moved all test cases that require the default block
size to this file.
 */
use socketcan::Socket;

use co_test::util::{exp, send};
use co_test::util as tu;

use crate::testing::CONTEXT;

mod testing;

#[test]
// SDO 21, write
fn test_block_download_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, 0x602, 0xC2_17_10_00_02_00_00_00, 8);
    exp(&s, 0x582, 0xA4_17_10_00_7F_00_00_00, 8);
    send(&s, 0x602, 0x81_00_00_00_00_00_00_00, 8);
    exp(&s, 0x582, 0xA2_01_7F_00_00_00_00_00, 8);
    send(&s, 0x602, 0xD5_00_00_00_00_00_00_00, 8);
    exp(&s, 0x582, 0xA1_00_00_00_00_00_00_00, 8);
}

#[test]
// SDO 25, write
// Where is CRC?
fn test_block_download_with_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, 0x602, 0xC6_17_10_00_02_00_00_00, 8);
    exp(&s, 0x582, 0xA4_17_10_00_7F_00_00_00, 8);
    send(&s, 0x602, 0x81_00_00_00_00_00_00_00, 8);
    exp(&s, 0x582, 0xA2_01_7F_00_00_00_00_00, 8);
    send(&s, 0x602, 0xD5_00_00_00_00_00_00_00, 8);
    exp(&s, 0x582, 0xA1_00_00_00_00_00_00_00, 8);
}

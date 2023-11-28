/*
The file document was separated from sdo_test.rs.

Since the block size on the server side can be modified by incoming
requests, I've moved all test cases that require the default block
size to this file.
 */
mod testing;

use crate::testing::CONTEXT;
use socketcan::Socket;
use canopen::util::genf_and_padding;
use co_test::util::{exp, send};
use co_test::util as tu;

#[test]
// SDO 21, write
fn test_block_download_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf_and_padding(0x602, &[0xC2, 0x17, 0x10, 0x00, 0x02, 0, 0, 0]));
    exp(&s, &genf_and_padding(0x582, &[0xA4, 0x17, 0x10, 0x00, 0x7F, 0, 0, 0]));

    send(&s, &genf_and_padding(0x602, &[0x81, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf_and_padding(0x582, &[0xA2, 0x01, 0x7F, 0, 0, 0, 0, 0]));

    send(&s, &genf_and_padding(0x602, &[0xD5, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf_and_padding(0x582, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

#[test]
// SDO 25, write
// Where is CRC?
fn test_block_download_with_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf_and_padding(0x602, &[0xC6, 0x17, 0x10, 0x00, 0x02, 0, 0, 0]));
    exp(&s, &genf_and_padding(0x582, &[0xA4, 0x17, 0x10, 0x00, 0x7F, 0, 0, 0]));

    send(&s, &genf_and_padding(0x602, &[0x81, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf_and_padding(0x582, &[0xA2, 0x01, 0x7F, 0, 0, 0, 0, 0]));

    send(&s, &genf_and_padding(0x602, &[0xD5, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf_and_padding(0x582, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

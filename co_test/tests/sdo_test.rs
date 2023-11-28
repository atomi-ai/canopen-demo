
mod testing;

use crate::testing::CONTEXT;
use socketcan::Socket;
use co_test::{sdo_block_upload_string_with_crc, sdo_block_upload_string_without_crc, sdo_block_upload_with_wrong_ack_seqno, sdo_block_upload_with_wrong_blocksize, sdo_block_upload_without_crc, sdo_error_mismatch_length, sdo_error_read, sdo_error_write, sdo_expedite_read, sdo_segment_download_basic, sdo_segment_upload, sdo_segment_upload_with_toggle_bit_error, sdo_with_node_id_in_expr, sdo_write_and_read, util as tu};

#[test]
fn test_write_and_read() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_write_and_read(&s);
}

#[test]
fn test_error_write() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_error_write(&s);
}

#[test]
fn test_error_read() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_error_read(&s);
}

#[test]
// Expedite upload
fn test_read_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_expedite_read(&s);
}

#[test]
// SDO 08 & 09
fn test_error_mismatch_length() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_error_mismatch_length(&s);
}

#[test]
// SDO 12
fn test_with_node_id_in_expr() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_with_node_id_in_expr(&s);
}

#[test]
// SDO 15
fn test_segment_download_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_segment_download_basic(&s);
}

#[test]
// SDO 16
fn test_segment_upload() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_segment_upload(&s);
}

#[test]
// SDO 17
fn test_segment_upload_with_toggle_bit_error() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_segment_upload_with_toggle_bit_error(&s);
}

#[test]
// SDO 19, read
fn test_block_upload_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_without_crc(&s);
}

#[test]
// SDO 21, read
fn test_block_upload_string_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_string_without_crc(&s);
}

#[test]
// SDO 23, read
fn test_block_upload_string_with_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_string_with_crc(&s);
}

#[test]
// SDO 26, read
fn test_block_upload_with_wrong_blocksize() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_with_wrong_blocksize(&s);
}

#[test]
// SDO 27, read
fn test_block_upload_with_wrong_ack_seqno() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_with_wrong_ack_seqno(&s);
}

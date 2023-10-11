use socketcan::Socket;

use co_test::{sdo_block_upload_string_with_crc, sdo_block_upload_string_without_crc, sdo_block_upload_with_wrong_ack_seqno, sdo_block_upload_with_wrong_blocksize, sdo_block_upload_without_crc, sdo_error_mismatch_length, sdo_error_read, sdo_error_write, sdo_expedite_read, sdo_segment_download_basic, sdo_segment_upload, sdo_segment_upload_with_toggle_bit_error, sdo_with_node_id_in_expr, sdo_write_and_read};

pub const INTERFACE_NAME: &str = "can0";
pub const DEMO_EDS_PATH: &str = "tests/fixtures/demoDevice.eds";
fn main() {
    let s = socketcan::CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    sdo_write_and_read(&s);
    sdo_error_write(&s);
    sdo_error_read(&s);
    sdo_expedite_read(&s);
    sdo_error_mismatch_length(&s);
    sdo_with_node_id_in_expr(&s);
    sdo_segment_download_basic(&s);
    sdo_segment_upload(&s);
    sdo_segment_upload_with_toggle_bit_error(&s);
    sdo_block_upload_without_crc(&s);
    sdo_block_upload_string_without_crc(&s);
    sdo_block_upload_string_with_crc(&s);
    sdo_block_upload_with_wrong_blocksize(&s);
    sdo_block_upload_with_wrong_ack_seqno(&s);
}

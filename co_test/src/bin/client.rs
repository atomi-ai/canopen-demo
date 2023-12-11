use socketcan::{CanSocket, Socket};
use canopen_rust::error::ErrorCode;

use co_test::{sdo_block_upload_string_with_crc, sdo_block_upload_string_without_crc, sdo_block_upload_with_wrong_ack_seqno, sdo_block_upload_with_wrong_blocksize, sdo_block_upload_without_crc, sdo_error_mismatch_length, sdo_error_read, sdo_error_write, sdo_expedite_read, sdo_segment_download_basic, sdo_segment_upload, sdo_segment_upload_with_toggle_bit_error, sdo_with_node_id_in_expr, sdo_write_and_read};
use co_test::emergency_func::emergency_basic;
use co_test::heartbeat_func::heartbeat_basic;
use co_test::pdo_func::{rpdo_event_driven_mode, rpdo_sync_mode, tpdo_event_driven_mode, tpdo_sync_mode};
use co_test::pdo_object_func::{pdo_mapping_object_accessibility, pdo_test_mapping_params};
use co_test::restore_func::{error_restore_params, restore_all_params, restore_application_params, restore_communication_params};
use co_test::util::{CAN0_INTERFACE, default_logger_init, send};

pub const DEMO_EDS_PATH: &str = "tests/fixtures/demoDevice.eds";

fn run_sdo_tests(s: &CanSocket) {
    sdo_write_and_read(s);
    sdo_error_write(s);
    sdo_error_read(s);
    sdo_expedite_read(s);
    sdo_error_mismatch_length(s);
    sdo_with_node_id_in_expr(s);
    sdo_segment_download_basic(s);
    sdo_segment_upload(s);
    sdo_segment_upload_with_toggle_bit_error(s);
    sdo_block_upload_without_crc(s);
    sdo_block_upload_string_without_crc(s);
    sdo_block_upload_string_with_crc(s);
    sdo_block_upload_with_wrong_blocksize(s);
    sdo_block_upload_with_wrong_ack_seqno(s);
}

fn run_pdo_tests(s: &CanSocket) -> Result<(), ErrorCode> {
    tpdo_event_driven_mode(s, CAN0_INTERFACE)?;
    tpdo_sync_mode(s, CAN0_INTERFACE)?;
    rpdo_event_driven_mode(s)?;
    rpdo_sync_mode(s)?;

    Ok(())
}

fn run_emergency_tests(s: &CanSocket) {
    emergency_basic(s);
}

fn run_heartbeat_tests(s: &CanSocket) {
    heartbeat_basic(s);
}

fn run_restore_tests(s: &CanSocket) {
    restore_application_params(s);
    restore_communication_params(s);
    restore_all_params(s);

    error_restore_params(&s, 1);
    error_restore_params(&s, 2);
    error_restore_params(&s, 3);
}

fn run_pdo_object_tests(s: &CanSocket) {
    send(&s, 0x000, 0x81_02, 2);

    pdo_test_mapping_params(&s, 0x1600);
    pdo_test_mapping_params(&s, 0x1601);
    pdo_test_mapping_params(&s, 0x1602);
    pdo_test_mapping_params(&s, 0x1603);
    pdo_test_mapping_params(&s, 0x1A00);
    pdo_test_mapping_params(&s, 0x1A01);
    pdo_test_mapping_params(&s, 0x1A02);
    pdo_test_mapping_params(&s, 0x1A03);

    pdo_mapping_object_accessibility(&s, 0x1600, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1601, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1602, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1603, 0x1);
}

fn main() -> Result<(), ErrorCode> {
    default_logger_init();

    let s = CanSocket::open(CAN0_INTERFACE).expect("Failed to open CAN socket");
    run_restore_tests(&s);
    run_sdo_tests(&s);
    run_pdo_tests(&s)?;
    run_emergency_tests(&s);
    run_heartbeat_tests(&s);
    run_pdo_object_tests(&s);

    Ok(())
}

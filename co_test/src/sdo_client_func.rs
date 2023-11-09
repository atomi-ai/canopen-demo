use socketcan::CanSocket;
use canopen::util::genf;
use crate::util::{exp, send};

pub fn sdo_write_and_read(s: &CanSocket) {
    // read / write / read for 1017h:00h
    send(&s, &genf(0x602, &[0x2B, 0x17, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x60, 0x17, 0x10, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0x40, 0x17, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x4B, 0x17, 0x10, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0x2B, 0x17, 0x10, 0, 0x12, 0x34, 0, 0]));
    exp(&s, &genf(0x582, &[0x60, 0x17, 0x10, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0x40, 0x17, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x4B, 0x17, 0x10, 0, 0x12, 0x34, 0, 0]));
}

pub fn sdo_error_write(s: &CanSocket) {
    // Write object 1000h:00h (ro) => ERR 06010002h
    send(&s, &genf(0x602, &[0x23, 0, 0x10, 0, 0x91, 0x01, 0x0F, 0]));
    exp(&s, &genf(0x582, &[0x80, 0, 0x10, 0, 0x02, 0, 0x01, 0x06]));
}

pub fn sdo_error_read(s: &CanSocket) {
    // Read object 1000h:01h => ERR 06090011h
    send(&s, &genf(0x602, &[0x40, 0, 0x10, 0x1, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0, 0x10, 0x1, 0x11, 0, 0x09, 0x06]));
    // Read object 1004h:00h => ERR 06020000h
    send(&s, &genf(0x602, &[0x40, 0x04, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x04, 0x10, 0, 0, 0, 0x02, 0x06]));
    // Read object 1000h:00h => ERR 05040001h
    send(&s, &genf(0x602, &[0xE0, 0, 0x10, 0x0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0, 0x10, 0x0, 0x01, 0, 0x04, 0x05]));
}

pub fn sdo_expedite_read(s: &CanSocket) {
    // Read object 1000h:00h => 0xF0191
    send(&s, &genf(0x602, &[0x40, 0, 0x10, 0x0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x43, 0, 0x10, 0x0, 0x91, 0x01, 0x0F, 0]));
}

pub fn sdo_error_mismatch_length(s: &CanSocket) {
    // Read object 1003h:00h => 0x0 (u8)
    send(&s, &genf(0x602, &[0x40, 0x03, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x4F, 0x03, 0x10, 0, 0, 0, 0, 0]));

    // Write object 1003h:00h with 0x00000000 (u32) => ERR 06070012
    send(&s, &genf(0x602, &[0x23, 0x03, 0x10, 0x0, 0x12, 0x34, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x03, 0x10, 0, 0x12, 0, 0x07, 0x6]));

    // Read object 1005h:00h => 0x0 (u8)
    send(&s, &genf(0x602, &[0x40, 0x05, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x43, 0x05, 0x10, 0, 0x80, 0, 0, 0]));

    // Write object 1005h:00h with 0x00000000 (u32) => ERR 06070012
    send(&s, &genf(0x602, &[0x2F, 0x05, 0x10, 0, 0x12, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x05, 0x10, 0, 0x13, 0, 0x07, 0x6]));
}

pub fn sdo_with_node_id_in_expr(s: &CanSocket) {
    // Read object 1200h:01h => 0x0 (u8)
    send(&s, &genf(0x602, &[0x40, 0x00, 0x12, 0x1, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x43, 0x00, 0x12, 0x1, 0x02, 0x06, 0, 0]));

    // Read object 1200h:02h => 0x0 (u8)
    send(&s, &genf(0x602, &[0x40, 0x00, 0x12, 0x2, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x43, 0x00, 0x12, 0x2, 0x82, 0x05, 0, 0]));
}

pub fn sdo_segment_download_basic(s: &CanSocket) {
    // Write object 1017h:00h with 0x0002 (u16)
    send(&s, &genf(0x602, &[0x21, 0x17, 0x10, 0x0, 0x02, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x60, 0x17, 0x10, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0x0B, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x20, 0, 0, 0, 0, 0, 0, 0]));
}

pub fn sdo_segment_upload(s: &CanSocket) {
    // Read object 1008h:00h => 0x10 (u8)
    send(&s, &genf(0x602, &[0x40, 0x08, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x41, 0x08, 0x10, 0x0, 0x10, 0, 0, 0]));

    let t = [0x0, 0x43, 0x41, 0x4E, 0x6F, 0x70, 0x65, 0x6E];
    send(&s, &genf(0x602, &[0x60, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &t));

    let t = [0x10, 0x44, 0x65, 0x6D, 0x6F, 0x50, 0x49, 0x43];
    send(&s, &genf(0x602, &[0x70, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &t));

    send(&s, &genf(0x602, &[0x60, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x0B, 0x33, 0x32, 0, 0, 0, 0, 0]));
}

pub fn sdo_segment_upload_with_toggle_bit_error(s: &CanSocket) {
    // Read object 1008h:00h => 0x10 (u8)
    send(&s, &genf(0x602, &[0x40, 0x08, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x41, 0x08, 0x10, 0x0, 0x10, 0, 0, 0]));

    send(&s, &genf(0x602, &[0x70, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x08, 0x10, 0, 0, 0, 0x03, 0x05]));
}

pub fn sdo_block_upload_without_crc(s: &CanSocket) {
    send(&s, &genf(0x602, &[0xA0, 0x00, 0x10, 0x00, 0x14, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xC6, 0x00, 0x10, 0x00, 0x04, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA3, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x81, 0x91, 0x01, 0x0F, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA2, 0x01, 0x14, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xCD, 0, 0, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

pub fn sdo_block_upload_string_without_crc(s: &CanSocket) {
    send(&s, &genf(0x602, &[0xA0, 0x08, 0x10, 0x00, 0x14, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xC6, 0x08, 0x10, 0x00, 0x10, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA3, 0, 0, 0, 0, 0, 0, 0]));
    let t = [0x01, 0x43, 0x41, 0x4E, 0x6F, 0x70, 0x65, 0x6E];
    exp(&s, &genf(0x582, &t));
    let t = [0x02, 0x44, 0x65, 0x6D, 0x6F, 0x50, 0x49, 0x43];
    exp(&s, &genf(0x582, &t));
    exp(&s, &genf(0x582, &[0x83, 0x33, 0x32, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA2, 0x03, 0x14, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xD5, 0, 0, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

pub fn sdo_block_upload_string_with_crc(s: &CanSocket) {
    send(&s, &genf(0x602, &[0xA4, 0x08, 0x10, 0x00, 0x14, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xC6, 0x08, 0x10, 0x00, 0x10, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA3, 0, 0, 0, 0, 0, 0, 0]));
    let t = [0x01, 0x43, 0x41, 0x4E, 0x6F, 0x70, 0x65, 0x6E];
    exp(&s, &genf(0x582, &t));
    let t = [0x02, 0x44, 0x65, 0x6D, 0x6F, 0x50, 0x49, 0x43];
    exp(&s, &genf(0x582, &t));
    exp(&s, &genf(0x582, &[0x83, 0x33, 0x32, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA2, 0x03, 0x14, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xD5, 0xF3, 0x43, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

pub fn sdo_block_upload_with_wrong_blocksize(s: &CanSocket) {
    send(&s, &genf(0x602, &[0xA0, 0x00, 0x10, 0x00, 0x80, 0, 0, 0]));
    let t = [0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x04, 0x05];
    exp(&s, &genf(0x582, &t));
}

pub fn sdo_block_upload_with_wrong_ack_seqno(s: &CanSocket) {
    send(&s, &genf(0x602, &[0xA0, 0x00, 0x10, 0x00, 0x14, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xC6, 0x00, 0x10, 0x00, 0x04, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA3, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x81, 0x91, 0x01, 0x0F, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA2, 0x80, 0x14, 0, 0, 0, 0, 0]));
    let t = [0x80, 0x00, 0x10, 0x00, 0x01, 0x00, 0x04, 0x05];
    exp(&s, &genf(0x582, &t));
}
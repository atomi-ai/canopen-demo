use socketcan::CanSocket;
use crate::util::{expf, sendf};

pub fn sdo_write_and_read(s: &CanSocket) {
    // read / write / read for 1017h:00h
    sendf(&s, 0x602, 0x2B_17_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);
    sendf(&s, 0x602, 0x40_17_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x4B_17_10_00_00_00_00_00, 8);
    sendf(&s, 0x602, 0x2B_17_10_00_12_34_00_00, 8);
    expf(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);
    sendf(&s, 0x602, 0x40_17_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x4B_17_10_00_12_34_00_00, 8);
}

pub fn sdo_error_write(s: &CanSocket) {
    // Write object 1000h:00h (ro) => ERR 06010002h
    sendf(&s, 0x602, 0x23_00_10_00_91_01_0F_00, 8);
    expf(&s, 0x582, 0x80_00_10_00_02_00_01_06, 8);
}

pub fn sdo_error_read(s: &CanSocket) {
    // Read object 1000h:01h => ERR 06090011h
    sendf(&s, 0x602, 0x40_00_10_01_00_00_00_00, 8);
    expf(&s, 0x582, 0x80_00_10_01_11_00_09_06, 8);
    // Read object 1004h:00h => ERR 06020000h
    sendf(&s, 0x602, 0x40_04_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x80_04_10_00_00_00_02_06, 8);
    // Read object 1000h:00h => ERR 05040001h
    sendf(&s, 0x602, 0xE0_00_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x80_00_10_00_01_00_04_05, 8);
}

pub fn sdo_expedite_read(s: &CanSocket) {
    // Read object 1000h:00h => 0xF0191
    sendf(&s, 0x602, 0x40_00_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x43_00_10_00_91_01_0F_00, 8);
}

pub fn sdo_error_mismatch_length(s: &CanSocket) {
    // Read object 1003h:00h => 0x0 (u8)
    sendf(&s, 0x602, 0x40_03_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x4F_03_10_00_00_00_00_00, 8);

    // Write object 1003h:00h with 0x00000000 (u32) => ERR 06070012
    sendf(&s, 0x602, 0x23_03_10_00_12_34_00_00, 8);
    expf(&s, 0x582, 0x80_03_10_00_12_00_07_06, 8);

    // Read object 1005h:00h => 0x0 (u8)
    sendf(&s, 0x602, 0x40_05_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x43_05_10_00_80_00_00_00, 8);

    // Write object 1005h:00h with 0x00000000 (u32) => ERR 06070012
    sendf(&s, 0x602, 0x2F_05_10_00_12_00_00_00, 8);
    expf(&s, 0x582, 0x80_05_10_00_13_00_07_06, 8);
}

pub fn sdo_with_node_id_in_expr(s: &CanSocket) {
    // Read object 1200h:01h => 0x0 (u8)
    sendf(&s, 0x602, 0x40_00_12_01_00_00_00_00, 8);
    expf(&s, 0x582, 0x43_00_12_01_02_06_00_00, 8);

    // Read object 1200h:02h => 0x0 (u8)
    sendf(&s, 0x602, 0x40_00_12_02_00_00_00_00, 8);
    expf(&s, 0x582, 0x43_00_12_02_82_05_00_00, 8);
}

pub fn sdo_segment_download_basic(s: &CanSocket) {
    // Write object 1017h:00h with 0x0002 (u16)
    sendf(&s, 0x602, 0x21_17_10_00_02_00_00_00, 8);
    expf(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);

    sendf(&s, 0x602, 0x0B_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x20_00_00_00_00_00_00_00, 8);
}

pub fn sdo_segment_upload(s: &CanSocket) {
    // Read object 1008h:00h => 0x10 (u8)
    sendf(&s, 0x602, 0x40_08_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x41_08_10_00_10_00_00_00, 8);

    sendf(&s, 0x602, 0x60_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x00_43_41_4E_6F_70_65_6E, 8);

    sendf(&s, 0x602, 0x70_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x10_44_65_6D_6F_50_49_43, 8);

    sendf(&s, 0x602, 0x60_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x0B_33_32_00_00_00_00_00, 8);
}

pub fn sdo_segment_upload_with_toggle_bit_error(s: &CanSocket) {
    // Read object 1008h:00h => 0x10 (u8)
    sendf(&s, 0x602, 0x40_08_10_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x41_08_10_00_10_00_00_00, 8);

    sendf(&s, 0x602, 0x70_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x80_08_10_00_00_00_03_05, 8);
}

pub fn sdo_block_upload_without_crc(s: &CanSocket) {
    sendf(&s, 0x602, 0xA0_00_10_00_14_00_00_00, 8);
    expf(&s, 0x582, 0xC6_00_10_00_04_00_00_00, 8);

    sendf(&s, 0x602, 0xA3_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x81_91_01_0F_00_00_00_00, 8);

    sendf(&s, 0x602, 0xA2_01_14_00_00_00_00_00, 8);
    expf(&s, 0x582, 0xCD_00_00_00_00_00_00_00, 8);

    sendf(&s, 0x602, 0xA1_00_00_00_00_00_00_00, 8);
}

pub fn sdo_block_upload_string_without_crc(s: &CanSocket) {
    sendf(&s, 0x602, 0xA0_08_10_00_14_00_00_00, 8);
    expf(&s, 0x582, 0xC6_08_10_00_10_00_00_00, 8);

    sendf(&s, 0x602, 0xA3_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x01_43_41_4E_6F_70_65_6E, 8);
    expf(&s, 0x582, 0x02_44_65_6D_6F_50_49_43, 8);
    expf(&s, 0x582, 0x83_33_32_00_00_00_00_00, 8);

    sendf(&s, 0x602, 0xA2_03_14_00_00_00_00_00, 8);
    expf(&s, 0x582, 0xD5_00_00_00_00_00_00_00, 8);
    sendf(&s, 0x602, 0xA1_00_00_00_00_00_00_00, 8);
}

pub fn sdo_block_upload_string_with_crc(s: &CanSocket) {
    sendf(&s, 0x602, 0xA4_08_10_00_14_00_00_00, 8);
    expf(&s, 0x582, 0xC6_08_10_00_10_00_00_00, 8);

    sendf(&s, 0x602, 0xA3_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x01_43_41_4E_6F_70_65_6E, 8);
    expf(&s, 0x582, 0x02_44_65_6D_6F_50_49_43, 8);
    expf(&s, 0x582, 0x83_33_32_00_00_00_00_00, 8);

    sendf(&s, 0x602, 0xA2_03_14_00_00_00_00_00, 8);
    expf(&s, 0x582, 0xD5_F3_43_00_00_00_00_00, 8);
    sendf(&s, 0x602, 0xA1_00_00_00_00_00_00_00, 8);
}

pub fn sdo_block_upload_with_wrong_blocksize(s: &CanSocket) {
    sendf(&s, 0x602, 0xA0_00_10_00_80_00_00_00, 8);
    expf(&s, 0x582, 0x80_00_10_00_02_00_04_05, 8);

}

pub fn sdo_block_upload_with_wrong_ack_seqno(s: &CanSocket) {
    sendf(&s, 0x602, 0xA0_00_10_00_14_00_00_00, 8);
    expf(&s, 0x582, 0xC6_00_10_00_04_00_00_00, 8);

    sendf(&s, 0x602, 0xA3_00_00_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x81_91_01_0F_00_00_00_00, 8);

    sendf(&s, 0x602, 0xA2_80_14_00_00_00_00_00, 8);
    expf(&s, 0x582, 0x80_00_10_00_01_00_04_05, 8);
}
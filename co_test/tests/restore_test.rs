use socketcan::{CanSocket, Socket};
use co_test::util::{exp, INTERFACE_NAME, send};
use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_restore_all_params() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    // [1017:00] = 0x64 (0), [6000:00] = 0x2 (8).
    send(&s, 0x602, 0x2B_17_10_00_64_00_00_00, 8);
    exp(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);
    send(&s, 0x602, 0x40_17_10_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x4B_17_10_00_64_00_00_00, 8);
    send(&s, 0x602, 0x2F_00_62_01_02_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_62_01_00_00_00_00, 8);
    send(&s, 0x602, 0x40_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_62_01_02_00_00_00, 8);

    // reset all
    send(&s, 0x602, 0x23_11_10_01_6C_6F_61_64, 8);
    exp(&s, 0x582, 0x60_11_10_01_00_00_00_00, 8);
    send(&s, 0x602, 0x40_11_10_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x43_11_10_01_01_00_00_00, 8);

    // check the fields are restored.
    send(&s, 0x602, 0x40_17_10_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x4B_17_10_00_00_00_00_00, 8);
    send(&s, 0x602, 0x40_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_62_01_00_00_00_00, 8);
}

#[test]
fn test_restore_communication_params() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    // [1017:00] = 0x64 (0), [6000:00] = 0x2 (8).
    send(&s, 0x602, 0x2B_17_10_00_64_00_00_00, 8);
    exp(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);
    send(&s, 0x602, 0x40_17_10_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x4B_17_10_00_64_00_00_00, 8);
    send(&s, 0x602, 0x2F_00_62_01_02_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_62_01_00_00_00_00, 8);
    send(&s, 0x602, 0x40_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_62_01_02_00_00_00, 8);

    // reset communication
    send(&s, 0x602, 0x23_11_10_02_6C_6F_61_64, 8);
    exp(&s, 0x582, 0x60_11_10_02_00_00_00_00, 8);
    send(&s, 0x602, 0x40_11_10_02_00_00_00_00, 8);
    exp(&s, 0x582, 0x43_11_10_02_01_00_00_00, 8);

    // [1017:00] is restored to 0, but [6000:00] is still 0x2
    send(&s, 0x602, 0x40_17_10_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x4B_17_10_00_00_00_00_00, 8);
    send(&s, 0x602, 0x40_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_62_01_02_00_00_00, 8);
}

#[test]
fn test_restore_application_params() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");
    // [1017:00] = 0x64 (0), [6000:00] = 0x2 (8).
    send(&s, 0x602, 0x2B_17_10_00_64_00_00_00, 8);
    exp(&s, 0x582, 0x60_17_10_00_00_00_00_00, 8);
    send(&s, 0x602, 0x40_17_10_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x4B_17_10_00_64_00_00_00, 8);
    send(&s, 0x602, 0x2F_00_62_01_02_00_00_00, 8);
    exp(&s, 0x582, 0x60_00_62_01_00_00_00_00, 8);
    send(&s, 0x602, 0x40_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_62_01_02_00_00_00, 8);

    // reset application
    send(&s, 0x602, 0x23_11_10_03_6C_6F_61_64, 8);
    exp(&s, 0x582, 0x60_11_10_03_00_00_00_00, 8);
    send(&s, 0x602, 0x40_11_10_03_00_00_00_00, 8);
    exp(&s, 0x582, 0x43_11_10_03_01_00_00_00, 8);

    // [1017:00] is still 0x64, while [6000:00] is restored
    send(&s, 0x602, 0x40_17_10_00_00_00_00_00, 8);
    exp(&s, 0x582, 0x4B_17_10_00_64_00_00_00, 8);
    send(&s, 0x602, 0x40_00_62_01_00_00_00_00, 8);
    exp(&s, 0x582, 0x4F_00_62_01_00_00_00_00, 8);
}

fn error_restore_params(s: &CanSocket, sub_index: u8) {
    // Error to set the restoring param.
    // "If a wrong signature is written, the device refuses to restore the
    // defaults and responds with an Abort SDO Transfer (abort code:
    // 0800 002xh)."
    let si_bits = (sub_index as u64) << 32;
    send(&s, 0x602, 0x23_11_10_00_6E_6F_72_77 | si_bits, 8);
    exp(&s, 0x582, 0x80_11_10_00_20_00_00_08 | si_bits, 8);
}

#[test]
fn test_error_restore_params() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    error_restore_params(&s, 1);
    error_restore_params(&s, 2);
    error_restore_params(&s, 3);
}

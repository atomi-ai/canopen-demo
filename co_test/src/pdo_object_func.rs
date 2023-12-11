use socketcan::CanSocket;
use crate::util::{exp, send};

pub fn pdo_test_mapping_params(s: &CanSocket, index: u16) {
    let [low, high] = index.to_le_bytes();
    let idx_bits = ((low as u64) << 48) | ((high as u64) << 40);
    send(&s, 0x602, 0x2F_00_00_00_00_00_00_00 | idx_bits, 8);
    exp(&s, 0x582, 0x60_00_00_00_00_00_00_00 | idx_bits, 8);

    for si in 1..=8 {
        let si_bits = (si as u64) << 32;
        send(&s, 0x602, 0x23_00_00_00_08_01_00_62 | idx_bits | si_bits, 8);
        exp(&s, 0x582, 0x60_00_00_00_00_00_00_00 | idx_bits | si_bits, 8);
    }

    // Expect CanAbortCode: 0x06040041(ObjectCannotBeMappedToPDO)
    send(&s, 0x602, 0x2F_00_10_00_09_00_00_00 | idx_bits, 8);
    exp(&s, 0x582, 0x80_00_10_00_41_00_04_06 | idx_bits, 8);

    // Succeed
    send(&s, 0x602, 0x2F_00_00_00_08_00_00_00 | idx_bits, 8);
    exp(&s, 0x582, 0x60_00_00_00_00_00_00_00 | idx_bits, 8);

    // Object too large, 0x06040042(ExceedPDOSize)
    send(&s, 0x602, 0x23_00_00_08_10_01_00_62 | idx_bits, 8);
    exp(&s, 0x582, 0x60_00_00_08_00_00_00_00 | idx_bits, 8);
    send(&s, 0x602, 0x2F_00_00_00_08_00_00_00 | idx_bits, 8);
    exp(&s, 0x582, 0x80_00_00_00_42_00_04_06 | idx_bits, 8);
}

pub fn pdo_mapping_object_accessibility(s: &CanSocket, index: u16, sub_index: u8) {
    let [low, high] = index.to_le_bytes();
    let idx_bits = ((low as u64) << 48) | ((high as u64) << 40);
    let si_bits = (sub_index as u64) << 32;
    send(&s, 0x602, 0x23_00_00_00_08_00_01_10 | idx_bits | si_bits, 8);
    exp(&s, 0x582, 0x80_00_00_00_41_00_04_06 | idx_bits | si_bits, 8);
    send(&s, 0x602, 0x23_00_00_00_10_05_01_14 | idx_bits | si_bits, 8);
    exp(&s, 0x582, 0x80_00_00_00_41_00_04_06 | idx_bits | si_bits, 8);
}

extern crate alloc;

use socketcan::CanSocket;
use socketcan::Socket;

use canopen::node::Node;
use co_test::util as tu;
use co_test::util::{expf, INTERFACE_NAME, sendf};

use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_rpdo_comm_params() {
    let _context = CONTEXT.lock().unwrap();
    let content = std::fs::read_to_string(tu::DEMO_EDS_PATH).expect("Failed to read EDS file");
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    let node = Node::new(0x2, &content, s).expect("Errors in creating a node");
    // info!("xfguo: pdo_objs = {:#x?}", node.pdo_objects);
    let rpdo = node.pdo_objects().get_rpdo(0);
    assert_eq!(rpdo.largest_sub_index(), 5);
    assert_eq!(rpdo.cob_id(), 0x202);
    assert_eq!(rpdo.transmission_type(), 255);
    assert_eq!(rpdo.event_timer(), 0);
}

fn pdo_test_mapping_params(s: &CanSocket, index: u16) {
    let [low, high] = index.to_le_bytes();
    let idx_bits = ((low as u64) << 48) | ((high as u64) << 40);
    sendf(&s, 0x602, 0x2F_00_00_00_00_00_00_00 | idx_bits, 8);
    expf(&s, 0x582, 0x60_00_00_00_00_00_00_00 | idx_bits, 8);

    for si in 1..=8 {
        let si_bits = (si as u64) << 32;
        sendf(&s, 0x602, 0x23_00_00_00_08_01_00_62 | idx_bits | si_bits, 8);
        expf(&s, 0x582, 0x60_00_00_00_00_00_00_00 | idx_bits | si_bits, 8);
    }

    // Expect CanAbortCode: 0x06040041(ObjectCannotBeMappedToPDO)
    sendf(&s, 0x602, 0x2F_00_10_00_09_00_00_00 | idx_bits, 8);
    expf(&s, 0x582, 0x80_00_10_00_41_00_04_06 | idx_bits, 8);

    // Succeed
    sendf(&s, 0x602, 0x2F_00_00_00_08_00_00_00 | idx_bits, 8);
    expf(&s, 0x582, 0x60_00_00_00_00_00_00_00 | idx_bits, 8);

    // Object too large, 0x06040042(ExceedPDOSize)
    sendf(&s, 0x602, 0x23_00_00_08_10_01_00_62 | idx_bits, 8);
    expf(&s, 0x582, 0x60_00_00_08_00_00_00_00 | idx_bits, 8);
    sendf(&s, 0x602, 0x2F_00_00_00_08_00_00_00 | idx_bits, 8);
    expf(&s, 0x582, 0x80_00_00_00_42_00_04_06 | idx_bits, 8);
}

#[test]
fn test_mapping_params_validation() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    pdo_test_mapping_params(&s, 0x1600);
    pdo_test_mapping_params(&s, 0x1601);
    pdo_test_mapping_params(&s, 0x1602);
    pdo_test_mapping_params(&s, 0x1603);
    pdo_test_mapping_params(&s, 0x1A00);
    pdo_test_mapping_params(&s, 0x1A01);
    pdo_test_mapping_params(&s, 0x1A02);
    pdo_test_mapping_params(&s, 0x1A03);
}

fn pdo_mapping_object_accessibility(s: &CanSocket, index: u16, sub_index: u8) {
    let [low, high] = index.to_le_bytes();
    let idx_bits = ((low as u64) << 48) | ((high as u64) << 40);
    let si_bits = (sub_index as u64) << 32;
    sendf(&s, 0x602, 0x23_00_00_00_08_00_01_10 | idx_bits | si_bits, 8);
    expf(&s, 0x582, 0x80_00_00_00_41_00_04_06 | idx_bits | si_bits, 8);
    sendf(&s, 0x602, 0x23_00_00_00_10_05_01_14 | idx_bits | si_bits, 8);
    expf(&s, 0x582, 0x80_00_00_00_41_00_04_06 | idx_bits | si_bits, 8);
}

#[test]
fn test_pdo_mapping_object_accessibility() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");

    pdo_mapping_object_accessibility(&s, 0x1600, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1601, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1602, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1603, 0x1);
}
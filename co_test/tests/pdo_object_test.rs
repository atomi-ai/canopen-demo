extern crate alloc;

use socketcan::CanSocket;
use socketcan::Socket;

use canopen_rust::node::Node;
use co_test::pdo_object_func::{pdo_mapping_object_accessibility, pdo_test_mapping_params};
use co_test::util as tu;
use co_test::util::VCAN0_INTERFACE;

use crate::testing::CONTEXT;

mod testing;

#[test]
fn test_rpdo_comm_params() {
    let _context = CONTEXT.lock().unwrap();
    let content = std::fs::read_to_string(tu::DEMO_EDS_PATH).expect("Failed to read EDS file");
    let s = socketcan::CanSocket::open(tu::VCAN0_INTERFACE).expect("Failed to open CAN socket");
    let mut node = Node::new(0x2, &content, s).expect("Errors in creating a node");
    // info!("xfguo: pdo_objs = {:#x?}", node.pdo_objects);
    let rpdo = node.pdo_objects().get_mut_rpdo_with_cob_id(0x202).expect("");
    assert_eq!(rpdo.largest_sub_index(), 5);
    assert_eq!(rpdo.cob_id(), 0x202);
    assert_eq!(rpdo.transmission_type(), 255);
    assert_eq!(rpdo.event_timer(), 0);
}

#[test]
fn test_mapping_params_validation() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");

    pdo_test_mapping_params(&s, 0x1600);
    pdo_test_mapping_params(&s, 0x1601);
    pdo_test_mapping_params(&s, 0x1602);
    pdo_test_mapping_params(&s, 0x1603);
    pdo_test_mapping_params(&s, 0x1A00);
    pdo_test_mapping_params(&s, 0x1A01);
    pdo_test_mapping_params(&s, 0x1A02);
    pdo_test_mapping_params(&s, 0x1A03);
}

#[test]
fn test_pdo_mapping_object_accessibility() {
    let _context = CONTEXT.lock().unwrap();
    let s = CanSocket::open(VCAN0_INTERFACE).expect("Failed to open CAN socket");

    pdo_mapping_object_accessibility(&s, 0x1600, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1601, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1602, 0x1);
    pdo_mapping_object_accessibility(&s, 0x1603, 0x1);
}

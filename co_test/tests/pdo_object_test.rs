mod testing;
extern crate alloc;

use alloc::sync::Arc;
use std::io::Write;
use std::sync::Mutex;
use lazy_static::lazy_static;
use log::info;

use co_test::util as tu;
use canopen::node::Node;
use embedded_can::{nb::Can};
use socketcan::CanSocket;
use socketcan::{EmbeddedFrame, Frame, Socket};

lazy_static! {
    static ref TESTING_NODE: Arc<Mutex<Node<CanSocket>>> = {
        testing::default_logger_init();
        info!("xfguo: static init");
        let content = std::fs::read_to_string(tu::DEMO_EDS_PATH).expect("Failed to read EDS file");
        let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
        let node = Node::new(0x2, &content, s);
        Arc::new(Mutex::new(node))
    };
}

#[test]
fn test_rpdo_comm_params() {
    let node = TESTING_NODE.lock().unwrap();
    info!("xfguo: pdo_objs = {:#x?}", node.pdo_objects);
    let rpdo = &node.pdo_objects.rpdos[0];
    assert_eq!(rpdo.largest_sub_index, 5);
    assert_eq!(rpdo.cob_id, 0x202);
    assert_eq!(rpdo.transmission_type, 255);
    assert_eq!(rpdo.event_timer, 0);
}

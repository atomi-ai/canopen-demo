mod testing;
extern crate alloc;

use alloc::sync::Arc;
use std::io::Write;
use std::sync::Mutex;
use lazy_static::lazy_static;
use log::info;

use canopen::object_directory::ObjectDirectory;
use canopen::pdo::PdoObjects;
use co_test::util as tu;

lazy_static! {
    static ref PDO_OBJECTS: Arc<Mutex<PdoObjects>> = {
        testing::default_logger_init();
        info!("xfguo: static init");
        let content = std::fs::read_to_string(tu::DEMO_EDS_PATH).expect("Failed to read EDS file");
        let mut object_directory = ObjectDirectory::new(2, &content);
        let pdo_objects = PdoObjects::new(&mut object_directory);
        Arc::new(Mutex::new(pdo_objects))
    };
}

#[test]
fn test_rpdo_comm_params() {
    let pdo_objs = PDO_OBJECTS.lock().unwrap();
    let rpdo = &pdo_objs.rpdos[0];
    assert_eq!(rpdo.largest_sub_index, 5);
    assert_eq!(rpdo.cob_id, 0x202);
    assert_eq!(rpdo.transmission_type, 255);
    assert_eq!(rpdo.event_timer, 0);
}

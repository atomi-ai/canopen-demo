#[macro_use]
extern crate lazy_static;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use async_std::future::timeout;
use async_std::task;
use socketcan::{EmbeddedFrame, Socket};
use socketcan::Frame;

use canopen::node;
use co_test::{sdo_block_upload_string_with_crc, sdo_block_upload_string_without_crc, sdo_block_upload_with_wrong_ack_seqno, sdo_block_upload_with_wrong_blocksize, sdo_block_upload_without_crc, sdo_error_mismatch_length, sdo_error_read, sdo_error_write, sdo_expedite_read, sdo_segment_download_basic, sdo_segment_upload, sdo_segment_upload_with_toggle_bit_error, sdo_with_node_id_in_expr, sdo_write_and_read, util as tu};

mod testing;

struct TestContext {
    _node_thread: thread::JoinHandle<()>,
}

impl TestContext {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Wait for the server up...");
        let content = std::fs::read_to_string(tu::DEMO_EDS_PATH).expect("Failed to read EDS file");
        let s = socketcan::async_io::CanSocket::open(tu::INTERFACE_NAME).unwrap();
        let read_task = s.read_frame();

        println!("Start the testing server thread");
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_clone = is_running.clone();
        let node_thread = thread::spawn(move || {
            let mut node = node::Node::new(
                2,
                &content,
                Box::new(
                    socketcan::CanSocket::open(tu::INTERFACE_NAME)
                        .expect("Failed to open CAN socket"),
                ),
            );
            node.init();
            is_running_clone.store(true, Ordering::Relaxed);
            node.run();
        });
        while !is_running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }

        let msg = timeout(Duration::from_secs(3), read_task).await??;
        if msg.raw_id() != 0x234 || msg.data() != &[0x01, 0x02, 0x03, 0x05] {
            panic!(
                "Received unexpected CanFrame: {}",
                tu::frame_to_string(&msg)
            );
        }

        Ok(TestContext {
            _node_thread: node_thread,
        })
    }
}

lazy_static! {
    static ref CONTEXT: Arc<Mutex<TestContext>> = {
        let ctx = task::block_on(TestContext::new()).unwrap();
        Arc::new(Mutex::new(ctx))
    };
}

#[test]
fn test_write_and_read() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_write_and_read(&s);
}

#[test]
fn test_error_write() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_error_write(&s);
}

#[test]
fn test_error_read() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_error_read(&s);
}

#[test]
// Expedite upload
fn test_read_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_expedite_read(&s);
}

#[test]
// SDO 08 & 09
fn test_error_mismatch_length() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_error_mismatch_length(&s);
}

#[test]
// SDO 12
fn test_with_node_id_in_expr() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_with_node_id_in_expr(&s);
}

#[test]
// SDO 15
fn test_segment_download_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_segment_download_basic(&s);
}

#[test]
// SDO 16
fn test_segment_upload() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_segment_upload(&s);
}

#[test]
// SDO 17
fn test_segment_upload_with_toggle_bit_error() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_segment_upload_with_toggle_bit_error(&s);
}

#[test]
// SDO 19, read
fn test_block_upload_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_without_crc(&s);
}

#[test]
// SDO 21, read
fn test_block_upload_string_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_string_without_crc(&s);
}

#[test]
// SDO 23, read
fn test_block_upload_string_with_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_string_with_crc(&s);
}

#[test]
// SDO 26, read
fn test_block_upload_with_wrong_blocksize() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_with_wrong_blocksize(&s);
}

#[test]
// SDO 27, read
fn test_block_upload_with_wrong_ack_seqno() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    sdo_block_upload_with_wrong_ack_seqno(&s);
}

use std::sync::{Arc, Mutex};
use std::thread;

use embedded_can::Frame;
use embedded_can::nb::Can;
use socketcan::Socket;

use canopen::util::{genf_and_padding, u64_to_vec};
use co_test::async_util::AsyncExpector;
use co_test::util::INTERFACE_NAME;

fn main() {
    let mut ec = AsyncExpector::new();
    ec.async_expect(0x582, 0x60_00_18_01_00_00_00_00, 8);
    assert_eq!(ec.wait_for_all(), "");

    let mut socket = Arc::new(Mutex::new(socketcan::CanSocket::open(INTERFACE_NAME)
        .expect("Failed to open CAN socket")));

    {
        let socket_clone = socket.clone();
        thread::spawn(move || {
            let mut socket = socket_clone.lock().unwrap();
            match socket.receive() {
                Ok(frame) => { println!("got {:?}", frame); }
                Err(_) => { panic!("Errors"); }
            }
        });
    }

    {
        let socket_clone = socket.clone();
        let mut socket = socket_clone.lock().unwrap();
        let t = socket.receive();
        println!("t = {:?}", t);
    }
}

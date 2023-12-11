use std::sync::{Arc, Mutex};
use std::thread;

use embedded_can::nb::Can;
use socketcan::Socket;
use canopen_rust::error::ErrorCode;

use co_test::async_util::AsyncExpectorOld;
use co_test::util::VCAN0_INTERFACE;

fn main() -> Result<(), ErrorCode> {
    let mut ec = AsyncExpectorOld::new(VCAN0_INTERFACE);
    ec.async_expect(0x582, 0x60_00_18_01_00_00_00_00, 8)?;
    assert_eq!(ec.wait_for_all(), "");

    let socket = Arc::new(Mutex::new(socketcan::CanSocket::open(VCAN0_INTERFACE)
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
    Ok(())
}

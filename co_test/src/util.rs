use std::io::Write;
use std::os::fd::AsRawFd;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use embedded_can::{Frame, StandardId};
use env_logger::Builder;
use log::{debug, LevelFilter};
use nix::poll::{poll, PollFd, PollFlags};
use socketcan::{CanFrame, CanSocket, Socket};

use canopen_rust::util::u64_to_vec;

pub const VCAN0_INTERFACE: &str = "vcan0";
pub const CAN0_INTERFACE: &str = "can0";
pub const SAMPLE_EDS_PATH: &str = "tests/fixtures/sample.eds";
pub const DEMO_EDS_PATH: &str = "tests/fixtures/demoDevice.eds";

pub fn frame_to_string<F: socketcan::Frame>(frame: &F) -> String {
    let id = frame.raw_id();
    let data_string = frame
        .data()
        .iter()
        .fold(String::from(""), |a, b| format!("{} {:02x}", a, b));

    format!("{:X}  [{}] {}", id, frame.dlc(), data_string)
}

fn read_frame_with_timeout(
    socket: &socketcan::CanSocket,
    timeout: std::time::Duration,
) -> Result<CanFrame, &'static str> {
    let mut fds = [PollFd::new(socket.as_raw_fd(), PollFlags::POLLIN)];

    match poll(&mut fds, timeout.as_millis() as i32) {
        Ok(n) => {
            if n == 0 {
                // 超时
                return Err("Timeout");
            }
            match socket.read_frame() {
                Ok(frame) => Ok(frame),
                Err(_) => Err("Error reading frame"),
            }
        }
        Err(_) => Err("Poll error"),
    }
}

pub fn send(socket: &CanSocket, cob_id: u16, data: u64, len: usize) {
    let bytes = u64_to_vec(data, len);
    let frame = CanFrame::new(StandardId::new(cob_id).unwrap(), bytes.as_slice()).unwrap();
    debug!("Send frame: {:?}", frame);
    socket.write_frame(&frame).expect(&format!("Failed on sendf: {:?}", frame));
}

pub fn exp(socket: &CanSocket, cob_id: u16, data: u64, len: usize) {
    let frame = CanFrame::new(StandardId::new(cob_id).unwrap(), u64_to_vec(data, len).as_slice()).unwrap();
    expect_frame(socket, &frame);
}

fn expect_frame(socket: &CanSocket, expected: &CanFrame) {
    let timeout = Duration::from_millis(200);
    let start_time = Instant::now();

    loop {
        if let Ok(response_frame) = read_frame_with_timeout(socket, timeout) {
            if response_frame.id() == expected.id()
                && response_frame.data() == expected.data() {
                debug!("expect frame [{}] {:?}, succeeded", expected.dlc(), expected);
                return;
            } else {
                debug!("unexpected frame [{}] {:?}, ignored", response_frame.dlc(), response_frame);
            }
        }
        if start_time.elapsed() >= Duration::from_secs(1) {
            break;
        }
    }
    assert!(false, "Timeout in getting response of: {:?}", expected);
}

pub fn default_logger_init() {
    Builder::new().format(|buf, record| {
        buf.write_fmt(format_args!(
            "{} [{}, {}] - {}\n",
            Local::now().format("%Y-%m-%dT%H:%M:%S"),
            record.level(),
            thread::current().name().unwrap_or("main"),
            record.args(),
        ))
    }).filter(None, LevelFilter::Debug).init();
}
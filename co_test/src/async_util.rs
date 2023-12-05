// TODO(zephyr): use only one Expector, including sync / async expect.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use embedded_can::Frame;
use embedded_can::nb::Can;
use log::{debug, info};
use socketcan::{CanFrame, CanSocket, Socket};
use canopen::util::{genf, u64_to_vec};

use crate::util::INTERFACE_NAME;

struct FrameContainer {
    frames: Vec<CanFrame>,
}

impl FrameContainer {
    fn new() -> Self {
        FrameContainer { frames: Vec::new() }
    }

    fn push(&mut self, frame: CanFrame) {
        self.frames.push(frame);
    }

    fn find_and_remove(&mut self, expected: &CanFrame) -> Option<CanFrame> {
        if let Some(index) = self.frames.iter().position(|frame| {
            if frame.id() == expected.id() && frame.data() == expected.data() { true } else { false }
        }) {
            Some(self.frames.remove(index))
        } else {
            None
        }
    }
}

fn wait_for_frame(container: Arc<Mutex<FrameContainer>>, expected: CanFrame) -> Result<CanFrame, &'static str> {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(3);

    debug!("xfguo: wait_for_frame 0, {:?}", expected);
    loop {
        {
            let mut container = container.lock().unwrap();
            if let Some(frame) = container.find_and_remove(&expected) {
                info!("xfguo: wait_for_frame 1.1.1; success, {:?}", expected);
                return Ok(frame);
            }
        }

        if start_time.elapsed() >= timeout {
            info!("xfguo: wait_for_frame 1.1.2; TIMEOUT: {:?}", expected);
            return Err("Timeout in getting response");
        }

        thread::sleep(Duration::from_millis(100)); // 等待100ms后再次尝试
    }
    // info!("xfguo: wait_for_frame 9. Should not be here");
}

pub struct AsyncExpector {
    send_socket: Arc<Mutex<CanSocket>>,
    stop_flag: Arc<AtomicBool>,
    container: Arc<Mutex<FrameContainer>>,
    join_handlers: Vec<JoinHandle<Result<CanFrame, &'static str>>>,
    receive_jh: Option<JoinHandle<()>>,
}

impl AsyncExpector {
    pub fn new() -> Self {
        let mut res = AsyncExpector {
            send_socket: Arc::new(Mutex::new(socketcan::CanSocket::open(INTERFACE_NAME)
                .expect("Failed to open CAN socket"))),
            stop_flag: Arc::new(AtomicBool::new(false)),
            container: Arc::new(Mutex::new(FrameContainer::new())),
            join_handlers: vec![],
            receive_jh: None,
        };
        res.start();
        res
    }
    fn start(&mut self) {
        let container_clone = self.container.clone();
        let stop_flag_clone = self.stop_flag.clone();

        let receive_thread = thread::Builder::new().name("receiver".to_string()).spawn(move || {
            let mut recv_socket = socketcan::CanSocket::open(INTERFACE_NAME)
                .expect("Failed to open CAN socket");
            while !stop_flag_clone.load(Ordering::Relaxed) {
                match recv_socket.receive() {
                    Ok(frame) => {
                        let mut container = container_clone.lock().unwrap();
                        debug!("xfguo: Got frame: {:?}", frame);
                        container.push(frame);
                    },
                    Err(_) => { panic!("Errors in receiving packets") },
                }
                thread::sleep(Duration::from_millis(50));
            }
        });

        match receive_thread {
            Ok(rt) => {self.receive_jh = Option::from(rt)}
            Err(_) => {}
        }
    }

    pub fn async_expect(&mut self, cob_id: u16, data: u64, byte_num: usize) {
        let expected = genf(cob_id, &u64_to_vec(data, byte_num));
        let container_clone = self.container.clone();
        self.join_handlers.push(thread::spawn(move || {
            debug!("xfguo: wait for frame: {:?}", expected);
            wait_for_frame(container_clone, expected)
        }));
    }

    pub fn wait_for_all(&mut self) -> String {
        let mut res = String::new();
        let len = self.join_handlers.len();
        for i in 0..self.join_handlers.len() {
            let jh = self.join_handlers.remove(len - 1 - i);
            let t = jh.join();
            debug!("xfguo: wait result: {:?}", t);
            match t {
                Ok(Ok(_)) => {},
                Ok(Err(err)) => {
                    res += err;
                },
                Err(_) => {res += "join error";}
            }
        }
        debug!("xfguo: to return res = '{}'", res);
        res
    }

    pub fn send(&mut self, cob_id: u16, data: u64, byte_num: usize) {
        let f : CanFrame = genf(cob_id, &u64_to_vec(data, byte_num));
        {
            let socket = self.send_socket.lock().unwrap();
            debug!("xfguo: send packet: {:?}", f);
            socket.write_frame(&f).expect("Failed to send request frame");
        }
    }

    pub fn expect(&mut self, cob_id: u16, data: u64, byte_num: usize) {
        let expected = genf(cob_id, &u64_to_vec(data, byte_num));
        let container_clone = self.container.clone();
        match wait_for_frame(container_clone, expected) {
            Ok(_) => {}
            Err(err) => { assert!(false, "Error in getting response: {:?}", err)}
        }
    }

    pub fn unexpect(&mut self, cob_id: u16, data: u64, byte_num: usize) {
        let unexpected = genf(cob_id, &u64_to_vec(data, byte_num));
        let c_clone = self.container.clone();
        let mut c = c_clone.lock().unwrap();
        if let Some(_) = c.find_and_remove(&unexpected) {
            assert!(false, "Error to get unexpected frame: {:?}", unexpected);
        }
    }
}

impl Drop for AsyncExpector {
    fn drop(&mut self) {
        debug!("~AsyncExpector(), errors: '{:?}'", self.wait_for_all());

        self.stop_flag.store(true, Ordering::Relaxed);
        let x = self.receive_jh.take().unwrap();
        x.join().unwrap();
    }
}

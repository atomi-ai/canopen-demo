use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use embedded_can::Frame;
use embedded_can::nb::Can;
use log::info;
use socketcan::{CanFrame, Socket};
use canopen::node::NodeState::Operational;

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

    info!("xfguo: wait_for_frame 0, {:?}", expected);
    loop {
        {
            let mut container = container.lock().unwrap();
            if let Some(frame) = container.find_and_remove(&expected) {
                info!("xfguo: wait_for_frame 1.1.1; success");
                return Ok(frame);
            }
        }

        if start_time.elapsed() >= timeout {
            info!("xfguo: wait_for_frame 1.1.2; TIMEOUT");
            return Err("Timeout in getting response");
        }

        thread::sleep(Duration::from_millis(100)); // 等待100ms后再次尝试
    }
    // info!("xfguo: wait_for_frame 9. Should not be here");
}

pub struct AsyncExpector {
    stop_flag: Arc<AtomicBool>,
    container: Arc<Mutex<FrameContainer>>,
    join_handlers: Vec<JoinHandle<Result<CanFrame, &'static str>>>,
    receive_jh: Option<JoinHandle<()>>,
}

impl AsyncExpector {
    pub fn new() -> Self {
        let mut res = AsyncExpector {
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
            let mut socket = socketcan::CanSocket::open(INTERFACE_NAME)
                .expect("Failed to open CAN socket");
            while !stop_flag_clone.load(Ordering::Relaxed) {
                info!("xfguo: try to receive a packet");
                match socket.receive() {
                    Ok(frame) => {
                        let mut container = container_clone.lock().unwrap();
                        info!("xfguo: Got frame: {:#x?}", frame);
                        container.push(frame);
                    },
                    Err(_) => { panic!("Errors in receiving packets") },
                }
                info!("xfguo: finish one round to receive a packet");
                thread::sleep(Duration::from_millis(50));
            }
        });

        match receive_thread {
            Ok(rt) => {self.receive_jh = Option::from(rt)}
            Err(_) => {}
        }
    }

    pub fn async_expect(&mut self, expected: CanFrame) {
        let container_clone = self.container.clone();
        self.join_handlers.push(thread::spawn(move || {
            info!("xfguo: wait for frame: {:?}", expected);
            wait_for_frame(container_clone, expected)
        }));
    }

    pub fn wait_for_all(&mut self) -> String {
        let mut res = String::new();
        let len = self.join_handlers.len();
        for i in 0..self.join_handlers.len() {
            let jh = self.join_handlers.remove(len - 1 - i);
            let t = jh.join();
            info!("xfguo: wait result: {:?}", t);
            match t {
                Ok(Ok(_)) => {},
                Ok(Err(err)) => {
                    res += err;
                },
                Err(_) => {res += "join error";}
            }
        }
        info!("xfguo: to return res = '{}'", res);
        res
    }
}

impl Drop for AsyncExpector {
    fn drop(&mut self) {
        info!("~AsyncExpector(), errors: '{:?}'", self.wait_for_all());

        self.stop_flag.store(true, Ordering::Relaxed);
        let x = self.receive_jh.take().unwrap();
        x.join().unwrap();
    }
}

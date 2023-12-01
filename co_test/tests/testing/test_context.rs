extern crate alloc;
extern crate lazy_static;

use alloc::sync::Arc;
use std::io::Write;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use async_std::future::timeout;
use async_std::task;
use chrono::Local;
use env_logger::Builder;
use lazy_static::lazy_static;
use log::{error, info};
use log::LevelFilter;
use socketcan::{EmbeddedFrame, Frame, Socket};
use timer::Timer;

use canopen::node::Node;
use co_test::util as tu;

pub struct TestContext {
    _node_thread: thread::JoinHandle<()>,
}

const EVENT_TIMER_INTERVAL_MS: i64 = 100;


impl TestContext {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Wait for the server up...");
        let content = std::fs::read_to_string(tu::DEMO_EDS_PATH).expect("Failed to read EDS file");
        let s = socketcan::async_io::CanSocket::open(tu::INTERFACE_NAME).unwrap();
        let read_task = s.read_frame();

        info!("Start the testing server thread");
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_clone = is_running.clone();
        let node_thread = thread::Builder::new().name("node_thread".to_string()).spawn(move || {
            let sock = socketcan::CanSocket::open(tu::INTERFACE_NAME)
                .expect("Failed to open CAN socket");
            // Please remember to set "non-blocking" tag for the socket.
            sock.set_nonblocking(true).expect("TODO: panic message");
            let node_arc = Arc::new(Mutex::new(Node::new(2, &content, sock)));
            {
                let mut node_lock = node_arc.lock().unwrap();
                node_lock.init();
            }
            let timer = Timer::new();
            let node_clone = Arc::clone(&node_arc);
            let mut _guard = timer.schedule_repeating(
                chrono::Duration::milliseconds(1), move || {
                    // this can only be used in x86 test env. In MCU, this possibly to introduce
                    // dead-lock.
                    let mut node_lock = node_clone.lock().unwrap();
                    let res = node_lock.event_timer_callback();
                    res
            });

            // start_event_timer(&timer, EVENT_TIMER_INTERVAL_MS, move || {
            //     node.event_timer_callback();
            // });

            is_running_clone.store(true, Ordering::Relaxed);
            loop {
                {
                    let mut node_lock = node_arc.lock().unwrap();
                    node_lock.process_one_frame();
                }
                thread::sleep(Duration::from_micros(200));
            }
        }).unwrap();
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

        let tc = TestContext {
            _node_thread: node_thread,
        };
        Ok(tc)
    }
}

lazy_static! {
    pub static ref CONTEXT: Arc<Mutex<TestContext>> = {
        default_logger_init();

        // Init panic.
        let _ = panic::take_hook();
        panic::set_hook(Box::new(|panic_info| {
            error!("{}. Callstack:\n{:?}", panic_info, backtrace::Backtrace::new());
            std::process::exit(1);
        }));

        let ctx = task::block_on(TestContext::new()).unwrap();
        Arc::new(Mutex::new(ctx))
    };
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
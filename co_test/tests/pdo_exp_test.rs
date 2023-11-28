mod testing;

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::testing::CONTEXT;
use timer::{Guard, Timer};
use log::info;

fn get_current_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

fn start_sync(timer: &Timer) -> Guard {
    timer.schedule_repeating(chrono::Duration::milliseconds(100), move || {
        info!("xfguo: periodically SYNC is enabled at {}", get_current_timestamp_ms());
    })
}

#[test]
fn test_periodical_sync_timer() {
    let _context = CONTEXT.lock().unwrap();
    let timer = Timer::new();

    {
        let mut guard = start_sync(&timer);
        info!("xfguo: start wait at {}", get_current_timestamp_ms());
        thread::sleep(Duration::from_millis(1000));
        info!("xfguo: done wait at {}", get_current_timestamp_ms());
    }
    info!("xfguo: out");
    thread::sleep(Duration::from_secs(1));
}

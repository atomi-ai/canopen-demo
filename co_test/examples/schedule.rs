fn main() {
    extern crate timer;
    extern crate chrono;
    use std::thread;
    use std::sync::{Arc, Mutex};

    let timer = timer::Timer::new();
    // Number of times the callback has been called.
    let count = Arc::new(Mutex::new(0));

    // Start repeating. Each callback increases `count`.
    let guard = {
        let count = count.clone();
        timer.schedule_repeating(chrono::Duration::milliseconds(7), move || {
            *count.lock().unwrap() += 1;
        })
    };

    // Sleep one second. The callback should be called ~200 times.
    thread::sleep(std::time::Duration::new(1, 0));
    let count_result = *count.lock().unwrap();
    // assert!(190 <= count_result && count_result <= 210,
    //         "The timer was called {} times", count_result);
    println!("xfguo: count = {:?}", count_result);

    // Now drop the guard. This should stop the timer.
    drop(guard);
    thread::sleep(std::time::Duration::new(0, 100));

    // Let's check that the count stops increasing.
    let count_start = *count.lock().unwrap();
    thread::sleep(std::time::Duration::new(1, 0));
    let count_stop = *count.lock().unwrap();
    assert_eq!(count_start, count_stop);
}
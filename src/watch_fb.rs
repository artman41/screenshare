use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::{Duration, Instant};
use crate::{Frame, Remarkable};

pub(crate) fn run(frame: Arc<RwLock<Frame>>, time_per_frame: u128) {
    eprintln!("Getting remarkable");
    let mut remarkable = Remarkable::new().unwrap();
    eprintln!("Got remarkable!");
    loop {
        let start = Instant::now();
        let new_frame =
            match remarkable.read_frame() {
                Ok(frame) => frame,
                Err(_) => continue,
            };
        *(frame.write().expect("Couldn't get write lock on frame!")) = new_frame;
        let end = Instant::now();
        let diff_in_ms = end.duration_since(start).as_millis();
        if diff_in_ms < time_per_frame {
            sleep(Duration::from_millis((time_per_frame - diff_in_ms) as u64))
        }
    }
}
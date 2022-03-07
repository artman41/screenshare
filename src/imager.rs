use std::fs::File;
use std::io::Write;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use crate::Frame;

#[derive(Debug, PartialEq, Eq, Default)]
pub(crate) struct AnonFrame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}
pub(crate) fn get_frame(frame_arc: &Arc<RwLock<Frame>>) -> AnonFrame {
    let frame = frame_arc.read().expect("Couldn't read frame");
    let (w, h) = frame.get_size();
    let pixels = frame.get_pixels().read().expect("Couldn't get pixels").to_vec();
    return AnonFrame {
        width: w,
        height: h,
        pixels,
    }
}

pub(crate) fn run(frame_arc: Arc<RwLock<Frame>>) {
    let mut frame = AnonFrame::default();

    loop {
        let new_frame = get_frame(&frame_arc);
        if new_frame.eq(&frame) {
            sleep(Duration::new(1, 0));
            continue
        }

        frame = new_frame;

        let file_name = &format!("{}.bytes", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());

        eprintln!("Got frame {}x{}, writing to {}", &frame.width, &frame.height, &file_name);
        let mut output_file = File::create(file_name).expect("Couldn't open output file");
        let _ = output_file.write_all((&frame).pixels.as_slice());
        drop(output_file);
        sleep(Duration::new(1, 0));
    }
}
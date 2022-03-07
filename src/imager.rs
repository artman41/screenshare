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

        let byte_count = 50;

        let first: Vec<u8> =
            frame.pixels.iter()
                .take(byte_count)
                .map(|&n|n)
                .collect();

        let end: Vec<u8> =
            frame.pixels.iter()
                .rev()
                .take(byte_count)
                .map(|&n|n)
                .collect::<Vec<u8>>()
                .into_iter()
                .rev()
                .collect();

        eprintln!("Got frame {}x{}\nfirst {} bytes: {:?}\nlast {} bytes: {:?}", &frame.width, &frame.height, byte_count, first, byte_count, end);

        let file_name = &format!("{}.bytes", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
        let mut output_file = File::create(file_name).expect("Couldn't open output file");
        let _ = output_file.write_all((&frame).pixels.as_slice());
        drop(output_file);
        sleep(Duration::new(1, 0));
    }
}
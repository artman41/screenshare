use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Context, Result};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};

use crate::remarkable::Version::{UNKNOWN, V1, V2};

const RM_VERSION_PATH: &str = "/sys/devices/soc0/machine";
const FRAME_BUFFER_PATH: &str = "/dev/fb0";

#[derive(Debug)]
enum Version {
    V1,
    V2,
    UNKNOWN,
}

pub struct Remarkable {
    version: Version,
    width: usize,
    height: usize,
    frame_buffer: FrameBuffer,
}

#[derive(Debug)]
pub struct FrameBuffer {
    mem_file: File,
    start: u64,
    cursor: usize,
    size: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Frame {
    width: usize,
    height: usize,
    pixels: Arc<RwLock<Box<[u8]>>>,
}

impl Frame {
    pub fn new(width: usize, height: usize) -> Frame {
        Frame{
            width,
            height,
            pixels: Arc::new(RwLock::default()),
        }
    }

    pub fn get_pixels(&self) -> Arc<RwLock<Box<[u8]>>> {
        return self.pixels.clone();
    }

    pub fn get_size(&self) -> (usize, usize) {
        return (self.width, self.height);
    }
}

impl Remarkable {
    pub fn new() -> Result<Remarkable> {
        let mut str = String::default();
        File::open(RM_VERSION_PATH)?.read_to_string(&mut str)?;
        let version =
            match str.trim_end() {
                "reMarkable 1.0" => V1,
                "reMarkable 2.0" => V2,
                _ => UNKNOWN
            };
        println!("Version: {:?}", version);
        let (width, height) =
            match version {
                V1 => (1408, 1872),
                V2 => (1404, 1872),
                UNKNOWN => return Err(anyhow!("Unknown reMarkable version '{}'", str)),
            };
        println!("width: {}, height: {}", width, height);

        let frame_buffer = FrameBuffer::new(width, height)?;
        println!("frame_buffer: {:?}", frame_buffer);

        Ok(Remarkable{
            version,
            width,
            height,
            frame_buffer,
        })
    }

    pub fn read_frame(&mut self) -> Result<Frame> {
        let frame = Frame::new(self.width, self.height);
        match frame.pixels.write() {
            Ok(mut pixels) => {
                let frame_data =
                    self.frame_buffer.read_pixels(self.height * self.width)
                        .with_context(||"failed to read pixels")
                        .unwrap();
                *pixels = frame_data.to_vec().into()

            },
            Err(_) => return Err(anyhow!("Failed to write frame")),
        }
        return Ok(frame);
    }
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Result<FrameBuffer> {
        // Check xochitl is running
        let s = System::new_all();
        let xochitl =
            s.processes_by_name("xochitl").next()
                .expect("xochitl not found")
                .pid().as_u32();
        println!("xochitl: {:?}", xochitl);
        // Get the line defining the region of memory that xochitl owns of the framebuffer
        let fb0_line =
            BufReader::new(File::open(format!("/proc/{}/maps", xochitl))?)
                .lines()
                .skip_while(|line|matches!(line, Ok(l) if !l.ends_with(FRAME_BUFFER_PATH)))
                .skip(1)
                .next()
                .with_context(|| format!("No line containing {} in /proc/{}/maps file", FRAME_BUFFER_PATH, xochitl))
                .unwrap()
                .with_context(|| format!("Error reading file /proc/{}/maps", xochitl))
                .unwrap();
        println!("fb0_line: {:?}", &fb0_line);
        // Get the last memory address of xochitl's framebuffer
        let xochitl_fb0_addr =
            (&fb0_line).split(" ")
                .next()
                .with_context(|| format!("Error parsing line in /proc/{}/maps", xochitl))
                .unwrap();
        println!("xochitl_fb0_addr: {:?}", xochitl_fb0_addr);

        let addr_split: Vec<&str> = xochitl_fb0_addr.split("-").collect();

        let memory_from_str = &addr_split[0];
        // let memory_to_str = &addr_split[1];

        let memory_from = usize::from_str_radix(&memory_from_str, 16)? + 8;
        // let memory_to = usize::from_str_radix(&memory_to_str, 16)?;

        let mut frame_buffer =
            FrameBuffer{
                mem_file: File::open(format!("/proc/{}/mem", xochitl)).expect(&format!("Failed to open {}", FRAME_BUFFER_PATH)),
                start: memory_from as u64,
                cursor: 0,
                size: width*height,
            };
        println!("frame_buffer: {:?}", frame_buffer);

        frame_buffer.next_frame()?;

        return Ok(frame_buffer);
    }

    pub fn next_frame(&mut self) -> std::io::Result<()> {
        self.mem_file.seek(SeekFrom::Start(self.start))?;
        self.cursor = 0;
        Ok(())
    }

    pub fn read_pixels(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec!(0; count);
        self.read(&mut buf).with_context(||"Failed to read bytes into buffer")?;
        return Ok(buf);
    }
}

impl Read for FrameBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let requested = buf.len();
        let bytes_read =
            if self.cursor + requested < self.size {
                self.mem_file.read(buf)?
            } else {
                let rest = self.size - self.cursor;
                self.mem_file.read(&mut buf[0..rest])?
            };
        self.cursor += bytes_read;
        if self.cursor == self.size {
            self.next_frame()?;
        }
        Ok(bytes_read)
    }
}

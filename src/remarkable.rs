use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::ops::Deref;
use std::time::Instant;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use crate::remarkable::Version::{UNKNOWN, V1, V2};

const RM_VERSION_PATH: &str = "/sys/devices/soc0/machine";

#[derive(Debug)]
enum Version {
    V1,
    V2,
    UNKNOWN,
}

#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub struct Frame {
    width: usize,
    height: usize,
    pub pixels: Vec<u8>,
}

impl Frame {
    pub fn new(width: usize, height: usize) -> Frame {
        Frame{
            width,
            height,
            pixels: vec![],
        }
    }

    pub fn get_rgba(&self, x: usize, y: usize) -> Option<(u8, u8, u8, u8)> {
        let offset = self.width*4*y;
        let pos = offset + x;
        Some((
            *(self.pixels.get(pos + 0)?),
            *(self.pixels.get(pos + 1)?),
            *(self.pixels.get(pos + 2)?),
            *(self.pixels.get(pos + 3)?),
        ))
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
        let mut frame = Frame::new(self.width, self.height);
        frame.pixels = self.frame_buffer.read_pixels(self.height * self.width)?;
        return Ok(frame);
    }
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Result<FrameBuffer> {
        // Check xochitl is running
        let s = System::new_all();
        let xochitl =
            match s.processes_by_name("xochitl").next() {
                Some(proc) => proc.pid().as_u32(),
                None => return Err(anyhow!("xochitl not found"))
            };
        println!("xochitl: {:?}", xochitl);
        // Get the line defining the region of memory that xochitl owns of the framebuffer
        let fb0_line =
            BufReader::new(File::open(format!("/proc/{}/maps", xochitl))?)
                .lines()
                .skip_while(|line|matches!(line, Ok(l) if !l.ends_with("/dev/fb0")))
                .skip(1)
                .next()
                .with_context(|| format!("No line containing /dev/fb0 in /proc/{}/maps file", xochitl))?
                .with_context(|| format!("Error reading file /proc/{}/maps", xochitl))?;
        println!("fb0_line: {:?}", fb0_line);
        // Get the last memory address of xochitl's framebuffer
        let xochitl_fb0_addr =
            fb0_line.split("-")
                .next()
                .with_context(|| format!("Error parsing line in /proc/{}/maps", xochitl))?;
        println!("xochitl_fb0_addr: {:?}", xochitl_fb0_addr);
        // Get 1 byte after xochitl's ownership so that we can access our memory in the frame buffer
        let address =
            usize::from_str_radix(xochitl_fb0_addr, 16)
                .context("Error parsing framebuffer address")?
            + 8;
        println!("address: {:x}", address);

        let mut frame_buffer =
            FrameBuffer{
                mem_file: File::open(format!("/proc/{}/mem", xochitl))?,
                start: address as u64,
                cursor: 0,
                size: width * height,
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
        const BYTES_PER_PIXEL: usize = 4;
        let mut buf = vec!(0; count* BYTES_PER_PIXEL);
        self.read(&mut buf)?;
        return Ok(buf);
    }
}

impl Read for FrameBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let requested = buf.len();
        let bytes_read = if self.cursor + requested < self.size {
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

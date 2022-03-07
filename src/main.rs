use std::sync::{Arc, RwLock};
use std::thread;

use anyhow::Result;
use clap::Parser;

use crate::remarkable::{Frame, Remarkable};

mod remarkable;
mod watch_fb;
mod endpoint;
mod imager;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Opts {
    #[clap(
        long,
        name = "http_port",
        short = 'p',
        help = "Listen for HTTP connection.",
        default_value = "9090"
    )]
    http_port: usize,
    #[clap(
        long,
        name = "websocket_port",
        short = 'w',
        help = "Listen for WS connection.",
        default_value = "9091"
    )]
    websocket_port: usize,
    #[clap(
        long,
        name = "refresh",
        short = 'r',
        help = "Times per second to fetch a new frame",
        default_value = "60"
    )]
    refresh_rate: usize,
    #[clap(
        long,
        name = "debug",
        short = 'd',
        help = "Writes each frame to a file",
        parse(try_from_str),
        default_value = "false"
    )]
    debug: bool,
}

fn main() -> Result<()> {
    let frame: Arc<RwLock<Frame>> = Arc::new(RwLock::default());

    let opts: Opts = Opts::parse();
    let time_per_frame = 1u128/(opts.refresh_rate as u128);

    let frame_c = frame.clone();
    thread::spawn(move || watch_fb::run(frame_c, time_per_frame));

    let frame_c = frame.clone();
    thread::spawn(move || endpoint::run_websocket(frame_c, &format!("0.0.0.0:{}", opts.websocket_port)));
    thread::spawn(move || endpoint::run_http(&format!("0.0.0.0:{}", opts.http_port), opts.websocket_port));

    if opts.debug {
        let frame_c = frame.clone();
        thread::spawn(move || imager::run(frame_c));
    }

    loop {
        thread::park()
    }
}
//mod web;
mod remarkable;

use std::borrow::{Borrow};
use std::future::Future;
use crate::remarkable::{Frame, Remarkable};
use clap::Parser;
use anyhow::{Result};
use std::io::{Write};
use std::net::{TcpListener};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use futures::lock::Mutex;
use std::thread;
use std::thread::{Builder, JoinHandle, sleep};
use std::time::{Duration, Instant};
use futures::executor::block_on;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Opts {
    #[clap(
        long,
        name = "port",
        short = 'l',
        help = "Listen for an (unsecure) TCP connection to send the data to which reduces some load on the reMarkable and improves fps.",
        default_value = "16982"
    )]
    listen: usize,
    #[clap(
        long,
        name = "refresh",
        short = 'r',
        help = "Times per second to fetch a new frame",
        default_value = "60"
    )]
    refresh_rate: usize,
}

//#[actix_rt::main]
fn main() -> Result<()> {
    return block_on(do_thing());
}

async fn do_thing() -> Result<()> {
    let writer_set: Arc<Mutex<Vec<Sender<Frame>>>> = Arc::new(Mutex::new(vec!()));
    eprintln!("Created writer set");
    //env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    //env_logger::init();

    //ffmpeg::init()?;

    let opts: Opts = Opts::parse();

    let writer_set_clone = writer_set.clone();
    thread::spawn(move || {
        eprintln!("Getting remarkable");
        let mut remarkable = Remarkable::new().unwrap();
        eprintln!("Got remarkable!");
        let mut start = Instant::now();
        let mut read_count = 0;
        loop {
            // eprintln!("Getting frames");
            let frame =
                match remarkable.read_frame() {
                    Ok(frame) => frame,
                    Err(_) => continue,
                };
            let frame_ptr = frame.borrow();
            {
                // eprintln!("[READER] Locking writer set!");
                let mut writer_set_locked = block_on(writer_set_clone.lock());
                // eprintln!("[READER] Iterating over writer set!");
                writer_set_locked.retain(|tx: &Sender<Frame>|
                    match tx.send(frame_ptr.deref().clone()) {
                        Ok(_) => true,
                        Err(err) => {
                            eprintln!("Got error {}", err);
                            false
                        }
                    })
            }
            let now = Instant::now();
            let diff = now.duration_since(start);
            let diff_millis = diff.as_millis();
            if diff_millis >= 1000 {
                read_count = 0;
                start = now;
            } else {
                read_count += 1;
                if read_count >= opts.refresh_rate {
                    if diff_millis < 1000 {
                        read_count = 0;
                        eprintln!("sleeping for {}ms", diff_millis);
                        sleep(Duration::from_millis((1000 - diff_millis) as u64));
                        start = Instant::now();
                    }
                }
            }
        }
    });

    let writer_set_ptr = writer_set.clone();
    let listener = TcpListener::bind("0.0.0.0:9000")?;
    'outer: loop {
        eprintln!("Accepting client");
        match listener.accept() {
            Ok((mut stream, _addr)) => {
                eprintln!("Accepted client");
                let (tx, rx) = channel::<Frame>();

                eprintln!("Putting Sender into writer_set");
                {
                    eprintln!("[CLIENT] Locking writer set!");
                    let mut writer_set_locked = writer_set_ptr.lock().await;
                    eprintln!("[CLIENT] Adding Sender to writer set!");
                    writer_set_locked.push(tx.clone());
                }
                eprintln!("Put!");
                'inner: loop {
                    let frame =
                        match rx.recv() {
                            Ok(frame) => frame,
                            Err(_) => continue
                        };
                    eprintln!("Client got frame!");
                    match stream.write(frame.pixels.as_slice()) {
                        Ok(_) => (),
                        Err(_) => break 'inner,
                    }
                    eprintln!("Wrote frame to client stream!");
                };
                eprintln!("Client disconnected, dropping Receiver!");
                drop(rx)
            },
            Err(_) => continue 'outer,
        }
    }

    // let (tx, rx) = channel();

    // thread::spawn(move || {
    //     streamer.
    // });

    // HttpServer::new(|| {
    //     App::new()
    //         // enable logger - always register actix-web Logger middleware last
    //         .wrap(middleware::Logger::default())
    //         // register HTTP requests handlers
    //         .service(actix_web::web::resource("/").to(web::serve))
    //         .service(actix_web::web::resource("/ws").to(web::websocket))
    // })
    // .bind(format!("0.0.0.0:{}", opts.listen?))?
    // .run()
    // .await
}
mod utils;

use clap::Parser;
use std::io::{Read, Write};
use utils::CliArgs;
use zenoh::{Config, Wait};

fn main() {
    let args = CliArgs::parse();
    let config: Config = args.config();
    let to_read = args.read();
    let mut to_write = args.write();

    let s = zenoh::open(config).wait().unwrap();

    // Read from zenoh and write to stdout
    for r in to_read.iter() {
        s.declare_subscriber(r)
            .callback(|sample| {
                let mut stdout = std::io::stdout().lock();
                for slice in sample.payload().slices() {
                    stdout.write_all(slice).unwrap();
                }
            })
            .background()
            .wait()
            .unwrap();
    }

    // Read from stdin and write to zenoh
    let pubs = to_write
        .drain(..)
        .map(|w| {
            s.declare_publisher(w.keyexpr)
                .congestion_control(w.congestion_control)
                .priority(w.priority)
                .express(w.express)
                .wait()
                .unwrap()
        })
        .collect::<Vec<_>>();

    let mut stdin = std::io::stdin();
    let mut buf = vec![0u8; (u16::MAX / 2) as usize];
    while let Ok(len) = stdin.read(&mut buf) {
        if len == 0 {
            // EOF reached - Let's exit
            std::process::exit(-1);
        }

        for p in pubs.iter() {
            p.put(&buf[..len]).wait().unwrap();
        }
    }
}

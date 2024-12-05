mod utils;

use clap::Parser;
use std::io::{Read, Write};
use utils::CliArgs;
use zenoh::{Config, Wait};

fn main() {
    let args = CliArgs::parse();
    let config: Config = args.config();
    let mut to_read = args.read();
    let mut to_write = args.write();

    let s = zenoh::open(config).wait().unwrap();

    // Read from zenoh and write to stdout
    if let Some(r) = to_read.take() {
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
    if let Some(w) = to_write.take() {
        let p = s
            .declare_publisher(w.keyexpr)
            .reliability(w.reliability)
            .congestion_control(w.congestion_control)
            .priority(w.priority)
            .express(w.express)
            .wait()
            .unwrap();

        let mut stdin = std::io::stdin();
        let mut buf = vec![0u8; (u16::MAX / 2) as usize];
        while let Ok(len) = stdin.read(&mut buf) {
            if len == 0 {
                // EOF reached - Let's exit
                std::process::exit(-1);
            }

            p.put(&buf[..len]).wait().unwrap();
        }
    }

    std::thread::park();
}

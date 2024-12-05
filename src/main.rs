mod utils;

use clap::Parser;
use utils::CliArgs;
use zenoh::{qos::CongestionControl, Config, Wait};

fn main() {
    let args = CliArgs::parse();
    let config: Config = args.config();
    let to_read = args.read();
    let to_write = args.write();

    let s = zenoh::open(config).wait().unwrap();

    // Read from zenoh and write to stdout
    for r in to_read.iter() {
        s.declare_subscriber(r)
            .callback(|sample| {
                use std::io::Write;
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
    use std::io::Read;
    let mut stdin = std::io::stdin();
    let mut buf = vec![0u8; (u16::MAX / 2) as usize];

    let pubs = to_write
        .iter()
        .map(|w| {
            s.declare_publisher(w)
                .congestion_control(CongestionControl::Block)
                .wait()
                .unwrap()
        })
        .collect::<Vec<_>>();

    while let Ok(len) = stdin.read(&mut buf) {
        if len == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }
        for p in pubs.iter() {
            p.put(&buf[..len]).wait().unwrap();
        }
    }
}

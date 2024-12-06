mod utils;

use clap::Parser;
use std::io::{Read, Write};
use utils::{CliArgs, Params, PubParams, SubParams};
use zenoh::sample::SampleKind;
use zenoh::{Config, Wait};

fn main() {
    zenoh::try_init_log_from_env();

    let args = CliArgs::parse();
    let config: Config = args.config();
    let params: Params = args.params();

    let s = zenoh::open(config).wait().unwrap();

    // Read from zenoh and write to stdout
    match params {
        Params::Read(SubParams {
            keyexpr,
            ignore_eof,
        }) => {
            let sub = s.declare_subscriber(keyexpr).wait().unwrap();

            while let Ok(sample) = sub.recv() {
                match sample.kind() {
                    SampleKind::Put => {
                        let mut stdout = std::io::stdout().lock();
                        for slice in sample.payload().slices() {
                            stdout.write_all(slice).unwrap();
                        }
                    }
                    SampleKind::Delete => {
                        if !ignore_eof {
                            s.close().wait().unwrap();
                            break;
                        }
                    }
                }
            }
        }
        Params::Write(PubParams {
            keyexpr,
            reliability,
            congestion_control,
            priority,
            express,
            buffer,
        }) => {
            let p = s
                .declare_publisher(keyexpr)
                .reliability(reliability)
                .congestion_control(congestion_control)
                .priority(priority)
                .express(express)
                .wait()
                .unwrap();

            let mut stdin = std::io::stdin().lock();
            let mut buf = vec![0u8; buffer];
            while let Ok(len) = stdin.read(&mut buf) {
                buf.truncate(len);

                if buf.is_empty() {
                    drop(stdin);
                    p.delete().wait().unwrap();
                    s.close().wait().unwrap();
                    break;
                }

                p.put(buf).wait().unwrap();
                buf = vec![0u8; buffer];
            }
        }
    }
}

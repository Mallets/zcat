use serde_json::json;
use std::path::PathBuf;
use zenoh::key_expr::KeyExpr;
use zenoh::{config::WhatAmI, Config};

#[derive(clap::Parser, Debug)]
pub(crate) struct CliArgs {
    #[arg(short, long)]
    /// A configuration file.
    config: Option<PathBuf>,
    #[arg(long)]
    /// Allows arbitrary configuration changes as column-separated KEY:VALUE pairs, where:
    ///   - KEY must be a valid config path.
    ///   - VALUE must be a valid JSON5 string that can be deserialized to the expected type for the KEY field.
    ///
    /// Example: `--cfg='transport/unicast/max_links:2'`
    #[arg(long)]
    cfg: Vec<String>,
    #[arg(short, long)]
    /// The Zenoh session mode [default: peer].
    mode: Option<WhatAmI>,
    #[arg(short = 'e', long)]
    /// Endpoints to connect to.
    connect: Vec<String>,
    #[arg(short, long)]
    /// Endpoints to listen on.
    listen: Vec<String>,
    #[arg(long)]
    /// Disable the multicast-based scouting mechanism.
    no_multicast_scouting: bool,
    // The list of key expressions to read from zenoh and to write to stdout
    #[arg(short, long)]
    read: Vec<String>,
    #[arg(short, long)]
    // The list of key expressions to read from stdin and to write to zenoh
    write: Vec<String>,
}

impl CliArgs {
    pub(crate) fn read(&self) -> Vec<KeyExpr<'static>> {
        self.read
            .iter()
            .cloned()
            .map(|s| KeyExpr::try_from(s).unwrap())
            .collect()
    }

    pub(crate) fn write(&self) -> Vec<KeyExpr<'static>> {
        self.write
            .iter()
            .cloned()
            .map(|s| KeyExpr::try_from(s).unwrap())
            .collect()
    }

    pub(crate) fn config(&self) -> Config {
        let mut config = match &self.config {
            Some(path) => Config::from_file(path).unwrap(),
            None => Config::default(),
        };
        if let Some(mode) = self.mode {
            config
                .insert_json5("mode", &json!(mode.to_str()).to_string())
                .unwrap();
        }

        if !self.connect.is_empty() {
            config
                .insert_json5("connect/endpoints", &json!(self.connect).to_string())
                .unwrap();
        }
        if !self.listen.is_empty() {
            config
                .insert_json5("listen/endpoints", &json!(self.listen).to_string())
                .unwrap();
        }
        if self.no_multicast_scouting {
            config
                .insert_json5("scouting/multicast/enabled", &json!(false).to_string())
                .unwrap();
        }
        for json in &self.cfg {
            if let Some((key, value)) = json.split_once(':') {
                if let Err(err) = config.insert_json5(key, value) {
                    eprintln!("`--cfg` argument: could not parse `{json}`: {err}");
                    std::process::exit(-1);
                }
            } else {
                eprintln!("`--cfg` argument: expected KEY:VALUE pair, got {json}");
                std::process::exit(-1);
            }
        }
        config
    }
}

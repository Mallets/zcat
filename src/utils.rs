use serde_json::json;
use std::path::PathBuf;
use zenoh::{
    config::{Config, WhatAmI},
    key_expr::KeyExpr,
    qos::{CongestionControl, Priority, Reliability},
};

/********************/
/*     Config       */
/********************/
#[derive(clap::Subcommand, Clone, Debug)]
enum CliCommand {
    #[clap(short_flag = 'r')]
    Read {
        keyexpr: String,

        #[arg(short = 'i', long)]
        ignore_eof: bool,
    },
    #[clap(short_flag = 'w')]
    Write {
        keyexpr: String,

        #[arg(short = 't', long)]
        #[clap(value_parser(["reliable", "besteffort"]))]
        reliability: Option<String>,

        #[arg(short = 'd', long)]
        #[clap(value_parser(["drop", "block"]))]
        congestion_control: Option<String>,

        #[arg(short, long)]
        #[clap(value_parser(["1", "2", "3", "4", "5", "6", "7"]))]
        priority: Option<u8>,

        #[arg(short, long)]
        express: bool,

        /// The buffer size to read on
        #[arg(short, long, default_value = "32768")]
        buffer: usize,
    },
}

#[derive(clap::Parser, Debug)]
pub(crate) struct CliArgs {
    /* zcat config */
    /// The list of key expressions to read from zenoh and to write to stdout
    #[command(subcommand)]
    command: CliCommand,

    /* Zenoh config */
    /// The Zenoh session mode [default: peer].
    #[arg(short, long)]
    mode: Option<WhatAmI>,

    /// Endpoints to connect to.
    #[arg(short = 'e', long)]
    connect: Vec<String>,

    /// Endpoints to listen on.
    #[arg(short, long)]
    listen: Vec<String>,

    #[arg(long)]
    /// Disable the multicast-based scouting mechanism.
    no_multicast_scouting: bool,

    /// A configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Allows arbitrary configuration changes as column-separated KEY:VALUE pairs, where:
    ///   - KEY must be a valid config path.
    ///   - VALUE must be a valid JSON5 string that can be deserialized to the expected type for the KEY field.
    ///
    /// Example: `--cfg='transport/unicast/max_links:2'`
    #[arg(long)]
    cfg: Vec<String>,
}

impl CliArgs {
    pub(crate) fn params(&self) -> Params {
        match &self.command {
            CliCommand::Read {
                keyexpr,
                ignore_eof,
            } => Params::Read(SubParams {
                keyexpr: KeyExpr::try_from(keyexpr.to_string()).unwrap(),
                ignore_eof: *ignore_eof,
            }),
            CliCommand::Write {
                keyexpr,
                reliability,
                congestion_control,
                priority,
                express,
                buffer,
            } => Params::Write(PubParams {
                keyexpr: KeyExpr::try_from(keyexpr.to_string()).unwrap(),
                reliability: reliability
                    .as_ref()
                    .map(|s| match s.as_str() {
                        "reliable" => Reliability::Reliable,
                        "besteffort" => Reliability::BestEffort,
                        _ => unreachable!(),
                    })
                    .unwrap_or_default(),
                congestion_control: congestion_control
                    .as_ref()
                    .map(|s| match s.as_str() {
                        "drop" => CongestionControl::Drop,
                        "block" => CongestionControl::Block,
                        _ => unreachable!(),
                    })
                    .unwrap_or_default(),
                priority: priority
                    .as_ref()
                    .map(|s| Priority::try_from(*s).unwrap())
                    .unwrap_or_default(),
                express: *express,
                buffer: *buffer,
            }),
        }
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

/********************/
/*    PSubParams    */
/********************/
pub(crate) enum Params {
    Write(PubParams),
    Read(SubParams),
}

#[derive(Clone, Debug)]
pub(crate) struct PubParams {
    pub(crate) keyexpr: KeyExpr<'static>,
    pub(crate) reliability: Reliability,
    pub(crate) congestion_control: CongestionControl,
    pub(crate) priority: Priority,
    pub(crate) express: bool,
    pub(crate) buffer: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct SubParams {
    pub(crate) keyexpr: KeyExpr<'static>,
    pub(crate) ignore_eof: bool,
}

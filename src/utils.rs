use serde_json::json;
use std::{path::PathBuf, str::FromStr};
use zenoh::{
    config::{Config, WhatAmI},
    key_expr::KeyExpr,
    qos::{CongestionControl, Priority, Reliability},
};

/********************/
/*     Config       */
/********************/
#[derive(clap::Parser, Debug)]
pub(crate) struct CliArgs {
    /* zcat config */
    /// The list of key expressions to read from zenoh and to write to stdout
    #[arg(short, long)]
    read: Vec<String>,

    /// The list of key expressions to read from stdin and to write to zenoh
    #[arg(short, long)]
    write: Vec<PubParams>,

    /// The buffer size to read on
    #[arg(short, long, default_value = "32768")]
    buffer: Vec<String>,

    /* Zenoh config */
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
}

impl CliArgs {
    pub(crate) fn read(&self) -> Vec<KeyExpr<'static>> {
        self.read
            .iter()
            .cloned()
            .map(|s| KeyExpr::try_from(s).unwrap())
            .collect()
    }

    pub(crate) fn write(&self) -> Vec<PubParams> {
        self.write.clone()
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
/*    PubParams     */
/********************/
#[derive(Clone, Debug)]
pub(crate) struct PubParams {
    pub keyexpr: KeyExpr<'static>,
    pub reliability: Reliability,
    pub congestion_control: CongestionControl,
    pub priority: Priority,
    pub express: bool,
}

impl FromStr for PubParams {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format:
        // - <keyexpr>:<reliable|besteffort>:<drop|block>:<priority as u8>:<true|false>
        // - foo/bar:drop:5:false
        let mut iter = s.split(':');

        let Some(ke) = iter.next() else {
            return Err("KeyExpr must be provided".to_string());
        };
        let keyexpr = KeyExpr::try_from(ke.to_string()).map_err(|e| format!("{}", e))?;

        let reliability: Reliability = match iter.next() {
            Some("reliable") => Reliability::Reliable,
            Some("besteffort") => Reliability::BestEffort,
            Some(p) => {
                return Err(format!(
                    "Invalid reliability value: {p}. Valid values are: 'reliable' or 'besteffort'."
                ));
            }
            None => Reliability::default(),
        };

        let congestion_control: CongestionControl = match iter.next() {
            Some("drop") => CongestionControl::Drop,
            Some("block") => CongestionControl::Block,
            Some(p) => {
                return Err(format!(
                    "Invalid congestion control value: {p}. Valid values are: 'drop' or 'block'."
                ));
            }
            None => CongestionControl::default(),
        };

        let priority: Priority = match iter.next() {
            Some(p) => {
                let i: u8 = p.parse().map_err(|e| format!("{}", e))?;
                i.try_into().map_err(|e| format!("{}", e))?
            }
            None => Priority::default(),
        };

        let express: bool = match iter.next() {
            Some(p) => p.parse().map_err(|e| format!("{}", e))?,
            None => false,
        };

        if iter.next().is_some() {
            return Err("Too many parameters".to_string());
        }

        Ok(Self {
            keyexpr,
            reliability,
            congestion_control,
            priority,
            express,
        })
    }
}

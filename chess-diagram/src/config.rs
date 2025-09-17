use clap::Parser;
use shakmaty::Color;
// use log::LevelFilter;
use std::sync::OnceLock;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Path to a UCI engine
    #[arg(short, long, value_name = "ENGINE")]
    engine: String,

    /// Optional arguments to pass to the engine (separated by ";")
    ///
    /// Example: --engine-args '--uci;--quiet'
    #[arg(long, value_name = "ARGS", allow_hyphen_values = true)]
    engine_args: Option<String>,

    /// Engine color
    #[arg(
        long,
        value_name = "ENGINE COLOR",
        allow_hyphen_values = true,
        default_value = "black"
    )]
    engine_color: Color,

    /// UCI option
    ///
    /// This argument can be repeated. UCI options are of the
    /// form "ID[:VALUE]". VALUE can be missing if not needed (buttons).  
    /// See the engine's documentation for available options and their
    /// default values.
    ///
    /// Example: --uci-option 'Threads:2' --uci-option 'Skill Level:12'
    #[arg(long)]
    uci_option: Vec<String>,

    /// Opening
    ///
    /// Force moves into this opening, name is a pattern.
    ///
    /// Example: --opening french
    #[arg(long)]
    opening: Option<String>,

    /// ECO codes
    ///
    /// Force moves into these openings, eco code is a pattern.
    ///
    /// Example: --eco A21 --eco A3
    #[arg(long)]
    eco: Vec<String>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::parse)
}

pub fn get_engine() -> String {
    config().engine.clone()
}

pub fn get_engine_args() -> Option<Vec<String>> {
    config()
        .engine_args
        .clone()
        .map(|args| args.split(";").map(|arg| arg.to_string()).collect())
}

pub fn get_engine_color() -> Color {
    config().engine_color.clone()
}

pub fn get_engine_options() -> Vec<(String, Option<String>)> {
    config()
        .uci_option
        .iter()
        .map(|opt| {
            let parts: Vec<String> = opt.split(":").take(2).map(|s| s.to_string()).collect();
            match parts.len() {
                0 => (String::new(), None),
                1 => (parts[0].clone(), None),
                _ => (parts[0].clone(), Some(parts[1].clone())),
            }
        })
        .collect()
}

pub fn get_eco_codes() -> Vec<String> {
    config().eco.clone()
}

pub fn get_opening() -> Option<String> {
    config().opening.clone()
}

// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
// pub enum LogLevel {
//     /// A level lower than all log levels.
//     Off,
//     /// Corresponds to the `Error` log level.
//     Error,
//     /// Corresponds to the `Warn` log level.
//     Warn,
//     /// Corresponds to the `Info` log level.
//     Info,
//     /// Corresponds to the `Debug` log level.
//     Debug,
//     /// Corresponds to the `Trace` log level.
//     Trace,
// }

// pub fn get_log_level() -> LevelFilter {
//     match config().log_level {
//         LogLevel::Off => LevelFilter::Off,
//         LogLevel::Error => LevelFilter::Error,
//         LogLevel::Warn => LevelFilter::Warn,
//         LogLevel::Info => LevelFilter::Info,
//         LogLevel::Debug => LevelFilter::Debug,
//         LogLevel::Trace => LevelFilter::Trace,
//     }
// }

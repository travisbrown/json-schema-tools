use clap::Parser;
use json_schema_tools::{compose::compose, lint::lint};
use serde_json::Value;
use simplelog::LevelFilter;
use std::fs::File;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    init_logging(opts.verbose)?;

    match opts.command {
        Command::Lint { schema } => {
            let value: Value = serde_json::from_reader(File::open(schema)?)?;

            for issue in lint(&value) {
                println!("{:?}", issue);
            }
        }
        Command::Compose { schema, referenced } => {
            let base: Value = serde_json::from_reader(File::open(schema)?)?;
            let referenced = referenced
                .into_iter()
                .map(|path| {
                    File::open(path).map_err(Error::from).and_then(|reader| {
                        serde_json::from_reader::<_, Value>(reader).map_err(Error::from)
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let composed = compose(
                &base,
                &referenced
                    .into_iter()
                    .map(|value| (None, value))
                    .collect::<Vec<_>>(),
            )?;

            println!("{}", composed)
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[clap(name = "json-schema-tools", version, author)]
struct Opts {
    /// Level of verbosity
    #[clap(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Lint {
        /// Main schema path
        #[clap(short, long)]
        schema: PathBuf,
    },
    Compose {
        /// Main schema path
        #[clap(short, long)]
        schema: PathBuf,
        /// Referenced schema paths
        #[clap(short, long, value_parser, num_args = 0.., value_delimiter = ',')]
        referenced: Vec<PathBuf>,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Logging initialization error")]
    LogInit(#[from] log::SetLoggerError),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("Compose error")]
    Compose(#[from] json_schema_tools::compose::Error),
}

fn select_log_level_filter(verbosity: u8) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

fn init_logging(verbosity: u8) -> Result<(), log::SetLoggerError> {
    simplelog::TermLogger::init(
        select_log_level_filter(verbosity),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
}

#![forbid(unsafe_code)]

use std::env::set_var;

use clap::Parser;
pub(crate) use eyre::Result;
use metrics::Database;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(long, visible_alias = "db", value_name = "FILE")]
    database: String,
    #[clap(long, visible_alias = "gc", value_parser, value_name = "DAYS")]
    cleanup_older: Option<u32>,
    #[clap(long, short, value_name = "NETWORKS NAMES")]
    ignored_networks: Vec<String>,
    #[clap(short, long = "verbose", parse(from_occurrences))]
    verbosity: usize,
    // NOTE: This is only used for benchmarking and testing purposes and should not be used in normally.
    #[clap(long, value_name = "NUMBER OF SNAPSHOTS", hide(true))]
    times: Option<u32>,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let app = Cli::parse();
    env::setup_env()?;

    if app.verbosity > 2 {
        set_var("LOG_LEVEL", "trace");
    } else if app.verbosity == 2 {
        set_var("LOG_LEVEL", "debug");
    } else if app.verbosity == 1 {
        set_var("LOG_LEVEL", "info");
    }
    log::setup_logger();

    let (mut database, file, path) = Database::from_file_with_write(&app.database)?;
    if let Some(times) = app.times {
        for _ in 0..times {
            database.take_snapshot(
                app.ignored_networks
                    .iter()
                    .map(|n| n.as_ref())
                    .collect::<Vec<&str>>()
                    .as_ref(),
            )?;
        }
    } else {
        database.take_snapshot(
            app.ignored_networks
                .iter()
                .map(|n| n.as_ref())
                .collect::<Vec<&str>>()
                .as_ref(),
        )?;
    }
    database.write_and_close_file(file, &path)?;

    Ok(())
}

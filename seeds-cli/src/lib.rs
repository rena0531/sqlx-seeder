use crate::opt::{Command, DatabaseCommand, SeedsCommand};
use anyhow::anyhow;
use dotenv::dotenv;
use std::env;

mod database;
// mod migration;
mod seeds;
mod opt;

pub use crate::opt::Opt;

pub async fn run(opt: Opt) -> anyhow::Result<()> {
    match opt.command {
        Command::Seeds(seeds) => match seeds.command {
            SeedsCommand::Run {
                dry_run,
                ignore_missing,
            } => seeds::run()
        },
    };

    Ok(())
}

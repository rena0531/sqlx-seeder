// use crate::opt::{Command, DatabaseCommand, MigrateCommand};
// use anyhow::anyhow;
// use dotenv::dotenv;
// use std::env;

// mod database;
// // mod migration;
// mod migrate;

// pub use crate::opt::Opt;

// pub async fn run(opt: Opt) -> anyhow::Result<()> {
//     match opt.command {
//         Command::Migrate(migrate) => match migrate.command {
//             MigrateCommand::Run {
//                 dry_run,
//                 ignore_missing,
//             } => migrate::run()
//         },
//     };

//     Ok(())
// }

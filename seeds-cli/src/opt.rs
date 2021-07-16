use clap::Clap;

#[derive(Clap, Debug)]
pub struct Opt {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(short = 'D', long)]
    pub database_url: Option<String>,
}

#[derive(Clap, Debug)]
pub enum Command {
    #[clap(alias = "db")]
    Database(DatabaseOpt),

    /// Generate query metadata to support offline compile-time verification.
    ///
    /// Saves metadata for all invocations of `query!` and related macros to `sqlx-data.json`
    /// in the current directory, overwriting if needed.
    ///
    /// During project compilation, the absence of the `DATABASE_URL` environment variable or
    /// the presence of `SQLX_OFFLINE` (with a value of `true` or `1`) will constrain the
    /// compile-time verification to only read from the cached query metadata.
    #[clap(alias = "prep")]
    Prepare {
        /// Run in 'check' mode. Exits with 0 if the query metadata is up-to-date. Exits with
        /// 1 if the query metadata needs updating.
        #[clap(long)]
        check: bool,

        /// Generate a single top-level `sqlx-data.json` file when using a cargo workspace.
        #[clap(long)]
        merged: bool,

        /// Arguments to be passed to `cargo rustc ...`.
        #[clap(last = true)]
        args: Vec<String>,
    },

    #[clap(alias = "mig")]
    Seeds(SeedsOpt),
}

/// Group of commands for creating and dropping your database.
#[derive(Clap, Debug)]
pub struct DatabaseOpt {
    #[clap(subcommand)]
    pub command: DatabaseCommand,
}

#[derive(Clap, Debug)]
pub enum DatabaseCommand {
    /// Creates the database specified in your DATABASE_URL.
    Create,

    /// Drops the database specified in your DATABASE_URL.
    Drop {
        /// Automatic confirmation. Without this option, you will be prompted before dropping
        /// your database.
        #[clap(short)]
        yes: bool,
    },

    /// Drops the database specified in your DATABASE_URL, re-creates it, and runs any pending seeds.
    Reset {
        /// Automatic confirmation. Without this option, you will be prompted before dropping
        /// your database.
        #[clap(short)]
        yes: bool,

        /// Path to folder containing seeds.
        #[clap(long, default_value = "seeds")]
        source: String,
    },

    /// Creates the database specified in your DATABASE_URL and runs any pending seeds.
    Setup {
        /// Path to folder containing seeds.
        #[clap(long, default_value = "seeds")]
        source: String,
    },
}

/// Group of commands for creating and running seeds.
#[derive(Clap, Debug)]
pub struct SeedsOpt {
    /// Path to folder containing seeds.
    #[clap(long, default_value = "seeds")]
    pub source: String,

    #[clap(subcommand)]
    pub command: SeedsCommand,
}

#[derive(Clap, Debug)]
pub enum SeedsCommand {
    /// Create a new seeds with the given description,
    /// and the current time as the version.
    Add {
        description: String,

        /// If true, creates a pair of up and down seeds files with same version
        /// else creates a single sql file
        #[clap(short)]
        reversible: bool,
    },

    /// Run all pending seeds.
    Run {
        /// List all the seeds to be run without applying
        #[clap(long)]
        dry_run: bool,

        /// Ignore applied seeds that missing in the resolved seeds
        #[clap(long)]
        ignore_missing: bool,
    },

    /// Revert the latest seeds with a down file.
    Revert {
        /// List the seeds to be reverted without applying
        #[clap(long)]
        dry_run: bool,

        /// Ignore applied seeds that missing in the resolved seeds
        #[clap(long)]
        ignore_missing: bool,
    },

    /// List all available seeds.
    Info,
}

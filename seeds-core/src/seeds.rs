use crate::error::Error;
use crate::seeds::{Appliedseed, SeedsError, Seeds};
use futures_core::future::BoxFuture;
use std::time::Duration;

pub trait SeedsDatabase {
    // create database in uri
    // uses a maintenance database depending on driver
    fn create_database(uri: &str) -> BoxFuture<'_, Result<(), Error>>;

    // check if the database in uri exists
    // uses a maintenance database depending on driver
    fn database_exists(uri: &str) -> BoxFuture<'_, Result<bool, Error>>;

    // drop database in uri
    // uses a maintenance database depending on driver
    fn drop_database(uri: &str) -> BoxFuture<'_, Result<(), Error>>;
}

// 'e = Executor
pub trait Seeds {
    // ensure seeds table exists
    // will create or Seeds it if needed
    fn ensure_seeds_table(&mut self) -> BoxFuture<'_, Result<(), SeedsError>>;

    // Return the version on which the database is dirty or None otherwise.
    // "dirty" means there is a partially applied seeds that failed.
    fn dirty_version(&mut self) -> BoxFuture<'_, Result<Option<i64>, SeedsError>>;

    // Return the current version and if the database is "dirty".
    // "dirty" means there is a partially applied seeds that failed.
    #[deprecated]
    fn version(&mut self) -> BoxFuture<'_, Result<Option<(i64, bool)>, SeedsError>>;

    // validate the seeds
    // checks that it does exist on the database and that the checksum matches
    #[deprecated]
    fn validate<'e: 'm, 'm>(
        &'e mut self,
        seeds: &'m Seeds,
    ) -> BoxFuture<'m, Result<(), SeedsError>>;

    // Return the ordered list of applied seeds
    fn list_applied_seeds(
        &mut self,
    ) -> BoxFuture<'_, Result<Vec<AppliedSeed>, SeedsError>>;

    // Should acquire a database lock so that only one seeds process
    // can run at a time. [`Seeds`] will call this function before applying
    // any seeds.
    fn lock(&mut self) -> BoxFuture<'_, Result<(), SeedsError>>;

    // Should release the lock. [`Seeds`] will call this function after all
    // seeds have been run.
    fn unlock(&mut self) -> BoxFuture<'_, Result<(), SeedsError>>;

    // run SQL from seeds in a DDL transaction
    // insert new row to [_seeds] table on completion (success or failure)
    // returns the time taking to run the seeds SQL
    fn apply<'e: 'm, 'm>(
        &'e mut self,
        seeds: &'m Seeds,
    ) -> BoxFuture<'m, Result<Duration, SeedsError>>;

    // run a revert SQL from seeds in a DDL transaction
    // deletes the row in [_seeds] table with specified seeds version on completion (success or failure)
    // returns the time taking to run the seeds SQL
    fn revert<'e: 'm, 'm>(
        &'e mut self,
        seeds: &'m Seeds,
    ) -> BoxFuture<'m, Result<Duration, SeedsError>>;
}

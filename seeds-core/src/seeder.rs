use crate::acquire::Acquire;
use crate::seeds::{SEEDS, SeedsError, SeedsSource};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::slice;

#[derive(Debug)]
pub struct Seeder {
    pub seeds: Cow<'static, [Seeds]>,
    pub ignore_missing: bool,
}

fn validate_applied_seeds(
    applied_seeds: &[AppliedSeeds],
    seeder: &Seeder,
) -> Result<(), SeedsError> {
    if seeder.ignore_missing {
        return Ok(());
    }

    let seeds: HashSet<_> = seeder.iter().map(|m| m.version).collect();

    for applied_seeds in applied_seeds {
        if !seeds.contains(&applied_seeds.version) {
            return Err(SeedsError::VersionMissing(applied_seeds.version));
        }
    }

    Ok(())
}

impl Seeder {
    /// Creates a new instance with the given source.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sqlx_core::seeds::SeedsError;
    /// # fn main() -> Result<(), SeedsError> {
    /// # sqlx_rt::block_on(async move {
    /// # use sqlx_core::seeds::Seeder;
    /// use std::path::Path;
    ///
    /// // Read seeds from a local folder: ./seeds
    /// let m = Seeder::new(Path::new("./seeds")).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// See [SeedsSource] for details on structure of the `./seeds` directory.
    pub async fn new<'s, S>(source: S) -> Result<Self, SeedsError>
    where
        S: SeedsSource<'s>,
    {
        Ok(Self {
            seeds: Cow::Owned(source.resolve().await.map_err(SeedsError::Source)?),
            ignore_missing: false,
        })
    }

    /// Specify should ignore applied seeds that missing in the resolved seeds.
    pub fn set_ignore_missing(&mut self, ignore_missing: bool) -> &Self {
        self.ignore_missing = ignore_missing;
        self
    }

    /// Get an iterator over all known seeds.
    pub fn iter(&self) -> slice::Iter<'_, Seeds> {
        self.seeds.iter()
    }

    /// Run any pending seeds against the database; and, validate previously applied seeds
    /// against the current seeds source to detect accidental changes in previously-applied seeds.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sqlx_core::seeds::SeedsError;
    /// # #[cfg(feature = "sqlite")]
    /// # fn main() -> Result<(), SeedsError> {
    /// #     sqlx_rt::block_on(async move {
    /// # use sqlx_core::seeds::Seeder;
    /// let m = Seeder::new(std::path::Path::new("./seeds")).await?;
    /// let pool = sqlx_core::sqlite::SqlitePoolOptions::new().connect("sqlite::memory:").await?;
    /// m.run(&pool).await
    /// #     })
    /// # }
    /// ```
    pub async fn run<'a, A>(&self, seeder: A) -> Result<(), SeedsError>
    where
        A: Acquire<'a>,
        <A::Connection as Deref>::Target: Seeds,
    {
        let mut conn = seeder.acquire().await?;

        // lock the database for exclusive access by the seeder
        conn.lock().await?;

        // creates [_seeds] table only if needed
        // eventually this will likely seeds previous versions of the table
        conn.ensure_seeds_table().await?;

        let version = conn.dirty_version().await?;
        if let Some(version) = version {
            return Err(SeedsError::Dirty(version));
        }

        let applied_seeds = conn.list_applied_seeds().await?;
        validate_applied_seeds(&applied_seeds, self)?;

        let applied_seeds: HashMap<_, _> = applied_seeds
            .into_iter()
            .map(|m| (m.version, m))
            .collect();

        for seeds in self.iter() {
            if seeds.seeds_type.is_down_seeds() {
                continue;
            }

            match applied_seeds.get(&seeds.version) {
                Some(applied_seeds) => {
                    if seeds.checksum != applied_seeds.checksum {
                        return Err(SeedsError::VersionMismatch(seeds.version));
                    }
                }
                None => {
                    conn.apply(seeds).await?;
                }
            }
        }

        // unlock the seeder to allow other seeders to run
        // but do nothing as we already seedsd
        conn.unlock().await?;

        Ok(())
    }
}

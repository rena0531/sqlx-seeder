use crate::error::{BoxDynError, Error};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SeedsError {
    #[error("while executing seeds: {0}")]
    Execute(#[from] Error),

    #[error("while resolving seeds: {0}")]
    Source(#[source] BoxDynError),

    #[error("seeds {0} was previously applied but is missing in the resolved seeds")]
    VersionMissing(i64),

    #[error("seeds {0} was previously applied but has been modified")]
    VersionMismatch(i64),

    #[error("cannot mix reversible seeds with simple seeds. All seeds should be reversible or simple seeds")]
    InvalidMixReversibleAndSimple,

    // NOTE: this will only happen with a database that does not have transactional DDL (.e.g, MySQL or Oracle)
    #[error(
        "seeds {0} is partially applied; fix and remove row from `_sqlx_seeds` table"
    )]
    Dirty(i64),
}

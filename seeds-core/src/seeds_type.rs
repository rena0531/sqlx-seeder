/// Seeds Type represents the type of seeds
#[derive(Debug, Copy, Clone)]
pub enum SeedsType {
    /// Simple seeds are single file seeds with no up / down queries
    Simple,

    /// ReversibleUp seeds represents the  add or update part of a reversible seeds
    /// It is expected the every seeds of this type will have a corresponding down file
    ReversibleUp,

    /// ReversibleDown seeds represents the  delete or downgrade part of a reversible seeds
    /// It is expected the every seeds of this type will have a corresponding up file
    ReversibleDown,
}

impl SeedsType {
    pub fn from_filename(filename: &str) -> Self {
        if filename.ends_with(SeedsType::ReversibleUp.suffix()) {
            SeedsType::ReversibleUp
        } else if filename.ends_with(SeedsType::ReversibleDown.suffix()) {
            SeedsType::ReversibleDown
        } else {
            SeedsType::Simple
        }
    }

    pub fn is_reversible(&self) -> bool {
        match self {
            SeedsType::Simple => false,
            SeedsType::ReversibleUp => true,
            SeedsType::ReversibleDown => true,
        }
    }

    pub fn is_down_seeds(&self) -> bool {
        match self {
            SeedsType::Simple => false,
            SeedsType::ReversibleUp => false,
            SeedsType::ReversibleDown => true,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SeedsType::Simple => "seeds",
            SeedsType::ReversibleUp => "seeds",
            SeedsType::ReversibleDown => "revert",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            SeedsType::Simple => ".sql",
            SeedsType::ReversibleUp => ".up.sql",
            SeedsType::ReversibleDown => ".down.sql",
        }
    }

    pub fn file_content(&self) -> &'static str {
        match self {
            SeedsType::Simple => "-- Add seeds script here\n",
            SeedsType::ReversibleUp => "-- Add up seeds script here\n",
            SeedsType::ReversibleDown => "-- Add down seeds script here\n",
        }
    }
}

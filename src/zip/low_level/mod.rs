//! # Low-level access to a ZIP archive
//!
//! This module provides types and functions to navigate around an archive,
//! whether correctly formed or not.

mod cdfh;
mod eocd;

pub use self::eocd::{
    EndOfCentralDirectoryIterator,
    locate_end_of_central_directory
};

pub use self::cdfh::CentralDirectoryFileHeaderIterator;

//! # Low-level access to a ZIP archive
//!
//! This module provides types and functions to navigate around an archive,
//! whether correctly formed or not.

pub use super::eocd_iter::{
    EndOfCentralDirectoryIterator,
    locate_end_of_central_directory
};

pub use super::cdfh_iter::CentralDirectoryFileHeaderIterator;

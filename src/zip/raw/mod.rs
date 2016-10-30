//! # Raw ZIP structures
//!
//! This module provides types to interpret raw bytes as particular structures
//! of the ZIP specification.

mod cdfh;
mod dd;
mod eocd;
mod lfh;

pub use self::cdfh::CentralDirectoryFileHeaderReader;
pub use self::dd::DataDescriptorReader;
pub use self::eocd::EndOfCentralDirectoryReader;
pub use self::lfh::LocalFileHeaderReader;

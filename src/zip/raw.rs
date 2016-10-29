//! # Raw ZIP structures
//!
//! This module provides types to interpret raw bytes as particular structures
//! of the ZIP specification.

pub use super::eocd::EndOfCentralDirectoryReader;
pub use super::cdfh::CentralDirectoryFileHeaderReader;
pub use super::lfh::LocalFileHeaderReader;
pub use super::dd::DataDescriptorReader;

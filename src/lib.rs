#![deny(missing_docs)]

//! # Arkiv
//!
//! Arkiv is a library for reading and writing various archive formats.
//! 
//! 
//! # Architecture
//! 
//! The library exposes a few top-level modules:
//! 
//!  - one high-level uniform module, allowing reading/writing archive formats generically
//!  - one low-level module per format, allowing access to all the specifities of the format
//!  - one api module, containing the user-definable traits to operate with the library
//!  - one ffi module, containing the C FFI modules mirroring each of the above modules
//! 
//! The library also contains a utils module, used to share commonly used code across components.
//! 
//! 
//! # Covered formats
//! 
//! Complete:
//! 
//!  - NONE
//! 
//! Partial:
//! 
//!  - NONE
//! 
//! Planned:
//! 
//!  - 7z
//!  - rar
//!  - tar
//!  - zip

pub mod api;
pub mod zip;

mod utils;

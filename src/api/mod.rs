//! # Common API used by all formats

use std::ops::Range;

use super::utils::slice;

/// A trait used to access portions of a buffer at a time, without (necessarily)
/// having the full buffer in memory at any point in time.
pub trait Reader {
    /// Returns the size of the archive.
    fn size(&self) -> usize;

    /// Returns the slice of bytes corresponding to the range, or `None` if it
    /// is infeasible (erroneous range, out-of-bounds access).
    ///
    /// An instance of this traits juggling multiple buffers may need to use
    /// interior mutability.
    fn get(&self, range: Range<usize>) -> Option<&[u8]>;
}

impl Reader for [u8] {
    fn size(&self) -> usize { self.len() }

    fn get(&self, range: Range<usize>) -> Option<&[u8]> { slice(self, range) }
}

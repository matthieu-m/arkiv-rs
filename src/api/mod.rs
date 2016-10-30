//! # Common API used by all formats

use std::ops::Range;

use utils::intersect_slice;

/// A trait used to access portions of a buffer at a time, without (necessarily)
/// having the full buffer in memory at any point in time.
pub trait Reader {
    /// Returns the size of the archive.
    fn size(&self) -> usize;

    /// Returns the slice of bytes corresponding to the intersection of the
    /// provided range and the archive.
    ///
    /// An instance of this traits juggling multiple buffers may need to use
    /// interior mutability.
    fn get(&self, range: Range<usize>) -> &[u8];
}

impl Reader for [u8] {
    fn size(&self) -> usize { self.len() }

    fn get(&self, range: Range<usize>) -> &[u8] { intersect_slice(self, range) }
}

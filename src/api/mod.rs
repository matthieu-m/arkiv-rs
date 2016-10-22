//! # Common API used by all formats
use std::ops::Range;
use std::slice::from_raw_parts;

/// A slice of bytes.
///
/// It is used in preference to a raw `&[u8]` to avoid accidentally calling the
/// Index operator of `&[u8]` which panics when called with an incorrect index.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Slice<'a> {
    data: &'a [u8],
}

impl<'a> Slice<'a> {
    pub fn new(data: &'a [u8]) -> Slice<'a> {
        Slice { data: data }
    }

    /// Returns the number of bytes in the slice.
    pub fn len(&self) -> usize { self.data.len() }

    /// Returns true if the slice contains 0 bytes, false otherwise.
    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    /// Returns the element of a slice at a given index, or `None` if the index
    /// is out of bounds.
    pub fn get(&self, index: usize) -> Option<&u8> { self.data.get(index) }

    /// Returns the elements of the slice comprised in the range, or `None` if
    /// the range is out of bounds or ill-formed.
    pub fn slice(&self, range: Range<usize>) -> Option<Slice<'a>> {
        if range.start <= range.end && range.end <= self.len() {
            //  Pre-conditions:
            //  - the range is well-formed (start <= end)
            //  - the beginning of the range is within bounds
            //  - the end of the range is within bounds
            //  Post-conditions:
            //  - the subslice created has the same lifetime as the original
            Some(Slice { data: unsafe {
                from_raw_parts(
                    self.data.as_ptr().offset(range.start as isize),
                    range.end - range.start
                )
            } })
        } else {
            None
        }
    }
}

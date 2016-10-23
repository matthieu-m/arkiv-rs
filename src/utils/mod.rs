//! # Utility module

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

    /// Returns the underlying slice
    pub fn raw(&self) -> &'a [u8] { self.data }

    /// Returns the number of bytes in the slice.
    pub fn len(&self) -> usize { self.data.len() }

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

/// Returns a u16 interpreting the first 2 bytes of the slice in little-endian
/// encoding, or `None` if the slice is too short.
///
/// Does not assume that the data is suitably aligned.
pub fn read_u16_le<'a>(slice: Slice<'a>) -> Option<u16> {
    match (slice.get(0), slice.get(1)) {
        (Some(&b0), Some(&b1)) => {
            Some((b0 as u16) + ((b1 as u16) << 8))
        },
        _ => None
    }
}

/// Returns a u32 interpreting the first 4 bytes of the slice in little-endian
/// encoding, or `None` if the slice is too short.
///
/// Does not assume that the data is suitably aligned.
pub fn read_u32_le<'a>(slice: Slice<'a>) -> Option<u32> {
    match (slice.get(0), slice.get(1), slice.get(2), slice.get(3)) {
        (Some(&b0), Some(&b1), Some(&b2), Some(&b3)) => {
            Some(((b0 as u32) <<  0) +
                 ((b1 as u32) <<  8) +
                 ((b2 as u32) << 16) +
                 ((b3 as u32) << 24))
        },
        _ => None
    }
}
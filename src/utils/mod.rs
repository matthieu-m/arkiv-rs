//! # Utility module

#[cfg(test)]
pub mod test;

use std::ops::Range;
use std::slice::from_raw_parts;

/// Constant used when unwrapping an empty `Option<u16>`
pub const DEAD: u16 = 0xdead;

/// Constant used when unwrapping an empty `Option<u32>`
pub const DEADBEEF: u32 = 0xdeadbeef;

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

    /// Returns the position of the start of this slice in the `hay` slice,
    /// or None if this slice does not start within the `hay` slice.
    ///
    /// Note that this slice may not entirely fit within the `hay` slice.
    #[allow(dead_code)]
    pub fn position(&self, hay: Slice<'a>) -> Option<usize> {
        position(self.data, hay.data)
    }

    /// Returns the elements of the slice comprised in the range, or `None` if
    /// the range is out of bounds or ill-formed.
    pub fn slice(&self, range: Range<usize>) -> Option<Slice<'a>> {
        slice(self.data, range).map(Slice::new)
    }

    /// Returns a copy of the original slice, minus the `n` first bytes.
    #[allow(dead_code)]
    pub fn skip(&self, n: usize) -> Slice<'a> {
        Slice::new(skip(self.data, n))
    }

    /// Returns a copy of the slice, minus any byte after the `n`-th.
    pub fn take(&self, n: usize) -> Slice<'a> {
        Slice::new(take(self.data, n))
    }
}

/// Helper trait to read little-endian fields.
pub trait LeFieldReader<'a> {
    fn min_size() -> usize;

    fn get_slice(&self) -> Slice<'a>;

    /// Interprets the 2 bytes as u16 (little-endian).
    fn read_u16(&self, range: Range<usize>) -> u16 {
        debug_assert!(range.len() == 2);
        debug_assert!(range.end <= Self::min_size());

        self.get_slice()
            .slice(range)
            .and_then(read_u16_le)
            .unwrap_or(DEAD)
    }

    /// Interprets the 4 bytes as u32 (little-endian).
    fn read_u32(&self, range: Range<usize>) -> u32 {
        debug_assert!(range.len() == 4);
        debug_assert!(range.end <= Self::min_size());

        self.get_slice()
            .slice(range)
            .and_then(read_u32_le)
            .unwrap_or(DEADBEEF)
    }

    /// Optional binary field of a given length at a given index.
    fn read_field(&self, index: usize, length: usize) -> Option<&'a [u8]> {
        self.get_slice().slice(index..(index + length)).map(|s| s.raw())
    }
}

/// Returns the elements of the `hay` slice that overlap with `range`.
pub fn intersect_slice<'a>(hay: &'a [u8], range: Range<usize>) -> &'a [u8] {
    if range.start <= range.end && range.start <= hay.len() {
        take(skip(hay, range.start), range.end.wrapping_sub(range.start))
    } else {
        b""
    }
}

/// Returns the position of the `needle` slice in the `hay` slice, or None if 
/// the `needle` slice does start within the `hay` slice.
///
/// The `needle` slice may not entirely fit within the `hay` slice.
pub fn position(needle: &[u8], hay: &[u8]) -> Option<usize> {
    let hay_len = hay.len();
    let hay = hay.as_ptr() as usize;
    let needle = needle.as_ptr() as usize;

    if needle >= hay && needle.wrapping_sub(hay) < hay_len {
        Some(needle.wrapping_sub(hay))
    } else {
        None
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

/// Returns the elements of the slice comprised in the range, or `None` if
/// the range is out of bounds or ill-formed.
pub fn slice<'a>(original: &'a [u8], range: Range<usize>) -> Option<&'a [u8]> {
    if range.start <= range.end && range.end <= original.len() {
        Some(
            take(
                skip(original, range.start),
                range.end.wrapping_sub(range.start)
            )
        )
    } else {
        None
    }
}

/// Returns a copy of the original slice, minus the `n` first elements.
pub fn skip<'a>(original: &'a [u8], n: usize) -> &'a [u8] {
    if n == 0 { return original; }

    let mut iter = original.iter();
    iter.nth(n.wrapping_sub(1)); // O(1)
    iter.as_slice()
}

/// Returns a copy of the original slice, minus any element after the `n`-th.
pub fn take<'a>(original: &'a [u8], n: usize) -> &'a [u8] {
    use std::cmp::min;

    // TODO: replace with safe code
    //
    // While the following code optimizes well:
    // ```
    // let take_back = original.len().wrapping_sub(n).wrapping_sub(1);
    // let mut iter = original.iter();
    // iter.rev().nth(take_back); // O(n)
    // iter.as_slice()
    // ```
    // It can unfortunately be VERY slow in debug.
    unsafe { from_raw_parts(original.as_ptr(), min(original.len(), n)) }
}

#[cfg(test)]
mod tests {
    use super::slice;

    #[test]
    fn slice_success_with_zero_range_on_empty_slice() {
        assert_eq!(slice(&[], 0..0), Some(&[][..]));
    }

    #[test]
    fn slice_failure_with_invalid_range_on_empty_slice() {
        for i in 1..100 {
            let begin = i * 7919;
            assert_eq!(slice(&[], begin..(begin - i)), None);
        }
    }

    #[test]
    fn slice_failure_with_non_empty_range_on_empty_slice() {
        for i in 1..100 {
            let begin = i * 7919;
            assert_eq!(slice(&[], begin..(begin + i)), None);
        }
    }
}

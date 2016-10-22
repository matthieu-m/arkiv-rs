//! # Utility module

use super::api::Slice;

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

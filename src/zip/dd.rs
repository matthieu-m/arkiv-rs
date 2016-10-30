//! ## Data Descriptor

use std::ops::Range;

use super::super::utils::{DEADBEEF, Slice, read_u32_le};

/// A Local File Header
///
/// This structure only guarantees that access to its various fields is safe, it
/// does not guarantee their integrity.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DataDescriptorReader<'a> {
    data: Slice<'a>,
}

// +--------------------------------------------------------------------+
// | Offset  | Bytes  |                 Description                     |
// |---------+--------+-------------------------------------------------|
// | 0       | 0/4    | Optional data descriptor signature = 0x08074b50 |
// | 0/4     | 4      | CRC-32                                          |
// | 4/8     | 4      | Compressed size                                 |
// | 8/12    | 4      | Uncompressed size                               |
// +--------------------------------------------------------------------+
impl<'a> DataDescriptorReader<'a> {
    /// Returns the minimum size of the record.
    pub fn min_size() -> usize { 12 }

    /// Returns the maximum size of the record.
    pub fn max_size() -> usize { Self::min_size() + 4 }

    /// Returns the expected signature.
    pub fn expected_signature() -> u32 { 0x08074b50 }

    /// Returns a new instance if the slice is either 12 bytes or 16 bytes,
    /// otherwise returns `None`.
    ///
    /// Note that the signature is not checked, this is so that decoding
    /// potentially corrupted archives is still possible.
    pub fn new(slice: &'a [u8]) -> Option<DataDescriptorReader<'a>> {
        if slice.len() == Self::min_size() || slice.len() == Self::max_size() {
            Some(DataDescriptorReader { data: Slice::new(slice) })
        } else {
            None
        }
    }

    /// Returns the underlying slice.
    pub fn raw(&self) -> &'a [u8] { self.data.raw() }

    /// Returns whether this instance has a signature field, or not.
    pub fn has_signature(&self) -> bool { self.data.len() == Self::max_size() }

    /// Returns the signature.
    pub fn signature(&self) -> Option<u32> {
        if self.has_signature() { Some(self.read_u32(0..4)) } else { None }
    }

    /// Returns the CRC-32 of the file.
    ///
    /// (see CentralDirectoryFileHeaderReader for more ample information)
    pub fn crc32(&self) -> u32 {
        self.read_u32(if self.has_signature() { (4..8) } else { (0..4) })
    }

    /// Returns the compressed size of the file.
    pub fn compressed_size(&self) -> u32 {
        self.read_u32(if self.has_signature() { (8..12) } else { (4..8) })
    }

    /// Returns the uncompressed size of the file.
    pub fn uncompressed_size(&self) -> u32 {
        self.read_u32(if self.has_signature() { (12..16) } else { (8..12) })
    }

    /// Interprets the 4 bytes as u32 (little-endian).
    fn read_u32(&self, range: Range<usize>) -> u32 {
        debug_assert!(range.len() == 4);
        debug_assert!(range.end <= self.data.len());

        self.data
            .slice(range)
            .and_then(read_u32_le)
            .unwrap_or(DEADBEEF)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::utils::test::{test_some_u32_at};

    type Reader<'a> = super::DataDescriptorReader<'a>;

    #[test]
    fn reader_new_failure_on_inexact_slice() {
        let v = vec!(0; 65535);
        for length in 0..v.len() {
            if length != Reader::min_size() && length != Reader::max_size() {
                assert_eq!(Reader::new(&v[0..length]), None);
            }
        }
    }

    #[test]
    fn reader_new_success_on_12_bytes() {
        let v = vec!(0; Reader::min_size());
        assert!(Reader::new(&v).is_some());
        assert_eq!(Reader::new(&v).unwrap().has_signature(), false);
    }

    #[test]
    fn reader_new_success_on_16_bytes() {
        let v = vec!(0; Reader::max_size());
        assert!(Reader::new(&v).is_some());
        assert_eq!(Reader::new(&v).unwrap().has_signature(), true);
    }

    #[test]
    fn reader_signature_failure_on_12_bytes_with_expected_signature() {
        let mut v = vec!(0; Reader::min_size());
        v[3] = 0x08;
        v[2] = 0x07;
        v[1] = 0x4b;
        v[0] = 0x50;
        let dd = Reader::new(&v).unwrap();
        assert_eq!(dd.has_signature(), false);
        assert_eq!(dd.signature(), None);
    }

    #[test]
    fn reader_signature_failure_on_12_bytes_with_unexpected_signature() {
        let v = vec!(0; Reader::min_size());
        let dd = Reader::new(&v).unwrap();
        assert_eq!(dd.has_signature(), false);
        assert_eq!(dd.signature(), None);
    }

    #[test]
    fn reader_signature_success_on_16_bytes_with_expected_signature() {
        let mut v = vec!(0; Reader::max_size());
        v[3] = 0x08;
        v[2] = 0x07;
        v[1] = 0x4b;
        v[0] = 0x50;
        let dd = Reader::new(&v).unwrap();
        assert_eq!(dd.signature(), Some(Reader::expected_signature()));
    }

    #[test]
    fn reader_signature_success_on_16_bytes_with_unexpected_signature() {
        let v = vec!(0; Reader::max_size());
        let dd = Reader::new(&v).unwrap();
        assert_eq!(dd.signature(), Some(0));
    }

    #[test]
    fn reader_crc32_success_without_signature() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 0, |v, version| {
            let dd = Reader::new(v).unwrap();
            assert_eq!(dd.crc32(), version);
        });
    }

    #[test]
    fn reader_crc32_success_with_signature() {
        let mut v = vec!(0; Reader::max_size());
        test_some_u32_at(&mut v, 4, |v, version| {
            let dd = Reader::new(v).unwrap();
            assert_eq!(dd.crc32(), version);
        });
    }

    #[test]
    fn reader_compressed_size_success_without_signature() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 4, |v, version| {
            let dd = Reader::new(v).unwrap();
            assert_eq!(dd.compressed_size(), version);
        });
    }

    #[test]
    fn reader_compressed_size_success_with_signature() {
        let mut v = vec!(0; Reader::max_size());
        test_some_u32_at(&mut v, 8, |v, version| {
            let dd = Reader::new(v).unwrap();
            assert_eq!(dd.compressed_size(), version);
        });
    }

    #[test]
    fn reader_uncompressed_size_success_without_signature() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 8, |v, version| {
            let dd = Reader::new(v).unwrap();
            assert_eq!(dd.uncompressed_size(), version);
        });
    }

    #[test]
    fn reader_uncompressed_size_success_with_signature() {
        let mut v = vec!(0; Reader::max_size());
        test_some_u32_at(&mut v, 12, |v, version| {
            let dd = Reader::new(v).unwrap();
            assert_eq!(dd.uncompressed_size(), version);
        });
    }
}

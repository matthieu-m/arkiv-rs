//! # End of Central Directory

use super::super::utils::{Slice, LeFieldReader};

/// A End of Central Directory
///
/// This structure only guarantees that access to its various fields is safe, it
/// does not guarantee their integrity.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EndOfCentralDirectoryReader<'a> {
    data: Slice<'a>,
}

// +---------------------------------------------------------------------+
// | Offset  | Bytes  |                          Description             |
// |---------+--------+--------------------------------------------------|
// |      0  | 4      | End of central directory signature = 0x06054b50  |
// |      4  | 2      | Number of this disk                              |
// |      6  | 2      | Disk where central directory starts              |
// |      8  | 2      | Number of central directory records on this disk |
// |     10  | 2      | Total number of central directory records        |
// |     12  | 4      | Size of central directory (bytes)                |
// |     16  | 4      | Offset of start of central directory             |
// |     20  | 2      | Comment length (n)                               |
// |     22  | n      | Comment                                          |
// +---------------------------------------------------------------------+
impl<'a> EndOfCentralDirectoryReader<'a> {
    /// Returns the minimum size of the record.
    pub fn min_size() -> usize { 22 }

    /// Returns the maximum size of the record.
    pub fn max_size() -> usize { Self::min_size() + 65535 }

    /// Returns the expected signature.
    pub fn expected_signature() -> u32 { 0x06054b50 }

    /// Returns a new instance if the slice is sufficiently large (22 bytes),
    /// otherwise returns `None`.
    ///
    /// Note that neither the signature nor the size of comment field are
    /// checked, this is so that decoding potentially corrupted archives is
    /// still possible.
    pub fn new(slice: &'a [u8]) -> Option<EndOfCentralDirectoryReader<'a>> {
        if slice.len() >= Self::min_size() {
            Some(EndOfCentralDirectoryReader {
                data: Slice::new(slice).take(Self::max_size())
            })
        } else {
            None
        }
    }

    /// Returns the underlying slice.
    pub fn raw(&self) -> &'a [u8] { self.data.raw() }

    /// Returns the signature.
    pub fn signature(&self) -> u32 { self.read_u32(0..4) }

    /// Returns the number of the disk.
    pub fn disk(&self) -> u16 { self.read_u16(4..6) }

    /// Returns the number of the disk where the central directory starts.
    pub fn central_directory_disk(&self) -> u16 { self.read_u16(6..8) }

    /// Returns the number of central directory records on this disk.
    pub fn nb_local_central_directory_records(&self) -> u16 {
        self.read_u16(8..10)
    }

    /// Returns the number of central directory records on all disks.
    pub fn nb_central_directory_records(&self) -> u16 { self.read_u16(10..12) }

    /// Returns the size of the central directory (in bytes).
    pub fn central_directory_size(&self) -> u32 { self.read_u32(12..16) }

    /// Returns the offset of the central directory, from start of archive.
    pub fn central_directory_offset(&self) -> u32 { self.read_u32(16..20) }

    /// Returns the comment field size.
    pub fn comment_size(&self) -> u16 { self.read_u16(20..22) }

    /// Returns the comment field, possibly of length 0, or `None` if the slice
    /// is truncated.
    pub fn comment(&self) -> Option<&'a [u8]> {
        let min = Self::min_size();
        let range = min..(min + self.comment_size() as usize);
        self.data.slice(range).map(|s| s.raw())
    }
}

impl<'a> LeFieldReader<'a> for EndOfCentralDirectoryReader<'a> {
    fn min_size() -> usize { EndOfCentralDirectoryReader::min_size() }

    fn get_slice(&self) -> Slice<'a> { self.data }
}

#[cfg(test)]
mod tests {
    use super::super::super::utils::test::{test_all_u16_at, test_some_u32_at};

    type Reader<'a> = super::EndOfCentralDirectoryReader<'a>;

    #[test]
    fn reader_new_failure_on_short_slice() {
        let v = vec!(0; Reader::min_size() - 1);
        for length in 0..v.len() {
            assert_eq!(Reader::new(&v[0..length]), None);
        }
    }

    #[test]
    fn reader_new_success_on_22_bytes_slice() {
        let v = vec!(0; Reader::min_size());
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_22_bytes_slice_with_nonzero_comment_length() {
        let mut v = vec!(0; Reader::min_size());
        v[20] = 0x01;   // 1 byte comment
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_signature_success_with_expected_signature() {
        let mut v = vec!(0; Reader::min_size());
        v[3] = 0x06;
        v[2] = 0x05;
        v[1] = 0x4b;
        v[0] = 0x50;
        let eocd = Reader::new(&v).unwrap();
        assert_eq!(
            eocd.signature(),
            Reader::expected_signature()
        );
    }

    #[test]
    fn reader_signature_success_with_unexpected_signature() {
        let v = vec!(0; Reader::min_size());
        let eocd = Reader::new(&v).unwrap();
        assert_eq!(eocd.signature(), 0);
    }

    #[test]
    fn reader_disk_number_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 4, |v, disk| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.disk(), disk);
        });
    }

    #[test]
    fn reader_central_directory_disk_number_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 6, |v, disk| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.central_directory_disk(), disk);
        });
    }

    #[test]
    fn reader_nb_local_records_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 8, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.nb_local_central_directory_records(), nb);
        });
    }

    #[test]
    fn reader_nb_records_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 10, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.nb_central_directory_records(), nb);
        });
    }

    #[test]
    fn reader_central_directory_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 12, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.central_directory_size(), nb);
        });
    }

    #[test]
    fn reader_central_directory_offset_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 16, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.central_directory_offset(), nb);
        });
    }

    #[test]
    fn reader_comment_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 20, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.comment_size(), nb);
        });
    }

    #[test]
    fn reader_comment_success_with_zero_length() {
        let v = vec!(0; Reader::min_size());
        let eocd = Reader::new(&v).unwrap();

        assert_eq!(eocd.comment(), Some(&b""[..]));
    }

    #[test]
    fn reader_comment_success_with_hello_world() {
        let mut v = vec!(0; Reader::min_size());
        v.extend_from_slice(b"Hello, World!");
        v[20] = (v.len() - Reader::min_size()) as u8;

        let eocd = Reader::new(&v).unwrap();

        assert_eq!(eocd.comment(), Some(&b"Hello, World!"[..]));
    }

    #[test]
    fn reader_comment_success_with_max_length() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::max_size(), 1);

        v[21] = 0xff;
        v[20] = 0xff;

        let eocd = Reader::new(&v).unwrap();

        assert_eq!(eocd.comment(), Some(&v[Reader::min_size()..]));
    }

    #[test]
    fn reader_comment_failure_on_too_short_buffer() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::max_size() - 1, 1);

        for length in 1..(Reader::max_size() - Reader::min_size() + 1) {
            v[21] = (length >> 8) as u8;
            v[20] = (length >> 0) as u8;

            let slice = &v[0..(Reader::min_size() + length - 1)];

            let eocd = Reader::new(slice).unwrap();

            assert_eq!(eocd.comment(), None);
        }
    }
}

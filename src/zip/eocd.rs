//! # End of Central Directory

use std::ops::Range;

use super::super::utils::{Slice, read_u16_le, read_u32_le};

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
    /// Returns a new instance if the slice is sufficiently large (22 bytes),
    /// otherwise returns `None`.
    ///
    /// Note that neither the signature nor the size of comment field are
    /// checked, this is so that decoding potentially corrupted archives is
    /// still possible.
    pub fn new(slice: &'a [u8]) -> Option<EndOfCentralDirectoryReader<'a>> {
        if slice.len() >= 22 {
            Some(EndOfCentralDirectoryReader { data: Slice::new(slice) })
        } else {
            None
        }
    }

    /// Returns the expected signature
    pub fn expected_signature() -> u32 { 0x06054b50 }

    /// Returns the signature.
    pub fn signature(&self) -> u32 { self.read_u32(0..4) }

    /// Returns the number of the disk
    pub fn disk(&self) -> u16 { self.read_u16(4..6) }

    /// Returns the number of the disk where the central directory starts
    pub fn central_directory_disk(&self) -> u16 { self.read_u16(6..8) }

    /// Returns the number of central directory records on this disk
    pub fn nb_local_central_directory_records(&self) -> u16 {
        self.read_u16(8..10)
    }

    /// Returns the number of central directory records on all disks
    pub fn nb_central_directory_records(&self) -> u16 { self.read_u16(10..12) }

    /// Returns the size of the central directory (in bytes)
    pub fn central_directory_size(&self) -> u32 { self.read_u32(12..16) }

    /// Returns the offset of the central directory, from start of archive.
    pub fn central_directory_offset(&self) -> u32 { self.read_u32(16..20) }

    /// Returns the comment field size
    pub fn comment_size(&self) -> u16 { self.read_u16(20..22) }

    /// Returns the comment field, possibly of length 0, or `None` if the slice
    /// is truncated.
    pub fn comment(&self) -> Option<&'a [u8]> {
        let range = 22..(22 + self.comment_size() as usize);
        self.data.slice(range).map(|s| s.raw())
    }

    /// Interpret the 2 bytes
    fn read_u16(&self, range: Range<usize>) -> u16 {
        debug_assert!(range.len() == 2);
        debug_assert!(range.end <= self.data.len());

        read_u16_le(
            self.data.slice(range).expect("Length >= 22")
        ).expect("Length == 2")
    }

    /// Interpret the 4 bytes
    fn read_u32(&self, range: Range<usize>) -> u32 {
        debug_assert!(range.len() == 4);
        debug_assert!(range.end <= self.data.len());

        read_u32_le(
            self.data.slice(range).expect("Length >= 22")
        ).expect("Length == 4")
    }
}

#[cfg(test)]
mod tests {
    use std;

    type Reader<'a> = super::EndOfCentralDirectoryReader<'a>;

    #[test]
    fn reader_new_failure_on_short_slice() {
        let v = vec!(0; 21);
        for length in 0..21 {
            assert_eq!(Reader::new(&v[0..(length+1)]), None);
        }
    }

    #[test]
    fn reader_new_success_on_22_bytes_slice() {
        let v = vec!(0; 22);
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_22_bytes_slice_with_nonzero_comment_length() {
        let mut v = vec!(0; 22);
        v[20] = 0x01;   // 1 byte comment
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_signature_success_with_expected_signature() {
        let mut v = vec!(0; 22);
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
        let v = vec!(0; 22);
        let eocd = Reader::new(&v).unwrap();
        assert_eq!(eocd.signature(), 0);
    }

    #[test]
    fn reader_disk_number_success() {
        let mut v = vec!(0; 22);
        test_all_u16_at(&mut v, 4, |v, disk| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.disk(), disk);
        });
    }

    #[test]
    fn reader_central_directory_disk_number_success() {
        let mut v = vec!(0; 22);
        test_all_u16_at(&mut v, 6, |v, disk| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.central_directory_disk(), disk);
        });
    }

    #[test]
    fn reader_nb_local_records_success() {
        let mut v = vec!(0; 22);
        test_all_u16_at(&mut v, 8, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.nb_local_central_directory_records(), nb);
        });
    }

    #[test]
    fn reader_nb_records_success() {
        let mut v = vec!(0; 22);
        test_all_u16_at(&mut v, 10, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.nb_central_directory_records(), nb);
        });
    }

    #[test]
    fn reader_central_directory_size_success() {
        let mut v = vec!(0; 22);
        test_some_u32_at(&mut v, 12, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.central_directory_size(), nb);
        });
    }

    #[test]
    fn reader_central_directory_offset_success() {
        let mut v = vec!(0; 22);
        test_some_u32_at(&mut v, 16, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.central_directory_offset(), nb);
        });
    }

    #[test]
    fn reader_comment_size_success() {
        let mut v = vec!(0; 22);
        test_all_u16_at(&mut v, 20, |v, nb| {
            let eocd = Reader::new(v).unwrap();
            assert_eq!(eocd.comment_size(), nb);
        });
    }

    #[test]
    fn reader_comment_success_with_zero_length() {
        let v = vec!(0; 22);
        let eocd = Reader::new(&v).unwrap();

        assert_eq!(eocd.comment(), Some(&b""[..]));
    }

    #[test]
    fn reader_comment_success_with_hello_world() {
        let mut v = vec!(0; 22);
        v.extend_from_slice(b"Hello, World!");
        v[20] = (v.len() - 22) as u8;

        let eocd = Reader::new(&v).unwrap();

        assert_eq!(eocd.comment(), Some(&b"Hello, World!"[..]));
    }

    #[test]
    fn reader_comment_success_with_max_length() {
        let mut v = vec!(0; 22);
        v.resize(22 + 65535, 1);

        v[21] = 0xff;
        v[20] = 0xff;

        let eocd = Reader::new(&v).unwrap();

        assert_eq!(eocd.comment(), Some(&v[22..]));
    }

    #[test]
    fn reader_comment_failure_on_too_short_buffer() {
        let mut v = vec!(0; 22);
        v.resize(22 + 65534, 1);

        for length in 1..65536u32 {
            v[21] = (length >> 8) as u8;
            v[20] = (length >> 0) as u8;

            let slice = &v[0..(22 + (length as usize) - 1)];

            let eocd = Reader::new(slice).unwrap();

            assert_eq!(eocd.comment(), None);
        }
    }

    fn test_all_u16_at<F>(buffer: &mut [u8], index: usize, f: F)
        where F: Fn(&[u8], u16) -> ()
    {
        for data in 0..65536u32 {
            let data = data as u16;
            buffer[index + 1] = (data >> 8) as u8;
            buffer[index + 0] = (data >> 0) as u8;

            f(buffer, data)
        }
    }

    fn test_some_u32_at<F>(buffer: &mut [u8], index: usize, f: F)
        where F: Fn(&[u8], u32) -> ()
    {
        fn test<F>(buffer: &mut [u8], data: u32, index: usize, f: &F)
            where F: Fn(&[u8], u32) -> ()
        {
            buffer[index + 3] = (data >> 24) as u8;
            buffer[index + 2] = (data >> 16) as u8;
            buffer[index + 1] = (data >>  8) as u8;
            buffer[index + 0] = (data >>  0) as u8;

            f(buffer, data)
        }

        for data in 0..65536 {
            test(buffer, data, index, &f);
        }

        for data in 0..65536 {
            test(buffer, data * 251, index, &f);
        }

        for data in 0..65536 {
            test(buffer, std::u32::MAX - data * 251, index, &f);
        }

        for data in 0..65536 {
            test(buffer, std::u32::MAX - data, index, &f);
        }
    }
}

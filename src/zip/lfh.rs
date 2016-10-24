//! ## Local File Header

use super::super::utils::{Slice, LeFieldReader};

/// A Local File Header
///
/// This structure only guarantees that access to its various fields is safe, it
/// does not guarantee their integrity.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LocalFileHeaderReader<'a> {
    data: Slice<'a>,
}

// +-------------------------------------------------------------+
// | Offset  | Bytes  |             Description                  |
// |---------+--------+------------------------------------------|
// | 0       | 4      | Local file header signature = 0x04034b50 |
// | 4       | 2      | Version needed to extract (minimum)      |
// | 6       | 2      | General purpose bit flag                 |
// | 8       | 2      | Compression method                       |
// | 10      | 2      | File last modification time              |
// | 12      | 2      | File last modification date              |
// | 14      | 4      | CRC-32                                   |
// | 18      | 4      | Compressed size                          |
// | 22      | 4      | Uncompressed size                        |
// | 26      | 2      | File name length (n)                     |
// | 28      | 2      | Extra field length (m)                   |
// | 30      | n      | File name                                |
// | 30+n    | m      | Extra field                              |
// +-------------------------------------------------------------+
impl<'a> LocalFileHeaderReader<'a> {
    /// Returns the minimum size of the record.
    pub fn min_size() -> usize { 30 }

    /// Returns the maximum size of the record.
    pub fn max_size() -> usize { Self::min_size() + 65535 * 2 }

    /// Returns the expected signature.
    pub fn expected_signature() -> u32 { 0x04034b50 }

    /// Returns a new instance if the slice is sufficiently large (30 bytes),
    /// otherwise returns `None`.
    ///
    /// Note that neither the signature nor the size of the dynamic fields are
    /// checked, this is so that decoding potentially corrupted archives is
    /// still possible.
    pub fn new(slice: &'a [u8]) -> Option<LocalFileHeaderReader<'a>> {
        if slice.len() >= Self::min_size() {
            Some(LocalFileHeaderReader { data: Slice::new(slice) })
        } else {
            None
        }
    }

    /// Returns the signature.
    pub fn signature(&self) -> u32 { self.read_u32(0..4) }

    /// Returns the minimum ZIP version needed to extract.
    ///
    /// (see CentralDirectoryFileHeaderReader for more ample information)
    pub fn version_needed_to_extract(&self) -> u16 { self.read_u16(4..6) }

    /// Returns the general purpose bit flags.
    ///
    /// (see CentralDirectoryFileHeaderReader for more ample information)
    pub fn general_purpose_bit_flag(&self) -> u16 { self.read_u16(6..8) }

    /// Returns the compression method used to compressed the file.
    ///
    /// (see CentralDirectoryFileHeaderReader for more ample information)
    pub fn compression_method(&self) -> u16 { self.read_u16(8..10) }

    /// Returns the last modification time of the file, MS-DOS format.
    pub fn file_last_modification_time(&self) -> u16 { self.read_u16(10..12) }

    /// Returns the last modification date of the file, MS-DOS format.
    pub fn file_last_modification_date(&self) -> u16 { self.read_u16(12..14) }

    /// Returns the CRC-32 of the file.
    ///
    /// (see CentralDirectoryFileHeaderReader for more ample information)
    pub fn crc32(&self) -> u32 { self.read_u32(14..18) }

    /// Returns the compressed size of the file.
    pub fn compressed_size(&self) -> u32 { self.read_u32(18..22) }

    /// Returns the uncompressed size of the file.
    pub fn uncompressed_size(&self) -> u32 { self.read_u32(22..26) }

    /// Returns the size of the file name.
    pub fn file_name_size(&self) -> u16 { self.read_u16(26..28) }

    /// Returns the size of the extra field.
    pub fn extra_field_size(&self) -> u16 { self.read_u16(28..30) }

    /// Returns the file name, possibly of length 0, or `None` if the slice is
    /// truncated.
    pub fn file_name(&self) -> Option<&'a [u8]> {
        self.read_field(Self::min_size(), self.file_name_size() as usize)
    }

    /// Returns the extra field, possibly of length 0, or `None` if the slice is
    /// truncated.
    pub fn extra_field(&self) -> Option<&'a [u8]> {
        self.read_field(
            self.extra_field_position(),
            self.extra_field_size() as usize
        )
    }

    /// Returns the position of the extra field.
    fn extra_field_position(&self) -> usize {
        Self::min_size() +
        self.file_name_size() as usize
    }
}

impl<'a> LeFieldReader<'a> for LocalFileHeaderReader<'a> {
    fn min_size() -> usize { LocalFileHeaderReader::min_size() }

    fn get_slice(&self) -> Slice<'a> { self.data }
}

#[cfg(test)]
mod tests {
    use std;

    use super::super::super::utils::test::{test_all_u16_at, test_some_u32_at};

    type Reader<'a> = super::LocalFileHeaderReader<'a>;

    #[test]
    fn reader_new_failure_on_short_slice() {
        let v = vec!(0; Reader::min_size() - 1);
        for length in 0..v.len() {
            assert_eq!(Reader::new(&v[0..length]), None);
        }
    }

    #[test]
    fn reader_new_success_on_30_bytes_slice() {
        let v = vec!(0; Reader::min_size());
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_30_bytes_slice_with_nonzero_file_name_length() {
        let mut v = vec!(0; Reader::min_size());
        v[26] = 0x01;   // 1 byte file name
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_30_bytes_slice_with_nonzero_extra_field_length() {
        let mut v = vec!(0; Reader::min_size());
        v[28] = 0x01;   // 1 byte extra field
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_30_bytes_slice_with_all_nonzero_variables() {
        let mut v = vec!(0; Reader::min_size());
        v[26] = 0x01;   // 1 byte file name
        v[28] = 0x01;   // 1 byte extra field
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_signature_success_with_expected_signature() {
        let mut v = vec!(0; Reader::min_size());
        v[3] = 0x04;
        v[2] = 0x03;
        v[1] = 0x4b;
        v[0] = 0x50;
        let lfh = Reader::new(&v).unwrap();
        assert_eq!(
            lfh.signature(),
            Reader::expected_signature()
        );
    }

    #[test]
    fn reader_signature_success_with_unexpected_signature() {
        let v = vec!(0; Reader::min_size());
        let lfh = Reader::new(&v).unwrap();
        assert_eq!(lfh.signature(), 0);
    }

    #[test]
    fn reader_version_needed_to_extract_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 4, |v, version| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.version_needed_to_extract(), version);
        });
    }

    #[test]
    fn reader_general_purpose_bit_flag_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 6, |v, f| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.general_purpose_bit_flag(), f);
        });
    }

    #[test]
    fn reader_compression_method_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 8, |v, c| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.compression_method(), c);
        });
    }

    #[test]
    fn reader_file_last_modification_time_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 10, |v, t| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.file_last_modification_time(), t);
        });
    }

    #[test]
    fn reader_file_last_modification_date_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 12, |v, d| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.file_last_modification_date(), d);
        });
    }

    #[test]
    fn reader_crc32_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 14, |v, crc| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.crc32(), crc);
        });
    }

    #[test]
    fn reader_compressed_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 18, |v, size| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.compressed_size(), size);
        });
    }

    #[test]
    fn reader_uncompressed_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 22, |v, size| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.uncompressed_size(), size);
        });
    }

    #[test]
    fn reader_file_name_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 26, |v, size| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.file_name_size(), size);
        });
    }

    #[test]
    fn reader_extra_field_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 28, |v, size| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(lfh.extra_field_size(), size);
        });
    }

    #[test]
    fn reader_extra_field_position_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 26, |v, size| {
            let lfh = Reader::new(v).unwrap();
            assert_eq!(
                lfh.extra_field_position(),
                Reader::min_size() + size as usize
            );
        });
    }

    #[test]
    fn reader_file_name_success_with_zero_length() {
        let v = vec!(0; Reader::min_size());
        let lfh = Reader::new(&v).unwrap();

        assert_eq!(lfh.file_name(), Some(&b""[..]));
    }

    #[test]
    fn reader_extra_field_success_with_zero_length() {
        let v = vec!(0; Reader::min_size());
        let lfh = Reader::new(&v).unwrap();

        assert_eq!(lfh.extra_field(), Some(&b""[..]));
    }

    #[test]
    fn reader_file_name_success_with_hello_world() {
        let mut v = vec!(0; Reader::min_size());
        v.extend_from_slice(b"Hello, World!");
        v[26] = (v.len() - Reader::min_size()) as u8;

        let lfh = Reader::new(&v).unwrap();

        assert_eq!(lfh.file_name(), Some(&b"Hello, World!"[..]));
    }

    #[test]
    fn reader_extra_field_success_with_hello_world() {
        let mut v = vec!(0; Reader::min_size());
        v.extend_from_slice(b"Hello, World!");
        v[28] = (v.len() - Reader::min_size()) as u8;

        let lfh = Reader::new(&v).unwrap();

        assert_eq!(lfh.extra_field(), Some(&b"Hello, World!"[..]));
    }

    #[test]
    fn reader_file_name_success_with_max_length() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize, 1);

        v[27] = 0xff;
        v[26] = 0xff;

        let lfh = Reader::new(&v).unwrap();

        assert_eq!(lfh.file_name(), Some(&v[Reader::min_size()..]));
    }

    #[test]
    fn reader_extra_field_success_with_max_length() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize, 1);

        v[29] = 0xff;
        v[28] = 0xff;

        let lfh = Reader::new(&v).unwrap();

        assert_eq!(lfh.extra_field(), Some(&v[Reader::min_size()..]));
    }

    #[test]
    fn reader_file_name_failure_on_too_short_buffer() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize - 1, 1);

        for length in 1..(std::u16::MAX as usize + 1) {
            v[27] = (length >> 8) as u8;
            v[26] = (length >> 0) as u8;

            let slice = &v[0..(Reader::min_size() + length - 1)];

            let lfh = Reader::new(slice).unwrap();

            assert_eq!(lfh.file_name(), None);
        }
    }

    #[test]
    fn reader_extra_field_failure_on_too_short_buffer() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize - 1, 1);

        for length in 1..(std::u16::MAX as usize + 1) {
            v[29] = (length >> 8) as u8;
            v[28] = (length >> 0) as u8;

            let slice = &v[0..(Reader::min_size() + length - 1)];

            let lfh = Reader::new(slice).unwrap();

            assert_eq!(lfh.extra_field(), None);
        }
    }

    #[test]
    fn reader_all_variables_success_with_hello_world() {
        let hello_file = &b"Hello, File!"[..];
        let hello_extra = &b"Hello, Extra!"[..];

        let mut v = vec!(0; Reader::min_size());
        v.extend_from_slice(hello_file);
        v.extend_from_slice(hello_extra);

        v[26] = hello_file.len() as u8;
        v[28] = hello_extra.len() as u8;

        let lfh = Reader::new(&v).unwrap();

        assert_eq!(lfh.file_name(), Some(hello_file));
        assert_eq!(lfh.extra_field(), Some(hello_extra));
    }
}

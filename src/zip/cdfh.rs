//! # Central Directory File Header

use super::super::utils::{Slice, LeFieldReader};

/// A Central Directory File Header
///
/// This structure only guarantees that access to its various fields is safe, it
/// does not guarantee their integrity.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CentralDirectoryFileHeaderReader<'a> {
    data: Slice<'a>,
}

// +-------------------------------------------------------------------------+
// | Offset  | Bytes  |                   Description                        |
// |---------+--------+------------------------------------------------------|
// | 0       | 4      | Central directory file header signature = 0x02014b50 |
// | 4       | 2      | Version made by                                      |
// | 6       | 2      | Version needed to extract (minimum)                  |
// | 8       | 2      | General purpose bit flag                             |
// | 10      | 2      | Compression method                                   |
// | 12      | 2      | File last modification time                          |
// | 14      | 2      | File last modification date                          |
// | 16      | 4      | CRC-32                                               |
// | 20      | 4      | Compressed size                                      |
// | 24      | 4      | Uncompressed size                                    |
// | 28      | 2      | File name length (n)                                 |
// | 30      | 2      | Extra field length (m)                               |
// | 32      | 2      | File comment length (k)                              |
// | 34      | 2      | Disk number where file starts                        |
// | 36      | 2      | Internal file attributes                             |
// | 38      | 4      | External file attributes                             |
// | 42      | 4      | Relative offset of local file header [1].            |
// | 46      | n      | File name                                            |
// | 46+n    | m      | Extra field                                          |
// | 46+n+m  | k      | File comment                                         |
// +-------------------------------------------------------------------------+
impl<'a> CentralDirectoryFileHeaderReader<'a> {
    /// Returns the minimum size of the record.
    pub fn min_size() -> usize { 46 }

    /// Returns the maximum size of the record.
    pub fn max_size() -> usize { Self::min_size() + 65535 * 3 }

    /// Returns the expected signature.
    pub fn expected_signature() -> u32 { 0x02014b50 }

    /// Returns a new instance if the slice is sufficiently large (46 bytes),
    /// otherwise returns `None`.
    ///
    /// Note that neither the signature nor the size of the dynamic fields are
    /// checked, this is so that decoding potentially corrupted archives is
    /// still possible.
    pub fn new(slice: &'a [u8]) -> Option<CentralDirectoryFileHeaderReader<'a>> {
        if slice.len() >= Self::min_size() {
            Some(CentralDirectoryFileHeaderReader { data: Slice::new(slice) })
        } else {
            None
        }
    }

    /// Returns the signature.
    pub fn signature(&self) -> u32 { self.read_u32(0..4) }

    /// Returns the version of the software that created the record, and its OS.
    ///
    /// The upper byte encodes the host, with 0 for FAT, 3 for UNIX, 10 for NTFS
    /// and 19 for OS X.
    /// The lower byte encode the ZIP specification version as:
    /// `major * 10 + minor`.
    pub fn version_made_by(&self) -> u16 { self.read_u16(4..6) }

    /// Returns the minimum ZIP version needed to extract.
    ///
    /// The ZIP specification version is encoded as `major * 10 + minor`. The
    /// following features are mapped as:
    ///
    /// - 1.0 - Default value
    /// - 1.1 - File is a volume label
    /// - 2.0 - File is a folder (directory)
    /// - 2.0 - File is compressed using Deflate compression
    /// - 2.0 - File is encrypted using traditional PKWARE encryption
    /// - 2.1 - File is compressed using Deflate64(tm)
    /// - 2.5 - File is compressed using PKWARE DCL Implode 
    /// - 2.7 - File is a patch data set 
    /// - 4.5 - File uses ZIP64 format extensions
    /// - 4.6 - File is compressed using BZIP2 compression*
    /// - 5.0 - File is encrypted using DES
    /// - 5.0 - File is encrypted using 3DES
    /// - 5.0 - File is encrypted using original RC2 encryption
    /// - 5.0 - File is encrypted using RC4 encryption
    /// - 5.1 - File is encrypted using AES encryption
    /// - 5.1 - File is encrypted using corrected RC2 encryption**
    /// - 5.2 - File is encrypted using corrected RC2-64 encryption**
    /// - 6.1 - File is encrypted using non-OAEP key wrapping***
    /// - 6.2 - Central directory encryption
    /// - 6.3 - File is compressed using LZMA
    /// - 6.3 - File is compressed using PPMd+
    /// - 6.3 - File is encrypted using Blowfish
    /// - 6.3 - File is encrypted using Twofish
    ///
    pub fn version_needed_to_extract(&self) -> u16 { self.read_u16(6..8) }

    /// Returns the general purpose bit flags.
    ///
    /// Of particular interest, 0x08 is used to indicate the presence of a
    /// Data Descriptor record for this file.
    pub fn general_purpose_bit_flag(&self) -> u16 { self.read_u16(8..10) }

    /// Returns the compression method used to compressed the file.
    ///
    /// Of particular intest:
    /// -  0: No compression
    /// -  8: Deflate
    /// - 14: LZMA
    pub fn compression_method(&self) -> u16 { self.read_u16(10..12) }

    /// Returns the last modification time of the file, MS-DOS format.
    pub fn file_last_modification_time(&self) -> u16 { self.read_u16(12..14) }

    /// Returns the last modification date of the file, MS-DOS format.
    pub fn file_last_modification_date(&self) -> u16 { self.read_u16(14..16) }

    /// Returns the CRC-32 of the file.
    ///
    /// The magic number used is 0xdebb20e3, the register is pre-conditioned
    /// with 0xffffffff and the value is post-conditioned by taking the
    /// one-complement of the CRC residual.
    pub fn crc32(&self) -> u32 { self.read_u32(16..20) }

    /// Returns the compressed size of the file.
    pub fn compressed_size(&self) -> u32 { self.read_u32(20..24) }

    /// Returns the uncompressed size of the file.
    pub fn uncompressed_size(&self) -> u32 { self.read_u32(24..28) }

    /// Returns the size of the file name.
    pub fn file_name_size(&self) -> u16 { self.read_u16(28..30) }

    /// Returns the size of the extra field.
    pub fn extra_field_size(&self) -> u16 { self.read_u16(30..32) }

    /// Returns the size of the file comment.
    pub fn file_comment_size(&self) -> u16 { self.read_u16(32..34) }

    /// Returns the number of the disk in which the file starts.
    pub fn file_start_disk(&self) -> u16 { self.read_u16(34..36) }

    /// Returns the internal file attributes.
    pub fn interal_file_attributes(&self) -> u16 { self.read_u16(36..38) }

    /// Returns the external file attributes.
    ///
    /// The signification depends on the host (see version_made_by).
    pub fn external_file_attributes(&self) -> u32 { self.read_u32(38..42) }

    /// Returns the relative offset of the Local File Header record.
    pub fn local_file_header_relative_offset(&self) -> u32 {
        self.read_u32(42..46)
    }

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

    /// Returns the file comment, possibly of length 0, or `None` if the slice
    /// is truncated.
    pub fn file_comment(&self) -> Option<&'a [u8]> {
        self.read_field(
            self.file_comment_position(),
            self.file_comment_size() as usize
        )
    }

    /// Returns the position of the extra field.
    fn extra_field_position(&self) -> usize {
        Self::min_size() +
        self.file_name_size() as usize
    }

    /// Returns the position of the file comment field.
    fn file_comment_position(&self) -> usize {
        Self::min_size() +
        self.file_name_size() as usize +
        self.extra_field_size() as usize
    }
}

impl<'a> LeFieldReader<'a> for CentralDirectoryFileHeaderReader<'a> {
    fn min_size() -> usize { CentralDirectoryFileHeaderReader::min_size() }

    fn get_slice(&self) -> Slice<'a> { self.data }
}

#[cfg(test)]
mod tests {
    use std;

    use super::super::super::utils::tests::{test_all_u16_at, test_some_u32_at};

    type Reader<'a> = super::CentralDirectoryFileHeaderReader<'a>;

    #[test]
    fn reader_new_failure_on_short_slice() {
        let v = vec!(0; Reader::min_size() - 1);
        for length in 0..v.len() {
            assert_eq!(Reader::new(&v[0..length]), None);
        }
    }

    #[test]
    fn reader_new_success_on_46_bytes_slice() {
        let v = vec!(0; Reader::min_size());
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_46_bytes_slice_with_nonzero_file_name_length() {
        let mut v = vec!(0; Reader::min_size());
        v[28] = 0x01;   // 1 byte file name
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_46_bytes_slice_with_nonzero_extra_field_length() {
        let mut v = vec!(0; Reader::min_size());
        v[30] = 0x01;   // 1 byte extra field
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_46_bytes_slice_with_nonzero_file_comment_length() {
        let mut v = vec!(0; Reader::min_size());
        v[32] = 0x01;   // 1 byte file comment
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_new_success_on_46_bytes_slice_with_all_nonzero_variables() {
        let mut v = vec!(0; Reader::min_size());
        v[28] = 0x01;   // 1 byte file name
        v[30] = 0x01;   // 1 byte extra field
        v[32] = 0x01;   // 1 byte file comment
        assert!(Reader::new(&v).is_some());
    }

    #[test]
    fn reader_signature_success_with_expected_signature() {
        let mut v = vec!(0; Reader::min_size());
        v[3] = 0x02;
        v[2] = 0x01;
        v[1] = 0x4b;
        v[0] = 0x50;
        let cdfh = Reader::new(&v).unwrap();
        assert_eq!(
            cdfh.signature(),
            Reader::expected_signature()
        );
    }

    #[test]
    fn reader_signature_success_with_unexpected_signature() {
        let v = vec!(0; Reader::min_size());
        let cdfh = Reader::new(&v).unwrap();
        assert_eq!(cdfh.signature(), 0);
    }

    #[test]
    fn reader_version_made_by_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 4, |v, version| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.version_made_by(), version);
        });
    }

    #[test]
    fn reader_version_needed_to_extract_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 6, |v, version| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.version_needed_to_extract(), version);
        });
    }

    #[test]
    fn reader_general_purpose_bit_flag_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 8, |v, f| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.general_purpose_bit_flag(), f);
        });
    }

    #[test]
    fn reader_compression_method_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 10, |v, c| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.compression_method(), c);
        });
    }

    #[test]
    fn reader_file_last_modification_time_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 12, |v, t| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.file_last_modification_time(), t);
        });
    }

    #[test]
    fn reader_file_last_modification_date_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 14, |v, d| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.file_last_modification_date(), d);
        });
    }

    #[test]
    fn reader_crc32_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 16, |v, crc| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.crc32(), crc);
        });
    }

    #[test]
    fn reader_compressed_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 20, |v, size| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.compressed_size(), size);
        });
    }

    #[test]
    fn reader_uncompressed_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 24, |v, size| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.uncompressed_size(), size);
        });
    }

    #[test]
    fn reader_file_name_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 28, |v, size| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.file_name_size(), size);
        });
    }

    #[test]
    fn reader_extra_field_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 30, |v, size| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.extra_field_size(), size);
        });
    }

    #[test]
    fn reader_file_comment_size_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 32, |v, size| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.file_comment_size(), size);
        });
    }

    #[test]
    fn reader_file_start_disk_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 34, |v, disk| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.file_start_disk(), disk);
        });
    }

    #[test]
    fn reader_interal_file_attributes_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 36, |v, attributes| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.interal_file_attributes(), attributes);
        });
    }

    #[test]
    fn reader_external_file_attributes_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 38, |v, attributes| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.external_file_attributes(), attributes);
        });
    }

    #[test]
    fn reader_local_file_header_relative_offset_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 42, |v, offset| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.local_file_header_relative_offset(), offset);
        });
    }

    #[test]
    fn reader_extra_field_position_success() {
        let mut v = vec!(0; Reader::min_size());
        test_all_u16_at(&mut v, 28, |v, size| {
            let cdfh = Reader::new(v).unwrap();
            assert_eq!(
                cdfh.extra_field_position(),
                Reader::min_size() + size as usize
            );
        });
    }

    #[test]
    fn reader_file_comment_position_success() {
        let mut v = vec!(0; Reader::min_size());
        test_some_u32_at(&mut v, 28, |v, size| {
            let expected_position =
                Reader::min_size() +
                (size >> 16) as usize +
                (size & 0xffff) as usize;

            let cdfh = Reader::new(v).unwrap();
            assert_eq!(cdfh.file_comment_position(), expected_position);
            
        });
    }

    #[test]
    fn reader_file_name_success_with_zero_length() {
        let v = vec!(0; Reader::min_size());
        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.file_name(), Some(&b""[..]));
    }

    #[test]
    fn reader_extra_field_success_with_zero_length() {
        let v = vec!(0; Reader::min_size());
        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.extra_field(), Some(&b""[..]));
    }

    #[test]
    fn reader_file_comment_success_with_zero_length() {
        let v = vec!(0; Reader::min_size());
        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.file_comment(), Some(&b""[..]));
    }

    #[test]
    fn reader_file_name_success_with_hello_world() {
        let mut v = vec!(0; Reader::min_size());
        v.extend_from_slice(b"Hello, World!");
        v[28] = (v.len() - Reader::min_size()) as u8;

        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.file_name(), Some(&b"Hello, World!"[..]));
    }

    #[test]
    fn reader_extra_field_success_with_hello_world() {
        let mut v = vec!(0; Reader::min_size());
        v.extend_from_slice(b"Hello, World!");
        v[30] = (v.len() - Reader::min_size()) as u8;

        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.extra_field(), Some(&b"Hello, World!"[..]));
    }

    #[test]
    fn reader_file_comment_success_with_hello_world() {
        let mut v = vec!(0; Reader::min_size());
        v.extend_from_slice(b"Hello, World!");
        v[32] = (v.len() - Reader::min_size()) as u8;

        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.file_comment(), Some(&b"Hello, World!"[..]));
    }

    #[test]
    fn reader_file_name_success_with_max_length() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize, 1);

        v[29] = 0xff;
        v[28] = 0xff;

        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.file_name(), Some(&v[Reader::min_size()..]));
    }

    #[test]
    fn reader_extra_field_success_with_max_length() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize, 1);

        v[31] = 0xff;
        v[30] = 0xff;

        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.extra_field(), Some(&v[Reader::min_size()..]));
    }

    #[test]
    fn reader_file_comment_success_with_max_length() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize, 1);

        v[33] = 0xff;
        v[32] = 0xff;

        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.file_comment(), Some(&v[Reader::min_size()..]));
    }

    #[test]
    fn reader_file_name_failure_on_too_short_buffer() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize - 1, 1);

        for length in 1..(std::u16::MAX as usize + 1) {
            v[29] = (length >> 8) as u8;
            v[28] = (length >> 0) as u8;

            let slice = &v[0..(Reader::min_size() + length - 1)];

            let cdfh = Reader::new(slice).unwrap();

            assert_eq!(cdfh.file_name(), None);
        }
    }

    #[test]
    fn reader_extra_field_failure_on_too_short_buffer() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize - 1, 1);

        for length in 1..(std::u16::MAX as usize + 1) {
            v[31] = (length >> 8) as u8;
            v[30] = (length >> 0) as u8;

            let slice = &v[0..(Reader::min_size() + length - 1)];

            let cdfh = Reader::new(slice).unwrap();

            assert_eq!(cdfh.extra_field(), None);
        }
    }

    #[test]
    fn reader_file_comment_failure_on_too_short_buffer() {
        let mut v = vec!(0; Reader::min_size());
        v.resize(Reader::min_size() + std::u16::MAX as usize - 1, 1);

        for length in 1..(std::u16::MAX as usize + 1) {
            v[33] = (length >> 8) as u8;
            v[32] = (length >> 0) as u8;

            let slice = &v[0..(Reader::min_size() + length - 1)];

            let cdfh = Reader::new(slice).unwrap();

            assert_eq!(cdfh.file_comment(), None);
        }
    }

    #[test]
    fn reader_all_variables_success_with_hello_world() {
        let hello_file = &b"Hello, File!"[..];
        let hello_extra = &b"Hello, Extra!"[..];
        let hello_comment = &b"Hello, Comment!"[..];

        let mut v = vec!(0; Reader::min_size());
        v.extend_from_slice(hello_file);
        v.extend_from_slice(hello_extra);
        v.extend_from_slice(hello_comment);

        v[28] = hello_file.len() as u8;
        v[30] = hello_extra.len() as u8;
        v[32] = hello_comment.len() as u8;

        let cdfh = Reader::new(&v).unwrap();

        assert_eq!(cdfh.file_name(), Some(hello_file));
        assert_eq!(cdfh.extra_field(), Some(hello_extra));
        assert_eq!(cdfh.file_comment(), Some(hello_comment));
    }
}

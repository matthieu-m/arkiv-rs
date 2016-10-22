//! # Zip format
//!
//!
//! ## Sketch
//!
//! A .ZIP archive:
//! - stores files in arbitrary order
//! - compress each file independently from one another
//! - may contain files that are not to be read (deleted/updated)
//! - may contain unused blobs in-between files
//! - may contain a non-file start (self-extracting archives, or steganography)
//! - may contain up to 65,535 bytes of data after the central directory
//! - has a central directory, describing the current set of files in the
//! archive, located toward the end of the file
//!
//!
//! ## Structural records
//!
//! There are 4 records:
//!
//! - End of Central Directory (EOCD)
//! - Central Directory File Header
//! - Local File Header
//! - Data Descriptor
//!
//!
//! ## Encoding
//!
//! Each record contains multiple fields. Multi-bytes fields are stored in
//! little-endian order.
//!
//!
//! ## End of Central Directory
//!
//! This record is located toward the end of the archive (within 65,535 bytes of
//! the end, to be specific).
//!
//! Format (courtesy of Wikipedia):
//!
//! +---------------------------------------------------------------------+
//! | Offset  | Bytes  |                          Description             |
//! |---------+--------+--------------------------------------------------|
//! |      0  | 4      | End of central directory signature = 0x06054b50  |
//! |      4  | 2      | Number of this disk                              |
//! |      6  | 2      | Disk where central directory starts              |
//! |      8  | 2      | Number of central directory records on this disk |
//! |     10  | 2      | Total number of central directory records        |
//! |     12  | 4      | Size of central directory (bytes)                |
//! |     16  | 4      | Offset of start of central directory [1]         |
//! |     20  | 2      | Comment length (n)                               |
//! |     22  | n      | Comment                                          |
//! +---------------------------------------------------------------------+
//!
//! <sup>1</sup> The offset is given relative to the start of the archive.
//!
//!
//! ## Central Directory File Header
//!
//! This component contains the list of files "officially" present in the
//! archive. Some other files, either deleted or updated, may be present in the
//! archive without appearing here.
//!
//! Format (courtesy of Wikipedia):
//!
//! +-------------------------------------------------------------------------+
//! | Offset  | Bytes  |                   Description                        |
//! |---------+--------+------------------------------------------------------|
//! | 0       | 4      | Central directory file header signature = 0x02014b50 |
//! | 4       | 2      | Version made by                                      |
//! | 6       | 2      | Version needed to extract (minimum)                  |
//! | 8       | 2      | General purpose bit flag                             |
//! | 10      | 2      | Compression method                                   |
//! | 12      | 2      | File last modification time                          |
//! | 14      | 2      | File last modification date                          |
//! | 16      | 4      | CRC-32                                               |
//! | 20      | 4      | Compressed size                                      |
//! | 24      | 4      | Uncompressed size                                    |
//! | 28      | 2      | File name length (n)                                 |
//! | 30      | 2      | Extra field length (m)                               |
//! | 32      | 2      | File comment length (k)                              |
//! | 34      | 2      | Disk number where file starts                        |
//! | 36      | 2      | Internal file attributes                             |
//! | 38      | 4      | External file attributes                             |
//! | 42      | 4      | Relative offset of local file header [1].            |
//! | 46      | n      | File name                                            |
//! | 46+n    | m      | Extra field                                          |
//! | 46+n+m  | k      | File comment                                         |
//! +-------------------------------------------------------------------------+
//!
//! <sup>1</sup> This is the number of bytes between the start of the first disk
//! on which the file occurs, and the start of the local file header. This
//! allows software reading the central directory to locate the position of the
//! file inside the .ZIP file.
//!
//!
//! ## Local File Header
//!
//! This record immediately precedes any file located in the archive. This makes
//! scanning a corrupted archive to recover files possible, although the
//! preferred method should be to just use the Central Directory Header to
//! iterate over it.
//!
//! Format (courtesy of Wikipedia):
//!
//! +-------------------------------------------------------------+
//! | Offset  | Bytes  |             Description                  |
//! |---------+--------+------------------------------------------|
//! | 0       | 4      | Local file header signature = 0x04034b50 |
//! | 4       | 2      | Version needed to extract (minimum)      |
//! | 6       | 2      | General purpose bit flag                 |
//! | 8       | 2      | Compression method                       |
//! | 10      | 2      | File last modification time              |
//! | 12      | 2      | File last modification date              |
//! | 14      | 4      | CRC-32                                   |
//! | 18      | 4      | Compressed size                          |
//! | 22      | 4      | Uncompressed size                        |
//! | 26      | 2      | File name length (n)                     |
//! | 28      | 2      | Extra field length (m)                   |
//! | 30      | n      | File name                                |
//! | 30+n    | m      | Extra field                              |
//! +-------------------------------------------------------------+
//!
//! The extra field is divided in chunks, each prepended by a 16-bits ID code
//! followed by a 16-bits content length. It is used to store optional data such
//! as OS-specific attributes.
//!
//! Note that in a well-formed archive, all those pieces of information were
//! already present in the Central Directory File Header. They are redundantly
//! stored in here to allow recovering data from incomplete or corrupted
//! archives.
//!
//!
//! ## Data Descriptor
//!
//! If the bit 0x08 of the general purpose bit flag in the Local File header is
//! set, then the file compressed size, uncompressed size and the CRC-32 were
//! unknown at the time of writing. This may happen when streaming, for example.
//!
//! In this case, the Local File Header contains 0s in those 3 fields, and a
//! Data Descriptor is added after the compressed data.
//!
//! Note that in a well-formed archive, the sizes and CRC-32 are already known
//! from the Central Directory File Header. Therefore, it is known at which
//! offset to look for the Data Descriptor anyway.
//!
//! Format (courtesy of Wikipedia):
//!
//! +--------------------------------------------------------------------+
//! | Offset  | Bytes  |                 Description                     |
//! |---------+--------+-------------------------------------------------|
//! | 0       | 0/4    | Optional data descriptor signature = 0x08074b50 |
//! | 0/4     | 4      | CRC-32                                          |
//! | 4/8     | 4      | Compressed size                                 |
//! | 8/12    | 4      | Uncompressed size                               |
//! +--------------------------------------------------------------------+
//!
//!
//! ## References
//!
//! - Wikipedia: https://en.wikipedia.org/wiki/Zip_(file_format)
//! - PKWare: http://www.pkware.com/documents/casestudies/APPNOTE.TXT

mod eocd;
pub use self::eocd::EndOfCentralDirectoryReader;

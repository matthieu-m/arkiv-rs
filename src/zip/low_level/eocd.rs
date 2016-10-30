//! # Low-level access to the End Of Central Directory records of a ZIP archive

use std::iter::Iterator;

use api::Reader;
use utils::{Slice, skip};

use zip::raw::EndOfCentralDirectoryReader;

/// An iterator over all potential End of Central Directory records within the
/// slice, iterating *backward* (from the end of the slice).
///
/// This is an iterator for two reasons:
///
/// - a correct archive may embed multiple "fake" such records within the
///   the comment field of the real record.
///
/// - a user may be interested in browsing all records in an archive, to see
///   its history.
///
/// To quickly get to the most likely End of Central Directory record, use the
/// `locate_end_of_central_directory` function below.
///
/// Note: this iterator does not attempt to validate the potential records in
/// any way.
#[derive(Debug)]
pub struct EndOfCentralDirectoryIterator<'a> {
    data: Slice<'a>,
    rpos: usize,
}

impl<'a> EndOfCentralDirectoryIterator<'a> {
    /// Returns an instance of EndOfCentralDirectoryIterator to search for End
    /// of Central Directory records within the slice.
    ///
    /// The most up-to-date record is in the last
    /// `EndOfCentralDirectoryReader::max_size()` bytes of the archive.
    pub fn new(slice: &'a [u8]) -> EndOfCentralDirectoryIterator<'a> {
        EndOfCentralDirectoryIterator {
            data: Slice::new(slice),
            rpos: Self::min_size().wrapping_sub(1)
        }
    }

    fn min_size() -> usize { EndOfCentralDirectoryReader::min_size() }
}

impl<'a> Iterator for EndOfCentralDirectoryIterator<'a> {
    type Item = EndOfCentralDirectoryReader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.rpos < self.data.len() {
            self.rpos = self.rpos.wrapping_add(1);

            let pos = self.data.len().wrapping_sub(self.rpos);

            if let Some(s) = self.data.slice(pos..pos.wrapping_add(4)) {
                if s.raw() == b"PK\x05\x06" {
                    return EndOfCentralDirectoryReader::new(
                        skip(self.data.raw(), pos)
                    );
                }
            }
        }

        None
    }
}

/// Returns the most likely End of Central Directory record in the archive.
///
/// The heuristic used is:
///
/// - to look, backward, in the last `EndOfCentralDirectoryReader::max_size()`
///   bytes of the archive.
///
/// - to pick the first record whose comment length field concords with the size
///   of the archive.
///
pub fn locate_end_of_central_directory<'a, R: ?Sized>(reader: &'a R)
    -> Option<EndOfCentralDirectoryReader<'a>>
    where R: Reader + 'a
{
    type EocdReader<'a> = EndOfCentralDirectoryReader<'a>;

    fn matches<'a>(eocd: &EocdReader<'a>, slice: &'a [u8]) -> bool {
        let comment_size = eocd.comment_size() as usize;

        let eocd_end = (eocd.raw().as_ptr() as usize)
            .wrapping_add(EocdReader::min_size())
            .wrapping_add(comment_size);

        let slice_end = (slice.as_ptr() as usize).wrapping_add(slice.len());

        eocd_end == slice_end
    }

    let slice = {
        use std::cmp::min;

        let end = reader.size();
        let start = end - min(EocdReader::max_size(), end);

        reader.get(start..end)
    };

    EndOfCentralDirectoryIterator::new(slice)
        .filter(|eocd| matches(eocd, slice))
        .next()
}

#[cfg(test)]
mod tests {
    use utils::position;
    use zip::raw::EndOfCentralDirectoryReader as EocdReader;
    use super::{EndOfCentralDirectoryIterator, locate_end_of_central_directory};

    #[test]
    fn iter_eocd_none_on_too_small_slice() {
        let v = vec![0; EocdReader::min_size() - 1];
        for length in 0..v.len() {
            let mut it = EndOfCentralDirectoryIterator::new(&v[..length]);

            assert_eq!(it.next(), None);
        }
    }

    #[test]
    fn iter_ecod_none_on_maximum_size_zeroed_slice() {
        let v = vec![0; EocdReader::max_size()];
        let mut it = EndOfCentralDirectoryIterator::new(&v[..]);

        assert_eq!(it.next(), None);
    }

    #[test]
    fn iter_eocd_once_on_minimum_size_slice() {
        let v = {
            let mut v = vec![0; EocdReader::min_size()];
            v[3] = 0x06;
            v[2] = 0x05;
            v[1] = 0x4b;
            v[0] = 0x50;
            v
        };

        let mut it = EndOfCentralDirectoryIterator::new(&v[..]);

        if let Some(eocd) = it.next() {
            assert_eq!(position(eocd.raw(), &v[..]), Some(0));
        } else {
            assert!(false);
        }

        assert_eq!(it.next(), None);
    }

    #[test]
    fn iter_eocd_all_on_maximum_size_slice() {
        let max_pos = (EocdReader::max_size() - EocdReader::min_size()) / 4 * 4;

        let v = {
            let mut v = vec![0; EocdReader::max_size()];
            for i in 0..(max_pos / 4 + 1) {
                let base_index = i * 4;
                v[base_index + 3] = 0x06;
                v[base_index + 2] = 0x05;
                v[base_index + 1] = 0x4b;
                v[base_index + 0] = 0x50;
            }
            v
        };

        let mut it = EndOfCentralDirectoryIterator::new(&v[..]);

        let mut pos = max_pos + 4;
        let mut total = 0;

        while let Some(eocd) = it.next() {
            assert_eq!(eocd.signature(), EocdReader::expected_signature());
            assert_eq!(position(eocd.raw(), &v[..]), Some(pos - 4));
            pos -= 4;
            total += 1;
        }

        assert_eq!(total, max_pos / 4 + 1);
    }

    #[test]
    fn locate_eocd_none_on_too_small_slice() {
        let v = vec![0; EocdReader::min_size() - 1];
        for length in 0..v.len() {
            assert_eq!(locate_end_of_central_directory(&v[..length]), None);
        }
    }

    #[test]
    fn locate_eocd_none_on_maximum_size_zeroed_slice() {
        let v = vec![0; EocdReader::max_size()];
        assert_eq!(locate_end_of_central_directory(&v[..]), None);
    }

    #[test]
    fn locate_eocd_on_minimum_size_slice() {
        let v = {
            let mut v = vec![0; EocdReader::min_size()];
            v[3] = 0x06;
            v[2] = 0x05;
            v[1] = 0x4b;
            v[0] = 0x50;
            v
        };

        if let Some(eocd) = locate_end_of_central_directory(&v[..]) {
            assert_eq!(position(eocd.raw(), &v[..]), Some(0));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn locate_eocd_latest_on_maximum_size_full_slice() {
        let max_pos = (EocdReader::max_size() - EocdReader::min_size()) / 4 * 4;

        let v = {
            let mut v = vec![0; EocdReader::max_size()];
            for i in 0..(max_pos / 4 + 1) {
                let base_index = i * 4 + 3;
                v[base_index + 3] = 0x06;
                v[base_index + 2] = 0x05;
                v[base_index + 1] = 0x4b;
                v[base_index + 0] = 0x50;
            }
            v
        };

        if let Some(eocd) = locate_end_of_central_directory(&v[..]) {
            assert_eq!(position(eocd.raw(), &v[..]), Some(65535));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn locate_eocd_latest_valid_on_slice() {
        let v = {
            let mut v = vec![0; EocdReader::min_size() + 8];
            v[28] = 0x01;   // EOCD [8..30), wrong comment field size
            v[24] = 0x04;   // EOCD [4..30), right comment field size   <=
            v[20] = 0x08;   // EOCD [0..30), right comment field size

            v[11] = 0x06;
            v[10] = 0x05;
            v[9] = 0x4b;
            v[8] = 0x50;

            v[7] = 0x06;
            v[6] = 0x05;
            v[5] = 0x4b;
            v[4] = 0x50;

            v[3] = 0x06;
            v[2] = 0x05;
            v[1] = 0x4b;
            v[0] = 0x50;
            v
        };

        if let Some(eocd) = locate_end_of_central_directory(&v[..]) {
            assert_eq!(position(eocd.raw(), &v[..]), Some(4));
        } else {
            assert!(false);
        }
    }
}

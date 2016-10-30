//! # Low-level access to the Central Directory File Header records of a ZIP
//! archive

use std::iter::Iterator;

use super::super::utils::Slice;

use super::raw::CentralDirectoryFileHeaderReader;

/// An iterator over a contiguous sequence of Central Directory File Header
/// records.
///
/// The iterator returns the Central Directory File Header records in the order
/// in which they appear in the archive.
///
/// Note in particular that there is no guarantee:
///
/// - that the sequence be contained within a single part
/// - that the files in the archive appear in the same order
///
/// Note: this iterator does not attempt to validate the potential records in
/// any way, it does not even check that the signature matches.
#[derive(Debug)]
pub struct CentralDirectoryFileHeaderIterator<'a> {
    data: Slice<'a>,
    remaining: usize,
}

impl<'a> CentralDirectoryFileHeaderIterator<'a> {
    /// Returns an instance of CentralDirectoryFileHeaderIterator.
    ///
    /// The `slice` parameter is the slice iterated over. It should start
    /// exactly on the boundary of the first record. See
    /// `EndOfCentralDirectoryReader::central_directory_offset` and
    /// `EndOfCentralDirectoryReader::central_directory_size`.
    ///
    /// The `total` parameter is the maximum number of records that will be
    /// returned by the iterator. See 
    /// `EndOfCentralDirectoryReader::nb_central_directory_records`.
    pub fn new(slice: &'a [u8], total: usize)
        -> CentralDirectoryFileHeaderIterator<'a>
    {
        CentralDirectoryFileHeaderIterator {
            data: Slice::new(slice),
            remaining: total
        }
    }
}

impl<'a> Iterator for CentralDirectoryFileHeaderIterator<'a> {
    type Item = CentralDirectoryFileHeaderReader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        if let Some(cdfh) =
            CentralDirectoryFileHeaderReader::new(self.data.raw())
        {
            self.data = self.data.skip(cdfh.raw().len());
            self.remaining -= 1;

            Some(cdfh)
        }
        else
        {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CentralDirectoryFileHeaderIterator as CdfhIterator;
    use super::super::raw::CentralDirectoryFileHeaderReader as CdfhReader;
    use super::super::super::utils::position;

    #[test]
    fn iter_cdfh_none_on_too_small_slice() {
        let v = vec![0; CdfhReader::min_size() - 1];
        for length in 0..v.len() {
            let mut it = CdfhIterator::new(&v[..length], 1);

            assert_eq!(it.next(), None);
        }
    }

    #[test]
    fn iter_cdfh_exact_on_zeroed_slice() {
        let v = vec![0; CdfhReader::min_size() * 40];
        for count in 0..40 {
            let it = CdfhIterator::new(&v[..], count);

            assert_eq!(it.count(), count);
        }
    }

    #[test]
    fn iter_cdfh_early_stop_on_zeroed_slice() {
        let v = vec![0; CdfhReader::min_size() * 40];
        for count in 0..40 {
            let it =
                CdfhIterator::new(&v[..CdfhReader::min_size() * count], 50);

            assert_eq!(it.count(), count);
        }
    }

    #[test]
    fn iter_cdfh_early_stop_on_maxed_slice() {
        let v = vec![0xff; CdfhReader::max_size() * 40];

        for count in 0..40 {
            let it =
                CdfhIterator::new(&v[..CdfhReader::max_size() * count], 50);

            assert_eq!(it.count(), count);
        }
    }

    #[test]
    fn iter_cdfh_adaptable_length_on_large_slice() {
        let min_size = CdfhReader::min_size();

        let v = {
            let mut v = vec![0; min_size * 2 + 40];
            v[32] = 22;
            v[30] = 10;
            v[28] =  8;
            v
        };

        let all: Vec<_> = CdfhIterator::new(&v[..], 2).collect();

        assert_eq!(all.len(), 2);
        assert_eq!(position(all[0].raw(), &v[..]), Some(0));
        assert_eq!(position(all[1].raw(), &v[..]), Some(min_size + 40));
    }
}

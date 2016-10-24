//! # Test utilities

use std;

/// Invokes the function 65536 times, once for each possible u16 value after
/// writing it in little-endian at [index, index+1] in the buffer.
pub fn test_all_u16_at<F>(buffer: &mut [u8], index: usize, f: F)
    where F: Fn(&[u8], u16) -> ()
{
    for data in 0..65536u32 {
        let data = data as u16;
        buffer[index + 1] = (data >> 8) as u8;
        buffer[index + 0] = (data >> 0) as u8;

        f(buffer, data)
    }
}

/// Invokes the function 4*65536 times, once for each u32 value close to 0,
/// close to the maximum, and with a scattering of values in the middle, after
/// writing it in little-endian at [index, index+3] in the buffer.
pub fn test_some_u32_at<F>(buffer: &mut [u8], index: usize, f: F)
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
        test(buffer, 65536 + data * 251, index, &f);
    }

    for data in 0..65536 {
        test(buffer, std::u32::MAX - 65536 - data * 251, index, &f);
    }

    for data in 0..65536 {
        test(buffer, std::u32::MAX - data, index, &f);
    }
}

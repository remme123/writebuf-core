//! A writeable buffer that implements [`fmt::Write`] or [`ufmt::uWrite`](https://docs.rs/ufmt/latest/ufmt/trait.uWrite.html).
//!
//! # Example
//! ```
//! use writebuf_core::WriteBuf;
//! use ufmt::{uwrite, uWrite};
//!
//! // write to buffer
//! let mut buf: WriteBuf<10> = WriteBuf::from("123");
//! uwrite!(&mut buf, "{}", "456").ok();
//! uwrite!(&mut buf, "{}", 789).ok();
//! buf.write_str("0").ok();
//! buf.write_str("E").err();
//!
//! // convert to ASCII string
//! buf.into_ascii_lossy().as_str();
//! ```
//!
//! # ufmt
//! ufmt is more compact than core::fmt. By default, ufmt feature is enabled.

#![cfg_attr(not(test), no_std)]

use heapless::{Vec, String};

#[cfg(not(feature = "ufmt"))]
use core::fmt;
use core::ops::{Deref, DerefMut};

#[cfg(feature = "ufmt")]
use ufmt::uWrite;

#[derive(Default, Clone, Debug)]
pub struct WriteBuf<const N: usize> {
    buffer: Vec<u8, N>,
}

impl<const N: usize> WriteBuf<N> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Try convert to UTF-8 str
    pub fn to_str(&self) -> Result<&str, ()> {
        core::str::from_utf8(self.buffer.as_slice()).map_err(|_e| ())
    }

    /// Convert to ASCII string by replacing any invalid characters with `~`
    pub fn into_ascii_lossy(self) -> String<N> {
        let mut s = String::<N>::new();
        for &c in self.iter() {
            if c >= 0x80 {
                s.push('~').ok();
            } else {
                s.push(c as char).ok();
            }
        }
        s
    }
}

impl<T: AsRef<[u8]>, const N: usize> From<T> for WriteBuf<N> {
    fn from(value: T) -> Self {
        let data = value.as_ref();
        let mut buf = Self::new();
        buf.extend_from_slice(&data[..core::cmp::min(data.len(), N)]).ok();
        buf
    }
}

impl<const N: usize> Deref for WriteBuf<N> {
    type Target = Vec<u8, N>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<const N: usize> DerefMut for WriteBuf<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

#[cfg(not(feature = "ufmt"))]
impl<const N: usize> fmt::Write for WriteBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer.write_str(s)
    }
}

#[cfg(feature = "ufmt")]
impl<const N: usize> uWrite for WriteBuf<N> {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.buffer.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write() {
        let buf = WriteBuf::<10>::from("1000");
        assert_eq!(buf.to_str().unwrap(), "1000");
    }

    #[test]
    fn test_full() {
        let mut buf = WriteBuf::<10>::from("123456789");
        buf.write_str("abc").err();
        assert_eq!(buf.to_str().unwrap(), "123456789");
    }

    #[test]
    fn test_into_ascii_lossy() {
        let mut buf = WriteBuf::<10>::from("123456789");
        buf.resize_default(10).ok();
        buf[9] = 0x80u8;
        assert_eq!(buf.into_ascii_lossy(), "123456789~");
    }
}
//! [Data Type](https://wiki.vg/Raknet_Protocol#Data_types)

use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    ops::{Bound, Deref, DerefMut, RangeBounds},
    str::{from_utf8, Utf8Error},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

#[allow(non_camel_case_types)]
pub type u24 = u32;

pub type Magic = [u8; 16];

pub(crate) trait RaknetBuffer {
    fn len(&self) -> usize;

    fn len_mut(&mut self) -> &mut usize;

    fn ptr(&self) -> usize;

    fn ptr_mut(&mut self) -> &mut usize;

    fn capacity(&self) -> usize;

    /// slice valid buffer
    /// valid range should in `0..self.capacity()`
    fn slice(&self, range: impl RangeBounds<usize>) -> IResult<&[u8]>;

    fn clear(&mut self) {
        *self.ptr_mut() = 0;
        *self.len_mut() = 0
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// only error when `ptr` and `len` is invalid
    fn initialized(&self) -> IResult<&[u8]> {
        let [len, ptr] = [self.len(), self.ptr()];
        self.slice(..ptr + len)
    }

    fn advance(&mut self, size: usize) -> IResult<()> {
        // size check, prevent panic
        if self.len() < size {
            return Err(DataError::InsufficientByte);
        }
        *self.len_mut() -= size;
        *self.ptr_mut() += size;
        Ok(())
    }

    /// only error when `ptr` and `len` is invalid
    fn remaining(&self) -> IResult<&[u8]> {
        let [len, ptr] = [self.len(), self.ptr()];
        self.slice(ptr..ptr + len)
    }

    fn take_bytes(&mut self, size: usize) -> IResult<&[u8]> {
        let [len, ptr] = [self.len(), self.ptr()];
        // size check, prevent panic
        if len < self.capacity() {
            return Err(DataError::InsufficientByte);
        }

        self.advance(size)?;
        self.slice(ptr..ptr + len)
    }

    fn take_sized_bytes<const S: usize>(&mut self) -> IResult<[u8; S]> {
        let bytes = (self.take_bytes(S)?).try_into().expect("should not error");
        Ok(bytes)
    }

    /// Byte
    fn take_u8(&mut self) -> IResult<u8> {
        Ok(u8::from_be_bytes(self.take_sized_bytes::<1>()?))
    }

    /// Long
    fn take_i64(&mut self) -> IResult<i64> {
        Ok(i64::from_be_bytes(self.take_sized_bytes::<8>()?))
    }

    fn take_magic(&mut self) -> IResult<Magic> {
        self.take_sized_bytes::<16>()
    }

    /// short
    fn take_i16(&mut self) -> IResult<i16> {
        let bytes = self.take_sized_bytes::<2>()?;
        Ok(i16::from_be_bytes(bytes))
    }

    /// unsigned short
    fn take_u16(&mut self) -> IResult<u16> {
        let bytes = self.take_sized_bytes::<2>()?;
        Ok(u16::from_be_bytes(bytes))
    }

    /// unsigned short little endian
    fn take_u16_le(&mut self) -> IResult<u16> {
        let bytes = self.take_sized_bytes::<2>()?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn take_str(&mut self) -> IResult<&str> {
        let len = self.take_u16()? as usize;
        Ok(from_utf8(self.take_bytes(len)?)?)
    }

    fn take_bool(&mut self) -> IResult<bool> {
        // non-zero value is true
        Ok(self.take_u8()? != 0)
    }

    /// [From Golang](https://github.com/Sandertv/go-raknet/blob/master/internal/message/packet.go#L63)
    /// [From TS](https://github.com/RaptorsMC/RakNet/blob/master/lib/protocol/Packet.ts#L41)
    fn take_address(&mut self) -> IResult<SocketAddr> {
        let ipv = self.take_u8()?;
        let ip = match ipv {
            4 => {
                let ip: [u8; 4] = self.take_bytes(4)?.try_into().expect("should never fail");
                let port = self.take_u16()?;

                SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from(ip), port))
            }
            6 => {
                // not sure what is this
                self.advance(2)?;
                let port = self.take_u16_le()?;
                self.advance(4)?;
                let ip: [u8; 16] = self.take_bytes(16)?.try_into().expect("should never fail");
                self.advance(4)?;

                SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from(ip), port, 0, 0))
            }
            n => Err(DataError::UnsupportedIpv(n))?,
        };
        Ok(ip)
    }

    fn take_u24(&mut self) -> IResult<u24> {
        // add padding zero
        let mut bytes = [0; 4];
        bytes[..3].copy_from_slice(&self.take_sized_bytes::<3>()?[..]);
        Ok(u24::from_be_bytes(bytes))
    }

    fn take_time(&mut self) -> IResult<SystemTime> {
        let n = self
            .take_i64()?
            .try_into()
            .map_err(|_| DataError::InvalidTime)?;
        UNIX_EPOCH
            .checked_add(Duration::from_millis(n))
            .ok_or(DataError::InvalidTime)
    }

    fn take_u32(&mut self) -> IResult<u32> {
        let bytes = self.take_sized_bytes::<4>()?;
        Ok(u32::from_be_bytes(bytes))
    }
}

/// A fix size buffer
///
/// all `take_*` method will advance byte
#[derive(Debug, Clone)]
pub struct SizedBuffer<const N: usize> {
    buf: [u8; N],
    len: usize,
    ptr: usize,
}

impl<const N: usize> RaknetBuffer for SizedBuffer<N> {
    fn len(&self) -> usize {
        self.len
    }

    fn len_mut(&mut self) -> &mut usize {
        &mut self.len
    }

    fn ptr(&self) -> usize {
        self.ptr
    }

    fn ptr_mut(&mut self) -> &mut usize {
        &mut self.ptr
    }

    fn capacity(&self) -> usize {
        N
    }

    fn slice(&self, range: impl RangeBounds<usize>) -> IResult<&[u8]> {
        let begin = match range.start_bound() {
            Bound::Excluded(&n) => n + 1,
            Bound::Included(&n) => n,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Excluded(&n) => n,
            Bound::Included(&n) => n + 1,
            Bound::Unbounded => self.buf.len(),
        };
        if begin >= N || end > N {
            Err(DataError::OutOfBound)?
        }
        Ok(&self.buf[begin..end])
    }
}

impl<const N: usize> SizedBuffer<N> {
    pub fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
            ptr: 0,
        }
    }
}
//     pub fn clear(&mut self) {
//         self.len = 0;
//         self.ptr = 0;
//     }

//     pub fn is_empty(&self) -> bool {
//         self.len == 0
//     }

//     pub fn set_len(&mut self, len: usize) -> IResult<()> {
//         if N < len {
//             return Err(DataError::Overflow);
//         }
//         self.len = len;
//         Ok(())
//     }

//     pub fn set_ptr(&mut self, ptr: usize) -> IResult<()> {
//         if N < ptr {
//             return Err(DataError::Overflow);
//         }
//         self.ptr = ptr;
//         Ok(())
//     }

//     pub fn initialized(&self) -> &[u8] {
//         &self.buf[..self.ptr + self.len]
//     }

//     pub(crate) fn advance(&mut self, size: usize) -> IResult<()> {
//         // size check, prevent panic
//         if self.len < size {
//             return Err(DataError::InsufficientByte);
//         }
//         self.len -= size;
//         self.ptr += size;
//         Ok(())
//     }

//     pub(crate) fn remaining(&self) -> &[u8] {
//         &self.buf[self.ptr..][..self.len]
//     }

//     pub(crate) fn len(&self) -> usize {
//         self.len
//     }

//     pub(crate) fn take_bytes(&mut self, size: usize) -> IResult<&[u8]> {
//         // size check, prevent panic
//         if self.len < size {
//             return Err(DataError::InsufficientByte);
//         }
//         let bytes = &self.buf[self.ptr..self.ptr + size];
//         // advance
//         self.len -= size;
//         self.ptr += size;
//         Ok(bytes)
//     }

//     pub(crate) fn take_sized_bytes<const S: usize>(&mut self) -> IResult<[u8; S]> {
//         let bytes = (self.take_bytes(S)?).try_into().expect("should not error");
//         Ok(bytes)
//     }

//     /// Byte
//     pub(crate) fn take_u8(&mut self) -> IResult<u8> {
//         Ok(u8::from_be_bytes(self.take_sized_bytes::<1>()?))
//     }

//     /// Long
//     pub(crate) fn take_i64(&mut self) -> IResult<i64> {
//         Ok(i64::from_be_bytes(self.take_sized_bytes::<8>()?))
//     }

//     pub(crate) fn take_magic(&mut self) -> IResult<Magic> {
//         self.take_sized_bytes::<16>()
//     }

//     /// short
//     pub(crate) fn take_i16(&mut self) -> IResult<i16> {
//         let bytes = self.take_sized_bytes::<2>()?;
//         Ok(i16::from_be_bytes(bytes))
//     }

//     /// unsigned short
//     pub(crate) fn take_u16(&mut self) -> IResult<u16> {
//         let bytes = self.take_sized_bytes::<2>()?;
//         Ok(u16::from_be_bytes(bytes))
//     }

//     /// unsigned short little endian
//     pub(crate) fn take_u16_le(&mut self) -> IResult<u16> {
//         let bytes = self.take_sized_bytes::<2>()?;
//         Ok(u16::from_be_bytes(bytes))
//     }

//     pub(crate) fn take_str(&mut self) -> IResult<&str> {
//         let len = self.take_u16()? as usize;
//         Ok(from_utf8(self.take_bytes(len)?)?)
//     }

//     pub(crate) fn take_bool(&mut self) -> IResult<bool> {
//         // non-zero value is true
//         Ok(self.take_u8()? != 0)
//     }

//     /// [From Golang](https://github.com/Sandertv/go-raknet/blob/master/internal/message/packet.go#L63)
//     /// [From TS](https://github.com/RaptorsMC/RakNet/blob/master/lib/protocol/Packet.ts#L41)
//     pub(crate) fn take_address(&mut self) -> IResult<SocketAddr> {
//         let ipv = self.take_u8()?;
//         let ip = match ipv {
//             4 => {
//                 let ip: [u8; 4] = self.take_bytes(4)?.try_into().expect("should never fail");
//                 let port = self.take_u16()?;

//                 SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from(ip), port))
//             }
//             6 => {
//                 // not sure what is this
//                 self.advance(2)?;
//                 let port = self.take_u16_le()?;
//                 self.advance(4)?;
//                 let ip: [u8; 16] = self.take_bytes(16)?.try_into().expect("should never fail");
//                 self.advance(4)?;

//                 SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from(ip), port, 0, 0))
//             }
//             n => Err(DataError::UnsupportedIpv(n))?,
//         };
//         Ok(ip)
//     }

//     pub(crate) fn take_u24(&mut self) -> IResult<u24> {
//         // add padding zero
//         let mut bytes = [0; 4];
//         bytes[..3].copy_from_slice(&self.take_sized_bytes::<3>()?[..]);
//         Ok(u24::from_be_bytes(bytes))
//     }

//     pub(crate) fn take_time(&mut self) -> IResult<SystemTime> {
//         let n = self
//             .take_i64()?
//             .try_into()
//             .map_err(|_| DataError::InvalidTime)?;
//         UNIX_EPOCH
//             .checked_add(Duration::from_millis(n))
//             .ok_or(DataError::InvalidTime)
//     }

//     pub(crate) fn take_u32(&mut self) -> IResult<u32> {
//         let bytes = self.take_sized_bytes::<4>()?;
//         Ok(u32::from_be_bytes(bytes))
//     }
// }

impl<const N: usize> Deref for SizedBuffer<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<const N: usize> DerefMut for SizedBuffer<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("run out of byte")]
    InsufficientByte,

    #[error("invalid char in string")]
    Utf8Error(#[from] Utf8Error),

    #[error("unsupported ipv{0}")]
    UnsupportedIpv(u8),

    #[error("buffer overflow")]
    Overflow,

    #[error("invalid time stamp")]
    InvalidTime,

    #[error("slice out of bound")]
    OutOfBound,
}

type IResult<T> = Result<T, DataError>;

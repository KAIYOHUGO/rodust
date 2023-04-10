use std::{
    borrow::Cow,
    num::TryFromIntError,
    str::{from_utf8, Utf8Error},
};

use thiserror::Error;

pub trait Deserialize<'de>: Sized {
    type Error;
    type Arg<'arg>;
    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("insufficient byte")]
    Incomplete,

    #[error("not find patten")]
    NotFind,

    #[error("str parse error")]
    InvalidStr(#[from] Utf8Error),

    #[error("no match value")]
    NoMatch,

    #[error("number overflow")]
    NumOverflow(#[from] TryFromIntError),

    #[error("malformed bytes")]
    Malformed,
}

impl<'de: 's, 's, const N: usize> Deserialize<'de> for &'s [u8; N] {
    type Error = Error;

    type Arg<'arg> = ();

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        _: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let end = *offset + N;
        if buf.len() < end {
            Err(Self::Error::Incomplete)?
        }
        let ret = buf[*offset..end].try_into().expect("should not fail");
        // advance
        *offset += N;
        Ok(ret)
    }
}

impl<'de, T, const N: usize> Deserialize<'de> for [T; N]
where
    T: Deserialize<'de> + Sized,
    for<'a> T::Arg<'a>: Clone,
{
    type Error = T::Error;

    type Arg<'arg> = T::Arg<'arg>;

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let data: Result<Vec<_>, _> = (0..N)
            .map(|_| Deserialize::deserialize(buf, offset, arg.clone()))
            .collect();

        Ok(data?
            .try_into()
            .unwrap_or_else(|_| panic!("should not fail")))
    }
}

impl<'de: 's, 's> Deserialize<'de> for &'s [u8] {
    type Error = Error;

    type Arg<'arg> = SliceArg<'arg>;

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let ret = match arg {
            SliceArg::Len(len) => {
                let end = *offset + len;
                if buf.len() < end {
                    Err(Self::Error::Incomplete)?
                }
                let bytes = &buf[*offset..end];
                *offset += len;
                bytes
            }
            SliceArg::Until(byte) => {
                if buf.len() <= *offset {
                    Err(Self::Error::Incomplete)?
                }
                let remain = &buf[*offset..];
                let pos = remain
                    .windows(byte.len())
                    .position(|b| b == byte)
                    .ok_or(Self::Error::NotFind)?;
                &remain[..pos]
            }
            SliceArg::All => {
                if buf.len() <= *offset {
                    Err(Self::Error::Incomplete)?
                }
                let bytes = &buf[*offset..];
                *offset = buf.len();
                bytes
            }
        };
        Ok(ret)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SliceArg<'arg> {
    Len(usize),
    Until(&'arg [u8]),
    All,
}

impl<'de: 's, 's> Deserialize<'de> for &'s str {
    type Error = Error;

    type Arg<'arg> = SliceArg<'arg>;

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let bytes: &[u8] = Deserialize::deserialize(buf, offset, arg)?;
        let ret = from_utf8(bytes)?;
        Ok(ret)
    }
}

impl<'de> Deserialize<'de> for () {
    type Error = Error;

    type Arg<'arg> = ();

    fn deserialize<'arg>(
        _: &'de [u8],
        _: &mut usize,
        _: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl<'de, T> Deserialize<'de> for Option<T>
where
    T: Deserialize<'de>,
{
    type Error = T::Error;

    type Arg<'arg> = T::Arg<'arg>;

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let some = Some(T::deserialize(buf, offset, arg)?);
        Ok(some)
    }
}

impl<'de: 'c, 'c, T> Deserialize<'de> for Cow<'c, T>
where
    &'c T: Deserialize<'de>,
    T: ToOwned,
{
    type Error = <&'c T as Deserialize<'de>>::Error;

    type Arg<'arg> = <&'c T as Deserialize<'de>>::Arg<'arg>;

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let cow = Cow::Borrowed(<&T>::deserialize(buf, offset, arg)?);
        Ok(cow)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    LE,
    BE,
    NE,
}

macro_rules! impl_byte {
    ($t:ty, $byte:ident, $e:expr) => {
        impl<'de> Deserialize<'de> for $t {
            type Error = Error;

            type Arg<'arg> = ();

            fn deserialize<'arg>(
                buf: &'de [u8],
                offset: &mut usize,
                _: Self::Arg<'arg>,
            ) -> Result<Self, Self::Error> {
                if buf.len() < *offset + 1 {
                    Err(Self::Error::Incomplete)?
                }
                let $byte = buf[*offset];
                *offset += 1;
                Ok($e)
            }
        }
    };
}

impl_byte!(u8, b, b);
impl_byte!(i8, b, Self::from_ne_bytes([b]));
impl_byte!(bool, b, b != 0);

macro_rules! impl_num {
    ($t:ty, $size:literal) => {
        impl<'de> Deserialize<'de> for $t {
            type Error = Error;

            type Arg<'arg> = Endian;

            fn deserialize<'arg>(
                buf: &'de [u8],
                offset: &mut usize,
                arg: Self::Arg<'arg>,
            ) -> Result<Self, Self::Error> {
                let bytes: &[u8; $size] =
                    Deserialize::deserialize(buf, offset, Default::default())?;
                let num = match arg {
                    Endian::LE => Self::from_le_bytes(bytes.clone()),
                    Endian::BE => Self::from_be_bytes(bytes.clone()),
                    Endian::NE => Self::from_ne_bytes(bytes.clone()),
                };
                Ok(num)
            }
        }
    };
}

impl_num!(u16, 2);
impl_num!(i16, 2);
impl_num!(u32, 4);
impl_num!(i32, 4);
impl_num!(u64, 8);
impl_num!(i64, 8);

impl_num!(f32, 4);
impl_num!(f64, 8);

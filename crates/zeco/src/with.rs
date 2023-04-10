use std::{
    borrow::Cow,
    marker::PhantomData,
    ops::{Range, RangeInclusive},
    str::{from_utf8, Utf8Error},
};

use crate::{des::Error, Deserialize, Len};

pub trait DeserializeWith<'de, T>: Deserialize<'de> {
    fn deserialize_with<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<T, Self::Error>;
}

impl<'de, T, S, De, Te> DeserializeWith<'de, T> for S
where
    S: Deserialize<'de, Error = De> + TryTo<T, Error = Te>,
    De: From<De> + From<Te>,
{
    fn deserialize_with<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<T, Self::Error> {
        let des: S = Deserialize::deserialize(buf, offset, arg)?;
        let ret = des.try_to()?;
        Ok(ret)
    }
}

pub trait TryTo<T> {
    type Error;

    fn try_to(self) -> Result<T, Self::Error>;
}

impl<T, S> TryTo<Option<T>> for S
where
    S: TryTo<T>,
{
    type Error = S::Error;

    fn try_to(self) -> Result<Option<T>, Self::Error> {
        self.try_to().map(Some)
    }
}

impl<'t, T, S> TryTo<Cow<'t, T>> for S
where
    S: TryTo<&'t T>,
    T: ToOwned,
{
    type Error = S::Error;

    fn try_to(self) -> Result<Cow<'t, T>, Self::Error> {
        self.try_to().map(Cow::Borrowed)
    }
}

impl<'s, S, E> TryTo<&'s str> for S
where
    S: TryTo<&'s [u8], Error = E>,
    E: From<E> + From<Utf8Error>,
{
    type Error = S::Error;

    fn try_to(self) -> Result<&'s str, Self::Error> {
        let string = from_utf8(self.try_to()?)?;
        Ok(string)
    }
}

impl<T, S> TryTo<RangeInclusive<T>> for [S; 2]
where
    S: TryTo<T>,
{
    type Error = S::Error;

    fn try_to(self) -> Result<RangeInclusive<T>, Self::Error> {
        let [begin, end] = self;
        Ok(begin.try_to()?..=end.try_to()?)
    }
}

impl<T, S> TryTo<Range<T>> for [S; 2]
where
    S: TryTo<T>,
{
    type Error = S::Error;

    fn try_to(self) -> Result<Range<T>, Self::Error> {
        let [begin, end] = self;
        Ok(begin.try_to()?..end.try_to()?)
    }
}

impl<T, S, const N: usize> TryTo<[T; N]> for [S; N]
where
    S: TryTo<T>,
{
    type Error = S::Error;

    fn try_to(self) -> Result<[T; N], Self::Error> {
        let data: Result<Vec<_>, _> = self.into_iter().map(|s| s.try_to()).collect();
        Ok(data?
            .try_into()
            .unwrap_or_else(|_| panic!("should not fail")))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixLen<'s, L, E = Error>
where
    L: for<'l> Deserialize<'l> + Into<usize>,
{
    bytes: &'s [u8],
    _p: PhantomData<(L, E)>,
}

impl<'de: 's, 's, L, E> Deserialize<'de> for PrefixLen<'s, L, E>
where
    L: for<'l> Deserialize<'l> + Into<usize>,
    E: From<E> + From<<L as Deserialize<'de>>::Error> + From<Error>,
{
    type Error = E;
    type Arg<'arg> = <L as Deserialize<'de>>::Arg<'arg>;

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        arg: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let len: L = Deserialize::deserialize(buf, offset, arg)?;
        let bytes = Deserialize::deserialize(buf, offset, Len(len.into()))?;
        Ok(Self {
            bytes,
            _p: PhantomData,
        })
    }
}

impl<'s, L, E> TryTo<&'s [u8]> for PrefixLen<'s, L, E>
where
    L: for<'l> Deserialize<'l> + Into<usize>,
{
    type Error = E;

    fn try_to(self) -> Result<&'s [u8], Self::Error> {
        Ok(self.bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarInt<T, E = Error>
where
    T: integer_encoding::VarInt,
{
    int: T,
    _p: PhantomData<E>,
}

impl<'de, T, E> Deserialize<'de> for VarInt<T, E>
where
    T: integer_encoding::VarInt,
    E: From<Error>,
{
    type Error = E;
    type Arg<'arg> = ();

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        _: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        Ok(T::decode_var(buf)
            .map(|(int, len)| {
                *offset += len;
                Self {
                    int,
                    _p: PhantomData,
                }
            })
            .ok_or(Error::Malformed)?)
    }
}

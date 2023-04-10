use std::{net::SocketAddr, ops::RangeInclusive, time::SystemTime};

use thiserror::Error;

use crate::types::*;

pub trait ParsePacket<'b>: Sized {
    type Error;
    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error>;
    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error>;
}

#[repr(u8)]
#[derive(Debug)]
pub enum PacketId {
    UConnPing = 0x01,
    UConnConnPing = 0x02,
    UConnPong = 0x1c,
    OConnReq1 = 0x05,
    OConnReply1 = 0x06,
    OConnReq2 = 0x07,
    OConnReply2 = 0x08,
    Incompatible = 0x19,
    /// [See](https://github.com/pmmp/RakLib/blob/8e6ba0541ac24b20b4da446ee272ae3699a4c1b1/src/protocol/Datagram.php#L24-L30)
    FrameSet(u8) = 0x80,
    Nack = 0xa0,
    Ack = 0xc0,
}

impl<'b> ParsePacket<'b> for PacketId {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let id = buf.take_u8()?;
        let ret = match id {
            0x01 => PacketId::UConnPing,
            0x02 => PacketId::UConnConnPing,
            0x1c => PacketId::UConnPong,
            0x05 => PacketId::OConnReq1,
            0x06 => PacketId::OConnReply1,
            0x07 => PacketId::OConnReq2,
            0x08 => PacketId::OConnReply2,
            0x19 => PacketId::Incompatible,
            id @ 0x80..=0x8d => PacketId::FrameSet(id),
            0xa0 => PacketId::Nack,
            0xc0 => PacketId::Ack,
            _ => Err(PacketError::UnknownPacket)?,
        };
        Ok(ret)
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum FramePacketId {
    ConnReq = 0x09,
    ConnReqAccept = 0x10,
    ConnPing = 0x00,
    ConnPong = 0x03,
    NewConn = 0x13,
    DisConn = 0x15,
    Game = 0xfe,
}

impl<'b> ParsePacket<'b> for FramePacketId {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let id = buf.take_u8()?;
        let ret = match id {
            0x09 => FramePacketId::ConnReq,
            0x10 => FramePacketId::ConnReqAccept,
            0x00 => FramePacketId::ConnPing,
            0x03 => FramePacketId::ConnPong,
            0x13 => FramePacketId::NewConn,
            0x15 => FramePacketId::DisConn,
            0xfe => FramePacketId::Game,
            _ => Err(PacketError::UnknownPacket)?,
        };
        Ok(ret)
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UConnPing {
    pub time: SystemTime,
    pub magic: Magic,
    pub client_guid: i64,
}

impl<'b> ParsePacket<'b> for UConnPing {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let time = buf.take_time()?;
        let magic = buf.take_magic()?;
        let client_guid = buf.take_i64()?;
        Ok(Self {
            time,
            magic,
            client_guid,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UConnConnPing {
    pub time: SystemTime,
    pub magic: Magic,
    pub client_guid: i64,
}

impl<'b> ParsePacket<'b> for UConnConnPing {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let time = buf.take_time()?;
        let magic = buf.take_magic()?;
        let client_guid = buf.take_i64()?;
        Ok(Self {
            time,
            magic,
            client_guid,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UConnPong<'p> {
    pub time: SystemTime,
    pub server_guid: i64,
    pub magic: Magic,
    pub server_id: &'p str,
}

impl<'b: 'p, 'p> ParsePacket<'b> for UConnPong<'p> {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let time = buf.take_time()?;
        let server_guid = buf.take_i64()?;
        let magic = buf.take_magic()?;
        let server_id = buf.take_str()?;
        Ok(Self {
            time,
            server_guid,
            magic,
            server_id,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConnPing {
    pub time: SystemTime,
}

impl<'b> ParsePacket<'b> for ConnPing {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let time = buf.take_time()?;
        Ok(Self { time })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConnPong {
    pub ping_time: SystemTime,
    pub pong_time: SystemTime,
}

impl<'b> ParsePacket<'b> for ConnPong {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        todo!()
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OConnReq1 {
    pub magic: Magic,
    /// protocol_version
    pub version: u8,
    /// size of mtu
    pub mtu_size: u16,
}

impl<'b> ParsePacket<'b> for OConnReq1 {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let magic = buf.take_magic()?;
        let version = buf.take_u8()?;
        // Panic(never): u16::max > udp max size
        let mtu_size = buf
            .len()
            .try_into()
            .map_err(|_| PacketError::InvalidMtuSize)?;
        Ok(Self {
            magic,
            version,
            mtu_size,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OConnReply1 {
    pub magic: Magic,
    pub server_guid: i64,
    pub security: SecurityState,
    pub mtu: u16,
}

impl<'b> ParsePacket<'b> for OConnReply1 {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let magic = buf.take_magic()?;
        let server_guid = buf.take_i64()?;
        let security = SecurityState::take(buf)?;
        let mtu = buf.take_u16()?;
        Ok(Self {
            magic,
            server_guid,
            security,
            mtu,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SecurityState {
    Raw,
    Encrypt,
}

impl<'b> ParsePacket<'b> for SecurityState {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let use_security = buf.take_bool()?;
        let ret = match use_security {
            true => Self::Encrypt,
            false => Self::Raw,
        };
        Ok(ret)
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OConnReq2 {
    pub magic: Magic,
    pub server_addr: SocketAddr,
    pub mtu: u16,
    pub client_guid: i64,
}

impl<'b> ParsePacket<'b> for OConnReq2 {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let magic = buf.take_magic()?;
        let server_addr = buf.take_address()?;
        let mtu = buf.take_u16()?;
        let client_guid = buf.take_i64()?;
        Ok(Self {
            magic,
            server_addr,
            mtu,
            client_guid,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct OConnReply2 {
    pub magic: Magic,
    pub server_guid: i64,
    pub client_addr: SocketAddr,
    pub mtu: u16,
    pub security: SecurityState,
}

impl<'b> ParsePacket<'b> for OConnReply2 {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let magic = buf.take_magic()?;
        let server_guid = buf.take_i64()?;
        let client_addr = buf.take_address()?;
        let mtu = buf.take_u16()?;
        let security = SecurityState::take(buf)?;
        Ok(Self {
            magic,
            server_guid,
            client_addr,
            mtu,
            security,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConnReq {}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConnReqAccept {}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NewConn {}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DisConn {}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Incompatible {}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FrameSet<'p> {
    pub sequence: u24,
    pub frame: Frame<'p>,
}

impl<'p, 'b: 'p> ParsePacket<'b> for FrameSet<'p> {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let sequence = buf.take_u24()?;
        let frame = Frame::take(buf)?;
        Ok(Self { sequence, frame })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Frame<'p> {
    pub flag: Flag,
    pub bit_len: u16,
    pub reliable_index: Option<u24>,
    pub sequence_index: Option<u24>,
    pub order: Option<Order>,
    pub fragment: Option<Fragment>,
    pub body: &'p [u8],
}

impl<'p, 'b: 'p> ParsePacket<'b> for Frame<'p> {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let flag = Flag::take(buf)?;
        let bit_len = buf.take_u16()?;
        let (mut reliable_index, mut sequence_index, mut order, mut fragment) = Default::default();
        if flag.is_reliable {
            reliable_index = Some(buf.take_u24()?);
        }
        if flag.is_sequence {
            sequence_index = Some(buf.take_u24()?);
        }
        if flag.is_order {
            order = Some(Order::take(buf)?);
        }
        if flag.is_fragment {
            fragment = Some(Fragment::take(buf)?)
        }
        let body = buf.take_bytes((bit_len / 8) as usize)?;
        Ok(Self {
            flag,
            bit_len,
            reliable_index,
            sequence_index,
            order,
            fragment,
            body,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Flag {
    pub is_reliable: bool,
    pub is_order: bool,
    pub is_sequence: bool,
    pub need_ack: bool,
    pub is_fragment: bool,
}

impl<'b> ParsePacket<'b> for Flag {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let mut byte = buf.take_u8()? >> 4;
        let is_fragment = byte & 0b0001 != 0;
        byte >>= 1;
        let [is_reliable, is_order, is_sequence, need_ack] = match byte {
            0 => [false, false, false, false],
            1 => [false, true, true, false],
            2 => [true, false, false, false],
            3 => [true, true, false, false],
            4 => [true, true, true, false],
            5 => [false, false, false, true],
            6 => [true, false, false, true],
            7 => [true, true, false, true],
            _ => unreachable!(),
        };
        Ok(Self {
            is_reliable,
            is_order,
            is_sequence,
            need_ack,
            is_fragment,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Order {
    pub index: u24,
    pub channel: u8,
}

impl<'b> ParsePacket<'b> for Order {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let index = buf.take_u24()?;
        let channel = buf.take_u8()?;
        Ok(Self { index, channel })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Fragment {
    pub compound_size: u32,
    pub compound_id: u16,
    pub index: u32,
}

impl<'b> ParsePacket<'b> for Fragment {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let size = buf.take_u32()?;
        let id = buf.take_u16()?;
        let index = buf.take_u32()?;
        Ok(Self {
            compound_size: size,
            compound_id: id,
            index,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Game {}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Nack {
    pub record_count: u16,
    pub record: Record,
}

impl<'b> ParsePacket<'b> for Nack {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let record_count = buf.take_u16()?;
        let record = Record::take(buf)?;
        Ok(Self {
            record_count,
            record,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Ack {
    pub record_count: u16,
    pub record: Record,
}

impl<'b> ParsePacket<'b> for Ack {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let record_count = buf.take_u16()?;
        let record = Record::take(buf)?;
        Ok(Self {
            record_count,
            record,
        })
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Record {
    Single(u24),
    Range(RangeInclusive<u24>),
}

impl<'b> ParsePacket<'b> for Record {
    type Error = PacketError;

    fn take<const N: usize>(buf: &'b mut SizedBuffer<N>) -> Result<Self, Self::Error> {
        let is_single = buf.take_bool()?;
        let ret = match is_single {
            true => {
                let num = buf.take_u24()?;
                Self::Single(num)
            }
            false => {
                let begin = buf.take_u24()?;
                let end = buf.take_u24()?;
                Self::Range(begin..=end)
            }
        };
        Ok(ret)
    }

    fn write<const N: usize>(self, buf: &mut SizedBuffer<N>) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum PacketError {
    #[error("data error")]
    DataError(#[from] DataError),

    #[error("unknown packet id")]
    UnknownPacket,

    #[error("invalid mtu size in udp packet")]
    InvalidMtuSize,
}

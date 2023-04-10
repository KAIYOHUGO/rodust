use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    ops::RangeInclusive,
};
use thiserror::Error;
use zeco::*;

type Str<'s> = PrefixLen<'s, u16>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct U24(u32);

impl<'de> Deserialize<'de> for U24 {
    type Error = zeco::des::Error;
    type Arg<'arg> = ();

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        _: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let bytes: &[u8; 3] = Deserialize::deserialize(buf, offset, ())?;
        let mut full_bytes = [0u8; 4];
        full_bytes[..3].copy_from_slice(bytes.as_ref());
        Ok(Self(u32::from_le_bytes(full_bytes)))
    }
}

impl TryTo<u32> for U24 {
    type Error = zeco::des::Error;

    fn try_to(self) -> Result<u32, Self::Error> {
        Ok(self.0)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[zeco(error = PacketError)]
pub struct Magic<'s>(&'s [u8; 16]);

#[derive(PartialEq, Eq, Debug, Clone)]
struct Addr(SocketAddr);

impl<'de> Deserialize<'de> for Addr {
    type Error = PacketError;

    type Arg<'arg> = ();

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        _: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let ipv: IPV = Deserialize::deserialize(buf, offset, ())?;
        let addr = match ipv {
            IPV::V4 => {
                let ip: &[u8; 4] = Deserialize::deserialize(buf, offset, ())?;
                let port: u16 = Deserialize::deserialize(buf, offset, BE)?;
                SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from(ip.clone()), port))
            }
            IPV::V6 => {
                *offset += 2;
                let port: u16 = Deserialize::deserialize(buf, offset, LE)?;
                *offset += 4;
                let ip: &[u8; 16] = Deserialize::deserialize(buf, offset, ())?;
                *offset += 4;
                SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from(ip.clone()), port, 0, 0))
            }
        };
        Ok(Self(addr))
    }
}

impl TryTo<SocketAddr> for Addr {
    type Error = PacketError;

    fn try_to(self) -> Result<SocketAddr, Self::Error> {
        Ok(self.0)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize)]
#[zeco(error = PacketError)]
enum IPV {
    V4 = 4,
    V6 = 6,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize)]
#[zeco(error = PacketError)]
pub enum PacketId {
    UConnPing = 0x01,
    UConnConnPing = 0x02,
    UConnPong = 0x1c,
    OConnReq1 = 0x05,
    OConnReply1 = 0x06,
    OConnReq2 = 0x07,
    OConnReply2 = 0x08,
    Incompatible = 0x19,
    /// [Read more about bit flag](https://github.com/pmmp/RakLib/blob/8e6ba0541ac24b20b4da446ee272ae3699a4c1b1/src/protocol/Datagram.php#L24-L30)
    #[zeco(tag = 0x80..=0x8d)]
    FrameSet = 0x80,
    Nack = 0xa0,
    Ack = 0xc0,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize)]
#[zeco(error = PacketError)]
pub enum FramePacketId {
    ConnReq = 0x09,
    ConnReqAccept = 0x10,
    ConnPing = 0x00,
    ConnPong = 0x03,
    NewConn = 0x13,
    DisConn = 0x15,
    Game = 0xfe,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[zeco(error = PacketError)]
pub struct UConnPing<'s> {
    #[zeco(arg = BE)]
    pub time: i64,
    pub magic: Magic<'s>,
    #[zeco(arg = BE)]
    pub client_guid: u64,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[zeco(error = PacketError)]
pub struct UConnConnPing<'s> {
    #[zeco(arg = BE)]
    pub time: i64,
    pub magic: Magic<'s>,
    #[zeco(arg = BE)]
    pub client_guid: u64,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[zeco(error = PacketError)]
pub struct UConnPong<'s> {
    #[zeco(arg = BE)]
    pub time: i64,
    #[zeco(arg = BE)]
    pub server_guid: u64,
    pub magic: Magic<'s>,
    #[zeco(with = Str<'s>, arg = BE)]
    pub server_id: &'s str,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct ConnPing {
    #[zeco(arg = BE)]
    pub time: i64,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct ConnPong {
    #[zeco(arg = BE)]
    pub ping_time: i64,
    #[zeco(arg = BE)]
    pub pong_time: i64,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct OConnReq1<'s> {
    pub magic: Magic<'s>,
    /// protocol_version
    pub version: u8,
    /// size of mtu
    #[zeco(arg = All)]
    pub mtu: &'s [u8],
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct OConnReply1<'s> {
    pub magic: Magic<'s>,
    #[zeco(arg = BE)]
    pub server_guid: u64,
    pub security: SecurityState,
    #[zeco(arg = BE)]
    pub mtu: u16,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub enum SecurityState {
    Raw = 0x00,
    Encrypt = 0x01,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct OConnReq2<'s> {
    pub magic: Magic<'s>,
    #[zeco(with = Addr)]
    pub server_addr: SocketAddr,
    #[zeco(arg = BE)]
    pub mtu: u16,
    #[zeco(arg = BE)]
    pub client_guid: u64,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct OConnReply2<'s> {
    pub magic: Magic<'s>,
    #[zeco(arg = BE)]
    pub server_guid: u64,
    #[zeco(with = Addr)]
    pub client_addr: SocketAddr,
    #[zeco(arg = BE)]
    pub mtu: u16,
    pub security: SecurityState,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct ConnReq {
    #[zeco(arg = BE)]
    pub guid: u64,
    #[zeco(arg = BE)]
    pub time: i64,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct ConnReqAccept {
    #[zeco(with = Addr)]
    pub client_addr: SocketAddr,
    #[zeco(arg = BE)]
    pub system_index: i16,
    #[zeco(with = [Addr; 10])]
    pub internal_addrs: [SocketAddr; 10],
    #[zeco(arg = BE)]
    pub req_time: i64,
    #[zeco(arg = BE)]
    pub time: i64,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct NewConn {
    #[zeco(with = Addr)]
    pub server_addr: SocketAddr,
    #[zeco(with = Addr)]
    pub internal_addr: SocketAddr,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct Incompatible<'p> {
    pub protocol: u8,
    pub magic: Magic<'p>,
    #[zeco(arg = BE)]
    pub server_guid: u64,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct FrameSet<'p> {
    #[zeco(with = U24)]
    pub sequence: u32,
    pub frame: Frame<'p>,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct Frame<'p> {
    pub flag: Flag,
    #[zeco(arg = BE)]
    pub bit_len: u16,
    #[zeco(if = flag.is_reliable, with = U24)]
    pub reliable_index: Option<u32>,
    #[zeco(if = flag.is_sequence, with = U24)]
    pub sequence_index: Option<u32>,
    #[zeco(if = flag.is_order)]
    pub order: Option<Order>,
    #[zeco(if = flag.is_fragment)]
    pub fragment: Option<Fragment>,
    #[zeco(arg = Len((bit_len / 8) as usize))]
    pub body: &'p [u8],
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Flag {
    pub is_reliable: bool,
    pub is_order: bool,
    pub is_sequence: bool,
    pub need_ack: bool,
    pub is_fragment: bool,
}

impl<'de> Deserialize<'de> for Flag {
    type Error = PacketError;

    type Arg<'arg> = ();

    fn deserialize<'arg>(
        buf: &'de [u8],
        offset: &mut usize,
        _: Self::Arg<'arg>,
    ) -> Result<Self, Self::Error> {
        let mut flag: u8 = Deserialize::deserialize(buf, offset, ())?;
        // 0b????
        flag >>= 4;

        let is_fragment = flag & 0b0001 != 0;
        // 0b???
        flag >>= 1;
        let [is_reliable, is_order, is_sequence, need_ack] = match flag {
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
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
#[zeco(error = PacketError)]
pub struct Order {
    #[zeco(with = U24)]
    pub index: u32,
    pub channel: u8,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct Fragment {
    #[zeco(arg = BE)]
    pub compound_size: u32,
    #[zeco(arg = BE)]
    pub compound_id: u16,
    #[zeco(arg = BE)]
    pub index: u32,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct Nack {
    #[zeco(arg = BE)]
    pub record_count: u16,
    pub record: Record,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct Ack {
    #[zeco(arg = BE)]
    pub record_count: u16,
    pub record: Record,
}

#[repr(u16)]
#[derive(PartialEq, Eq, Debug, Clone, Deserialize)]
pub enum Record {
    Range(#[zeco(with = [U24; 2])] RangeInclusive<u32>) = 0x00,
    Single(#[zeco(with = U24)] u32) = 0x01,
}

#[derive(Debug, Error)]
pub enum PacketError {
    #[error("data error")]
    DataError(#[from] zeco::des::Error),

    #[error("unknown packet id")]
    UnknownPacket,

    #[error("invalid mtu size in udp packet")]
    InvalidMtuSize,
}

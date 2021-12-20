use bytes::{Buf, BufMut};
use std::fmt;

use super::{
    coding::{BufExt, BufMutExt, Decode, Encode, UnexpectedEnd},
    varint::VarInt,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StreamType(u64);

macro_rules! stream_types {
    {$($name:ident = $val:expr,)*} => {
        impl StreamType {
            $(pub const $name: StreamType = StreamType($val);)*
        }
    }
}

stream_types! {
    CONTROL = 0x00,
    PUSH = 0x01,
    ENCODER = 0x02,
    DECODER = 0x03,
}

impl StreamType {
    pub const MAX_ENCODED_SIZE: usize = VarInt::MAX_SIZE;

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Decode for StreamType {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, UnexpectedEnd> {
        Ok(StreamType(buf.get_var()?))
    }
}

impl Encode for StreamType {
    fn encode<W: BufMut>(&self, buf: &mut W) {
        buf.write_var(self.0);
    }
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &StreamType::CONTROL => write!(f, "Control"),
            &StreamType::ENCODER => write!(f, "Encoder"),
            &StreamType::DECODER => write!(f, "Decoder"),
            x => write!(f, "StreamType({})", x.0),
        }
    }
}

/// Identifier for a stream
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StreamId(
    #[cfg(not(any(test, feature = "test_helpers")))] u64,
    #[cfg(any(test, feature = "test_helpers"))] pub u64,
);

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let initiator = match self.initiator() {
            Side::Client => "client",
            Side::Server => "server",
        };
        let dir = match self.dir() {
            Dir::Uni => "uni",
            Dir::Bi => "bi",
        };
        write!(
            f,
            "{} {}directional stream {}",
            initiator,
            dir,
            self.index()
        )
    }
}

impl StreamId {
    /// Distinguishes streams of the same initiator and directionality
    pub fn index(self) -> u64 {
        self.0 >> 2
    }

    pub fn is_request(&self) -> bool {
        self.dir() == Dir::Bi && self.initiator() == Side::Client
    }

    pub fn is_push(&self) -> bool {
        self.dir() == Dir::Uni && self.initiator() == Side::Server
    }

    /// Create a new StreamId
    /// Which side of a connection initiated the stream
    fn initiator(self) -> Side {
        if self.0 & 0x1 == 0 {
            Side::Client
        } else {
            Side::Server
        }
    }
    /// Which directions data flows in
    fn dir(self) -> Dir {
        if self.0 & 0x2 == 0 {
            Dir::Bi
        } else {
            Dir::Uni
        }
    }
}

impl From<u64> for StreamId {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl Encode for StreamId {
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        VarInt::from_u64(self.0).unwrap().encode(buf);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Side {
    /// The initiator of a connection
    Client = 0,
    /// The acceptor of a connection
    Server = 1,
}

/// Whether a stream communicates data in both directions or only from the initiator
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Dir {
    /// Data flows in both directions
    Bi = 0,
    /// Data flows only from the stream's initiator
    Uni = 1,
}


pub struct FixedHeader {
    type_: PacketType,
    flags: u8,
    remaining_length: u32,
}

pub enum PacketType {
    Reserved,
    Connect,
    ConnAck,
    Publish,
    PubAck,
    PubRec,
    PubRel,
    PubComp,
    Subscribe,
    SubAck,
    Unsubscribe,
    UnsubAck,
    PingReq,
    PingResp,
    Disconnect,
    Auth
}

impl PacketType {
    /// Convert to the raw 4-bit unsigned value that represents the given type.
    pub fn to_bits(&self) -> u8 {
        match self {
            PacketType::Reserved => 0,
            PacketType::Connect => 1,
            PacketType::ConnAck => 2,
            PacketType::Publish => 3,
            PacketType::PubAck => 4,
            PacketType::PubRec => 5,
            PacketType::PubRel => 6,
            PacketType::PubComp => 7,
            PacketType::Subscribe => 8,
            PacketType::SubAck => 9,
            PacketType::Unsubscribe => 10,
            PacketType::UnsubAck => 11,
            PacketType::PingReq => 12,
            PacketType::PingResp => 13,
            PacketType::Disconnect => 14,
            PacketType::Auth => 15,
        }
    }

    /// Get the [`PacketType`] that the given bits represent.
    /// 
    /// Bits in the upper half of the given bytes are discarded.
    pub fn from_bits(bits: u8) -> Self {
        let bits = bits & 0b00001111;

        match bits {
            0 => PacketType::Reserved,
            1 => PacketType::Connect,
            2 => PacketType::ConnAck,
            3 => PacketType::Publish,
            4 => PacketType::PubAck,
            5 => PacketType::PubRec,
            6 => PacketType::PubRel,
            7 => PacketType::PubComp,
            8 => PacketType::Subscribe,
            9 => PacketType::SubAck,
            10 => PacketType::Unsubscribe,
            11 => PacketType::UnsubAck,
            12 => PacketType::PingReq,
            13 => PacketType::PingResp,
            14 => PacketType::Disconnect,
            15 => PacketType::Auth,
            _ => unreachable!("Upper half of byte should be zero"),
        }
    }
}

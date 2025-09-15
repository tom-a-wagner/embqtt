//! This module deals with the MQTT fixed header and its fields.

use crate::{error::Error, packet::data_representation};
use embedded_io_async::{Read, Write};

#[derive(Debug)]
pub struct FixedHeader {
    type_: PacketType,
    flags: u8,
    remaining_length: u32,
}

impl FixedHeader {
    pub async fn read<R: Read>(input: &mut R) -> Result<Self, Error<R::Error>> {
        let control_byte = data_representation::read_u8(input).await?;
        let type_ = PacketType::from_bits(control_byte >> 4);
        let flags = control_byte & 0b0000_1111;
        let remaining_length = data_representation::read_variable_byte_integer(input).await?;

        Ok(Self {
            type_,
            flags,
            remaining_length,
        })
    }

    pub async fn write<W: Write>(&self, output: &mut W) -> Result<(), Error<W::Error>> {
        let control_byte = (self.type_.to_bits() << 4) | (self.flags & 0b0000_1111);
        data_representation::write_u8(control_byte, output).await?;
        data_representation::write_variable_byte_integer(self.remaining_length, output).await
    }
}

#[derive(Debug)]
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
    Auth,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_type_to_bits() {
        assert_eq!(PacketType::Reserved.to_bits(), 0);
        assert_eq!(PacketType::Connect.to_bits(), 1);
        assert_eq!(PacketType::ConnAck.to_bits(), 2);
        assert_eq!(PacketType::Publish.to_bits(), 3);
        assert_eq!(PacketType::PubAck.to_bits(), 4);
        assert_eq!(PacketType::PubRec.to_bits(), 5);
        assert_eq!(PacketType::PubRel.to_bits(), 6);
        assert_eq!(PacketType::PubComp.to_bits(), 7);
        assert_eq!(PacketType::Subscribe.to_bits(), 8);
        assert_eq!(PacketType::SubAck.to_bits(), 9);
        assert_eq!(PacketType::Unsubscribe.to_bits(), 10);
        assert_eq!(PacketType::UnsubAck.to_bits(), 11);
        assert_eq!(PacketType::PingReq.to_bits(), 12);
        assert_eq!(PacketType::PingResp.to_bits(), 13);
        assert_eq!(PacketType::Disconnect.to_bits(), 14);
        assert_eq!(PacketType::Auth.to_bits(), 15);
    }

    #[test]
    fn test_packet_type_from_bits() {
        assert!(matches!(PacketType::from_bits(0), PacketType::Reserved));
        assert!(matches!(PacketType::from_bits(1), PacketType::Connect));
        assert!(matches!(PacketType::from_bits(2), PacketType::ConnAck));
        assert!(matches!(PacketType::from_bits(3), PacketType::Publish));
        assert!(matches!(PacketType::from_bits(4), PacketType::PubAck));
        assert!(matches!(PacketType::from_bits(5), PacketType::PubRec));
        assert!(matches!(PacketType::from_bits(6), PacketType::PubRel));
        assert!(matches!(PacketType::from_bits(7), PacketType::PubComp));
        assert!(matches!(PacketType::from_bits(8), PacketType::Subscribe));
        assert!(matches!(PacketType::from_bits(9), PacketType::SubAck));
        assert!(matches!(PacketType::from_bits(10), PacketType::Unsubscribe));
        assert!(matches!(PacketType::from_bits(11), PacketType::UnsubAck));
        assert!(matches!(PacketType::from_bits(12), PacketType::PingReq));
        assert!(matches!(PacketType::from_bits(13), PacketType::PingResp));
        assert!(matches!(PacketType::from_bits(14), PacketType::Disconnect));
        assert!(matches!(PacketType::from_bits(15), PacketType::Auth));
    }

    #[test]
    fn test_packet_type_from_bits_ignores_upper_bits() {
        // Upper bits should be ignored
        assert!(matches!(
            PacketType::from_bits(0b11110001),
            PacketType::Connect
        ));
        assert!(matches!(
            PacketType::from_bits(0b10100010),
            PacketType::ConnAck
        ));
        assert!(matches!(
            PacketType::from_bits(0b01011111),
            PacketType::Auth
        ));
    }

    #[tokio::test]
    async fn test_fixed_header_read_success() {
        // Publish packet (type=3) with flags=0b1101, remaining_length=127
        let data = [0b00111101, 0x7F];
        let mut reader = &data[..];

        let header = FixedHeader::read(&mut reader).await.unwrap();
        assert!(matches!(header.type_, PacketType::Publish));
        assert_eq!(header.flags, 0b1101);
        assert_eq!(header.remaining_length, 127);
    }

    #[tokio::test]
    async fn test_fixed_header_read_eof() {
        let data = [];
        let mut reader = &data[..];

        let result = FixedHeader::read(&mut reader).await;
        assert!(matches!(result, Err(Error::MalformedPacket)));
    }

    // Tests for FixedHeader::write()
    #[tokio::test]
    async fn test_fixed_header_write_success() {
        let header = FixedHeader {
            type_: PacketType::Publish,
            flags: 0b1101,
            remaining_length: 127,
        };

        let mut buffer = [0u8; 2];
        let mut writer = &mut buffer[..];

        header.write(&mut writer).await.unwrap();
        assert_eq!(buffer, [0b00111101, 0x7F]);
    }

    #[tokio::test]
    async fn test_fixed_header_write_buffer_too_small() {
        let header = FixedHeader {
            type_: PacketType::Connect,
            flags: 0,
            remaining_length: 128, // Needs 3 bytes total
        };

        let mut buffer = [0u8; 2]; // Too small
        let mut writer = &mut buffer[..];

        let result = header.write(&mut writer).await;
        assert!(matches!(result, Err(Error::NetworkError(_))));
    }
}

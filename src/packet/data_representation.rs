//! This module contains functions for writing and reading the different basic types
//! present in an MQTT control packet.

pub use crate::error::Error;
pub use embedded_io_async::{ErrorType, Read, Write};

const VARINT_CONTINUATION_BIT_MASK: u8 = 0b1000_0000;

pub async fn read_u8<R: Read>(input: &mut R) -> Result<u8, Error<R::Error>> {
    let mut buf = [0u8; 1];
    input.read_exact(&mut buf).await?;
    Ok(buf[0])
}

pub async fn read_u16<R: Read>(input: &mut R) -> Result<u16, Error<R::Error>> {
    let mut buf = [0u8; 2];
    input.read_exact(&mut buf).await?;
    Ok(u16::from_be_bytes(buf))
}

pub async fn read_u32<R: Read>(input: &mut R) -> Result<u32, Error<R::Error>> {
    let mut buf = [0u8; 4];
    input.read_exact(&mut buf).await?;
    Ok(u32::from_be_bytes(buf))
}

pub async fn read_variable_byte_integer<R: Read>(input: &mut R) -> Result<u32, Error<R::Error>> {
    let mut buf = [0u8; 1];

    // The following algorithm is adapted from MQTT5 specification section 1.5.5
    let mut multiplier = 1u32;
    let mut value = 0u32;

    loop {
        input.read_exact(&mut buf).await?;
        let encoded_byte = buf[0];
        value += u32::from(encoded_byte & !VARINT_CONTINUATION_BIT_MASK) * multiplier;

        if encoded_byte & VARINT_CONTINUATION_BIT_MASK == 0 {
            // Continuation bit is not set, this is the last byte.
            break;
        }

        multiplier *= 128;
        if multiplier > 128 * 128 * 128 {
            // This would be the 5th byte, but the specification allows four bytes maximum.
            return Err(Error::MalformedPacketError);
        }
    }

    Ok(value)
}

pub async fn write_u8<W: Write>(num: u8, output: &mut W) -> Result<(), Error<W::Error>> {
    output
        .write_all(&[num])
        .await
        .map_err(|e| Error::NetworkError(e))
}

pub async fn write_u16<W: Write>(num: u16, output: &mut W) -> Result<(), Error<W::Error>> {
    output
        .write_all(&num.to_be_bytes())
        .await
        .map_err(|e| Error::NetworkError(e))
}

pub async fn write_u32<W: Write>(num: u32, output: &mut W) -> Result<(), Error<W::Error>> {
    output
        .write_all(&num.to_be_bytes())
        .await
        .map_err(|e| Error::NetworkError(e))
}

pub async fn write_variable_byte_integer<W: Write>(
    mut num: u32,
    output: &mut W,
) -> Result<(), Error<W::Error>> {
    // The following algorithm is adapted from MQTT5 specification section 1.5.5

    loop {
        let mut encoded_byte: u8 = (num % 128)
            .try_into()
            .expect("num % 128 should fit into a u8");
        num /= 128;

        // If we have more bits of `num` to encode, set continuation bit
        if num > 0 {
            encoded_byte |= VARINT_CONTINUATION_BIT_MASK;
        }

        output
            .write_all(&[encoded_byte])
            .await
            .map_err(|e| Error::NetworkError(e))?;

        if num == 0 {
            // All bits encoded, we are done.
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_u8_success() {
        let data = [0x42];
        let mut reader = &data[..];
        let result = read_u8(&mut reader).await.unwrap();
        assert_eq!(result, 0x42);
    }

    #[tokio::test]
    async fn test_read_u8_eof() {
        let data = [];
        let mut reader = &data[..];
        let result = read_u8(&mut reader).await;
        assert!(matches!(result, Err(Error::MalformedPacketError)));
    }

    #[tokio::test]
    async fn test_read_u16_success() {
        let data = [0x12, 0x34];
        let mut reader = &data[..];
        let result = read_u16(&mut reader).await.unwrap();
        assert_eq!(result, 0x1234);
    }

    #[tokio::test]
    async fn test_read_u16_eof() {
        let data = [0x12];
        let mut reader = &data[..];
        let result = read_u16(&mut reader).await;
        assert!(matches!(result, Err(Error::MalformedPacketError)));
    }

    #[tokio::test]
    async fn test_read_u32_success() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let mut reader = &data[..];
        let result = read_u32(&mut reader).await.unwrap();
        assert_eq!(result, 0x12345678);
    }

    #[tokio::test]
    async fn test_read_u32_eof() {
        let data = [0x12, 0x34, 0x56];
        let mut reader = &data[..];
        let result = read_u32(&mut reader).await;
        assert!(matches!(result, Err(Error::MalformedPacketError)));
    }

    #[tokio::test]
    async fn test_read_variable_byte_integer_single_byte() {
        let data = [0x7F]; // 127
        let mut reader = &data[..];
        let result = read_variable_byte_integer(&mut reader).await.unwrap();
        assert_eq!(result, 127);
    }

    #[tokio::test]
    async fn test_read_variable_byte_integer_two_bytes() {
        let data = [0x80, 0x01]; // 128
        let mut reader = &data[..];
        let result = read_variable_byte_integer(&mut reader).await.unwrap();
        assert_eq!(result, 128);
    }

    #[tokio::test]
    async fn test_read_variable_byte_integer_three_bytes() {
        let data = [0x80, 0x80, 0x01]; // 16384
        let mut reader = &data[..];
        let result = read_variable_byte_integer(&mut reader).await.unwrap();
        assert_eq!(result, 16384);
    }

    #[tokio::test]
    async fn test_read_variable_byte_integer_four_bytes() {
        let data = [0x80, 0x80, 0x80, 0x01]; // 2097152
        let mut reader = &data[..];
        let result = read_variable_byte_integer(&mut reader).await.unwrap();
        assert_eq!(result, 2097152);
    }

    #[tokio::test]
    async fn test_read_variable_byte_integer_max_value() {
        let data = [0xFF, 0xFF, 0xFF, 0x7F]; // 268435455
        let mut reader = &data[..];
        let result = read_variable_byte_integer(&mut reader).await.unwrap();
        assert_eq!(result, 268435455);
    }

    #[tokio::test]
    async fn test_read_variable_byte_integer_zero() {
        let data = [0x00];
        let mut reader = &data[..];
        let result = read_variable_byte_integer(&mut reader).await.unwrap();
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_read_variable_byte_integer_too_many_bytes() {
        let data = [0x80, 0x80, 0x80, 0x80, 0x01];
        let mut reader = &data[..];
        let result = read_variable_byte_integer(&mut reader).await;
        assert!(matches!(result, Err(Error::MalformedPacketError)));
    }

    #[tokio::test]
    async fn test_read_variable_byte_integer_eof() {
        let data = [0x80]; // Continuation bit set but no next byte
        let mut reader = &data[..];
        let result = read_variable_byte_integer(&mut reader).await;
        assert!(matches!(result, Err(Error::MalformedPacketError)));
    }

    #[tokio::test]
    async fn test_write_u8_success() {
        let mut buffer = [0u8; 1];
        let mut writer = &mut buffer[..];
        write_u8(0x42, &mut writer).await.unwrap();
        assert_eq!(buffer, [0x42]);
    }

    #[tokio::test]
    async fn test_write_u8_buffer_too_small() {
        let mut buffer = [];
        let mut writer = &mut buffer[..];
        let result = write_u8(0x42, &mut writer).await;
        assert!(matches!(result, Err(Error::NetworkError(_))));
    }

    #[tokio::test]
    async fn test_write_u16_success() {
        let mut buffer = [0u8; 2];
        let mut writer = &mut buffer[..];
        write_u16(0x1234, &mut writer).await.unwrap();
        assert_eq!(buffer, [0x12, 0x34]);
    }

    #[tokio::test]
    async fn test_write_u16_buffer_too_small() {
        let mut buffer = [0u8; 1];
        let mut writer = &mut buffer[..];
        let result = write_u16(0x1234, &mut writer).await;
        assert!(matches!(result, Err(Error::NetworkError(_))));
    }

    #[tokio::test]
    async fn test_write_u32_success() {
        let mut buffer = [0u8; 4];
        let mut writer = &mut buffer[..];
        write_u32(0x12345678, &mut writer).await.unwrap();
        assert_eq!(buffer, [0x12, 0x34, 0x56, 0x78]);
    }

    #[tokio::test]
    async fn test_write_u32_buffer_too_small() {
        let mut buffer = [0u8; 3];
        let mut writer = &mut buffer[..];
        let result = write_u32(0x12345678, &mut writer).await;
        assert!(matches!(result, Err(Error::NetworkError(_))));
    }

    #[tokio::test]
    async fn test_write_variable_byte_integer_single_byte() {
        let mut buffer = [0u8; 1];
        let mut writer = &mut buffer[..];
        write_variable_byte_integer(127, &mut writer).await.unwrap();
        assert_eq!(buffer, [0x7F]);
    }

    #[tokio::test]
    async fn test_write_variable_byte_integer_two_bytes() {
        let mut buffer = [0u8; 2];
        let mut writer = &mut buffer[..];
        write_variable_byte_integer(128, &mut writer).await.unwrap();
        assert_eq!(buffer, [0x80, 0x01]);
    }

    #[tokio::test]
    async fn test_write_variable_byte_integer_three_bytes() {
        let mut buffer = [0u8; 3];
        let mut writer = &mut buffer[..];
        write_variable_byte_integer(16384, &mut writer)
            .await
            .unwrap();
        assert_eq!(buffer, [0x80, 0x80, 0x01]);
    }

    #[tokio::test]
    async fn test_write_variable_byte_integer_four_bytes() {
        let mut buffer = [0u8; 4];
        let mut writer = &mut buffer[..];
        write_variable_byte_integer(2097152, &mut writer)
            .await
            .unwrap();
        assert_eq!(buffer, [0x80, 0x80, 0x80, 0x01]);
    }

    #[tokio::test]
    async fn test_write_variable_byte_integer_max_value() {
        let mut buffer = [0u8; 4];
        let mut writer = &mut buffer[..];
        write_variable_byte_integer(268435455, &mut writer)
            .await
            .unwrap();
        assert_eq!(buffer, [0xFF, 0xFF, 0xFF, 0x7F]);
    }

    #[tokio::test]
    async fn test_write_variable_byte_integer_zero() {
        let mut buffer = [u8::MAX; 1];
        let mut writer = &mut buffer[..];
        write_variable_byte_integer(0, &mut writer).await.unwrap();
        assert_eq!(buffer, [0x00]);
    }

    #[tokio::test]
    async fn test_write_variable_byte_integer_buffer_too_small() {
        let mut buffer = [0u8; 1];
        let mut writer = &mut buffer[..];
        let result = write_variable_byte_integer(128, &mut writer).await;
        assert!(matches!(result, Err(Error::NetworkError(_))));
    }

    #[tokio::test]
    async fn test_write_variable_byte_integer_boundary_values() {
        // Test values around byte boundaries
        let test_cases: &[(u32, &[u8])] = &[
            (0, &[0x00]),
            (127, &[0x7F]),
            (128, &[0x80, 0x01]),
            (16383, &[0xFF, 0x7F]),
            (16384, &[0x80, 0x80, 0x01]),
            (2097151, &[0xFF, 0xFF, 0x7F]),
            (2097152, &[0x80, 0x80, 0x80, 0x01]),
            (268435455, &[0xFF, 0xFF, 0xFF, 0x7F]),
        ];

        for &(value, expected) in test_cases {
            let mut buffer = [0u8; 4];
            let mut writer = &mut buffer[..];
            write_variable_byte_integer(value, &mut writer)
                .await
                .unwrap();
            assert_eq!(
                &buffer[..expected.len()],
                expected,
                "Failed for value {}",
                value
            );
        }
    }

    // Round-trip test for variable byte integer to ensure encoding/decoding consistency
    #[tokio::test]
    async fn test_variable_byte_integer_roundtrip() {
        let values = [
            0, 1, 127, 128, 255, 256, 16383, 16384, 32767, 65535, 2097151, 2097152, 268435455,
        ];

        for &value in &values {
            let mut buffer = [0u8; 4]; // Max size needed for variable byte integer
            let mut writer = &mut buffer[..];
            write_variable_byte_integer(value, &mut writer)
                .await
                .unwrap();

            let mut reader = &buffer[..];
            let read_value = read_variable_byte_integer(&mut reader).await.unwrap();
            assert_eq!(value, read_value, "Roundtrip failed for value {}", value);
        }
    }
}

use embedded_io_async::ReadExactError;

#[derive(Debug)]
pub enum Error<E> {
    MalformedPacketError,
    NetworkError(E),
}

impl<E> From<ReadExactError<E>> for Error<E> {
    fn from(value: ReadExactError<E>) -> Self {
        match value {
            // Connection was closed, without the entire packet being transmitted. Treat as malformed packet.
            embedded_io_async::ReadExactError::UnexpectedEof => Error::MalformedPacketError,
            embedded_io_async::ReadExactError::Other(e) => Error::NetworkError(e),
        }
    }
}

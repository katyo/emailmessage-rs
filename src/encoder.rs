use futures::{Stream, Poll, Async};
use quoted_printable;
use base64;
use header::{ContentTransferEncoding};
use {BinaryStream};

pub enum EncoderError<E> {
    Source(E),
    Coding,
}

pub trait EncoderCodec {
    fn encode_chunk(&mut self, chunk: Vec<u8>) -> Result<Vec<u8>, ()>;
}

/// 7bit codec
///
struct SevenBitCodec {
    line_wrapper: EightBitCodec,
}

impl SevenBitCodec {
    pub fn new() -> Self {
        SevenBitCodec { line_wrapper: EightBitCodec::new() }
    }
}

impl EncoderCodec for SevenBitCodec {
    fn encode_chunk(&mut self, chunk: Vec<u8>) -> Result<Vec<u8>, ()> {
        if chunk.iter().all(u8::is_ascii) {
            self.line_wrapper.encode_chunk(chunk)
        } else {
            Err(())
        }
    }
}

/// Quoted-Printable codec
///
struct QuotedPrintableCodec();

impl QuotedPrintableCodec {
    pub fn new() -> Self {
        QuotedPrintableCodec()
    }
}

impl EncoderCodec for QuotedPrintableCodec {
    fn encode_chunk(&mut self, chunk: Vec<u8>) -> Result<Vec<u8>, ()> {
        Ok(quoted_printable::encode(chunk))
    }
}

/// Base64 codec
///
struct Base64Codec();

impl Base64Codec {
    pub fn new() -> Self {
        Base64Codec()
    }
}

impl EncoderCodec for Base64Codec {
    fn encode_chunk(&mut self, chunk: Vec<u8>) -> Result<Vec<u8>, ()> {
        Ok(base64::encode_config(&chunk, base64::MIME).as_bytes().into())
    }
}

/// 8bit codec
///
struct EightBitCodec {
    line_bytes: usize,
}

impl EightBitCodec {
    pub fn new() -> Self {
        EightBitCodec { line_bytes: 0 }
    }
}

impl EncoderCodec for EightBitCodec {
    fn encode_chunk(&mut self, chunk: Vec<u8>) -> Result<Vec<u8>, ()> {
        // FIXME: correct line wrap
        //let line_break = chunk.iter().find(b'\n');
        let line_bytes = self.line_bytes + chunk.len();
        Ok(if line_bytes > 1000 - 2 {
            self.line_bytes = line_bytes - (1000 - 2);
            let (start, end) = chunk.split_at(self.line_bytes);
            let mut output = Vec::with_capacity(chunk.len() + 2);
            output.extend(start);
            output.extend(b"\r\n");
            output.extend(end);
            output
        } else {
            chunk
        })
    }
}

/// Binary codec
///
struct BinaryCodec();

impl BinaryCodec {
    pub fn new() -> Self {
        BinaryCodec()
    }
}

impl EncoderCodec for BinaryCodec {
    fn encode_chunk(&mut self, chunk: Vec<u8>) -> Result<Vec<u8>, ()> {
        Ok(chunk)
    }
}

pub struct EncoderChunk();

impl EncoderChunk {
    pub fn get(encoding: &ContentTransferEncoding) -> Box<EncoderCodec> {
        use self::ContentTransferEncoding::*;
        match *encoding {
            SevenBit => Box::new(SevenBitCodec::new()),
            QuotedPrintable => Box::new(QuotedPrintableCodec::new()),
            Base64 => Box::new(Base64Codec::new()),
            EightBit => Box::new(EightBitCodec::new()),
            Binary => Box::new(BinaryCodec::new()),
        }
    }
}

/// Generic data encoder
///
pub struct EncoderStream<S> {
    source: S,
    encoder: Box<EncoderCodec>,
}

impl<S> EncoderStream<S> {
    pub fn new(source: S, encoder: Box<EncoderCodec>) -> Self {
        EncoderStream { source, encoder }
    }

    pub fn wrap<E>(encoding: &ContentTransferEncoding, source: S) -> EncoderStream<S>
    where S: Stream<Item = Vec<u8>, Error = E> + 'static,
          E: 'static,
    {
        EncoderStream::new(source, EncoderChunk::get(encoding))
    }
}

impl<S, E> Stream for EncoderStream<S>
where S: Stream<Item = Vec<u8>, Error = E>,
{
    type Item = Vec<u8>;
    type Error = EncoderError<E>;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        //self.source.poll().map(|async| async.map(|packet| packet.map(|chunk| self.codec.encode_chunk(&chunk))))
        match self.source.poll() {
            Ok(Async::Ready(Some(chunk))) =>
                if let Ok(chunk) = self.encoder.encode_chunk(chunk) {
                    Ok(Async::Ready(Some(chunk)))
                } else {
                    Err(EncoderError::Coding)
                },
            Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(error) => Err(EncoderError::Source(error)),
        }
    }
}

/// Encoded binary stream
pub type EncodedBinaryStream<E> = BinaryStream<EncoderError<E>>;

#[cfg(test)]
mod test {
    use std::str::{from_utf8};
    use super::{EncoderCodec, SevenBitCodec, EightBitCodec, QuotedPrintableCodec, Base64Codec, BinaryCodec};

    #[test]
    fn seven_bit_encode() {
        let mut c = SevenBitCodec::new();

        assert_eq!(c.encode_chunk("Hello, world!".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("Hello, world!".into())));

        assert_eq!(c.encode_chunk("Hello, мир!".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Err(()));
    }

    #[test]
    fn quoted_printable_encode() {
        let mut c = QuotedPrintableCodec::new();

        assert_eq!(c.encode_chunk("Привет, мир!".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!".into())));

        assert_eq!(c.encode_chunk("Текст письма в уникоде".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5".into())));
    }

    #[test]
    fn base64_encode() {
        let mut c = Base64Codec::new();

        assert_eq!(c.encode_chunk("Привет, мир!".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("0J/RgNC40LLQtdGCLCDQvNC40YAh".into())));

        assert_eq!(c.encode_chunk("Текст письма в уникоде".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("0KLQtdC60YHRgiDQv9C40YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LU=".into())));
    }

    #[test]
    fn eight_bit_encode() {
        let mut c = EightBitCodec::new();

        assert_eq!(c.encode_chunk("Hello, world!".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("Hello, world!".into())));

        assert_eq!(c.encode_chunk("Hello, мир!".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("Hello, мир!".into())));
    }

    #[test]
    fn binary_encode() {
        let mut c = BinaryCodec::new();

        assert_eq!(c.encode_chunk("Hello, world!".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("Hello, world!".into())));

        assert_eq!(c.encode_chunk("Hello, мир!".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("Hello, мир!".into())));
    }
}

use base64;
use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use futures::{Async, Poll, Stream};
use header::ContentTransferEncoding;
use hyper::body::Payload;
use quoted_printable;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum EncoderError<E> {
    Source(E),
    Coding,
}

impl<E> Error for EncoderError<E> where E: fmt::Debug + fmt::Display {}

impl<E> fmt::Display for EncoderError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncoderError::Source(error) => write!(f, "Source error: {}", error),
            EncoderError::Coding => f.write_str("Coding error"),
        }
    }
}

pub trait EncoderCodec: Send {
    fn encode_chunk(&mut self, chunk: Bytes) -> Result<Bytes, ()>;
}

/// 7bit codec
///
struct SevenBitCodec {
    line_wrapper: EightBitCodec,
}

impl SevenBitCodec {
    pub fn new() -> Self {
        SevenBitCodec {
            line_wrapper: EightBitCodec::new(),
        }
    }
}

impl EncoderCodec for SevenBitCodec {
    fn encode_chunk(&mut self, chunk: Bytes) -> Result<Bytes, ()> {
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
    fn encode_chunk(&mut self, chunk: Bytes) -> Result<Bytes, ()> {
        Ok(quoted_printable::encode(chunk).into())
    }
}

/// Base64 codec
///
struct Base64Codec {
    line_wrapper: EightBitCodec,
}

impl Base64Codec {
    pub fn new() -> Self {
        Base64Codec {
            line_wrapper: EightBitCodec::new().with_limit(78 - 2),
        }
    }
}

impl EncoderCodec for Base64Codec {
    fn encode_chunk(&mut self, chunk: Bytes) -> Result<Bytes, ()> {
        let mut out = BytesMut::with_capacity(chunk.len() * 4 / 3 + 4);

        unsafe {
            let len = base64::encode_config_slice(&chunk, base64::STANDARD, out.bytes_mut());
            out.advance_mut(len);
        }

        self.line_wrapper.encode_chunk(out.freeze())
    }
}

/// 8bit codec
///
struct EightBitCodec {
    max_length: usize,
    line_bytes: usize,
}

const DEFAULT_MAX_LINE_LENGTH: usize = 1000 - 2;

impl EightBitCodec {
    pub fn new() -> Self {
        EightBitCodec {
            max_length: DEFAULT_MAX_LINE_LENGTH,
            line_bytes: 0,
        }
    }

    pub fn with_limit(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }
}

impl EncoderCodec for EightBitCodec {
    fn encode_chunk(&mut self, chunk: Bytes) -> Result<Bytes, ()> {
        let mut out = BytesMut::with_capacity(chunk.len() + 20);
        let mut src = chunk.into_buf();
        while src.has_remaining() {
            let line_break = src.bytes().iter().position(|b| *b == b'\n');
            let mut split_pos = if let Some(line_break) = line_break {
                line_break
            } else {
                src.remaining()
            };
            let max_length = self.max_length - self.line_bytes;
            if split_pos < max_length {
                // advance line bytes
                self.line_bytes += split_pos;
            } else {
                split_pos = max_length;
                // reset line bytes
                self.line_bytes = 0;
            };
            let has_remaining = split_pos < src.remaining();
            let mut taken = src.take(split_pos);
            out.reserve(split_pos + if has_remaining { 2 } else { 0 });
            out.put(&mut taken);
            if has_remaining {
                out.put_slice(b"\r\n");
            }
            src = taken.into_inner();
        }
        Ok(out.freeze())
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
    fn encode_chunk(&mut self, chunk: Bytes) -> Result<Bytes, ()> {
        Ok(chunk)
    }
}

pub struct EncoderChunk();

impl EncoderChunk {
    pub fn get(encoding: Option<&ContentTransferEncoding>) -> Box<EncoderCodec> {
        use self::ContentTransferEncoding::*;
        if let Some(encoding) = encoding {
            match encoding {
                SevenBit => Box::new(SevenBitCodec::new()),
                QuotedPrintable => Box::new(QuotedPrintableCodec::new()),
                Base64 => Box::new(Base64Codec::new()),
                EightBit => Box::new(EightBitCodec::new()),
                Binary => Box::new(BinaryCodec::new()),
            }
        } else {
            Box::new(BinaryCodec::new())
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

    pub fn wrap(encoding: Option<&ContentTransferEncoding>, source: S) -> EncoderStream<S>
    where
        S: Payload,
    {
        EncoderStream::new(source, EncoderChunk::get(encoding))
    }
}

impl<S> Stream for EncoderStream<S>
where
    S: Payload,
    S::Data: Into<Bytes>,
{
    type Item = Bytes;
    type Error = EncoderError<S::Error>;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.source.poll_data() {
            Ok(Async::Ready(Some(chunk))) => {
                if let Ok(chunk) = self.encoder.encode_chunk(chunk.into()) {
                    Ok(Async::Ready(Some(chunk)))
                } else {
                    Err(EncoderError::Coding)
                }
            }
            Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(error) => Err(EncoderError::Source(error)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        Base64Codec, BinaryCodec, EightBitCodec, EncoderCodec, QuotedPrintableCodec, SevenBitCodec,
    };
    use std::str::from_utf8;

    #[test]
    fn seven_bit_encode() {
        let mut c = SevenBitCodec::new();

        assert_eq!(
            c.encode_chunk("Hello, world!".as_bytes().into())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, world!".into()))
        );

        assert_eq!(
            c.encode_chunk("Hello, мир!".as_bytes().into())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Err(())
        );
    }

    #[test]
    fn quoted_printable_encode() {
        let mut c = QuotedPrintableCodec::new();

        assert_eq!(
            c.encode_chunk("Привет, мир!".as_bytes().into())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok(
                "=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!".into()
            ))
        );

        assert_eq!(c.encode_chunk("Текст письма в уникоде".as_bytes().into())
                   .map(|s| from_utf8(&s).map(|s| String::from(s))),
                   Ok(Ok("=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5".into())));
    }

    #[test]
    fn base64_encode() {
        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_chunk("Привет, мир!".as_bytes().into())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("0J/RgNC40LLQtdGCLCDQvNC40YAh".into()))
        );

        assert_eq!(
            c.encode_chunk(
                "Текст письма в уникоде подлиннее"
                    .as_bytes()
                    .into()
            ).map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok(concat!(
                "0KLQtdC60YHRgiDQv9C40YHRjNC80LAg0LIg0YPQvdC40LrQ\r\n",
                "vtC00LUg0L/QvtC00LvQuNC90L3QtdC1"
            ).into()))
        );
    }

    #[test]
    fn base64_encode_long() {
        let mut c = Base64Codec::new();

        assert_eq!(
            c.encode_chunk(
                "Ну прямо супер-длинный текст письма в уникоде, который уж точно ну никак не поместиться в 78 байт, как ни крути, я гарантирую это."
                    .as_bytes()
                    .into()
            ).map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok(
                concat!("0J3RgyDQv9GA0Y/QvNC+INGB0YPQv9C10YAt0LTQu9C40L3QvdGL0Lkg0YLQtdC60YHRgiDQv9C4\r\n",
                        "0YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LUsINC60L7RgtC+0YDRi9C5INGD0LYg0YLQvtGH0L3Q\r\n",
                        "viDQvdGDINC90LjQutCw0Log0L3QtSDQv9C+0LzQtdGB0YLQuNGC0YzRgdGPINCyIDc4INCx0LDQ\r\n",
                        "udGCLCDQutCw0Log0L3QuCDQutGA0YPRgtC4LCDRjyDQs9Cw0YDQsNC90YLQuNGA0YPRjiDRjdGC\r\n",
                        "0L4u").into()
            ))
        );
    }

    #[test]
    fn eight_bit_encode() {
        let mut c = EightBitCodec::new();

        assert_eq!(
            c.encode_chunk("Hello, world!".as_bytes().into())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, world!".into()))
        );

        assert_eq!(
            c.encode_chunk("Hello, мир!".as_bytes().into())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, мир!".into()))
        );
    }

    #[test]
    fn binary_encode() {
        let mut c = BinaryCodec::new();

        assert_eq!(
            c.encode_chunk("Hello, world!".as_bytes().into())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, world!".into()))
        );

        assert_eq!(
            c.encode_chunk("Hello, мир!".as_bytes().into())
                .map(|s| from_utf8(&s).map(|s| String::from(s))),
            Ok(Ok("Hello, мир!".into()))
        );
    }
}

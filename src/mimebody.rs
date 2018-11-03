use bytes::{BufMut, Bytes, BytesMut};
use encoder::{EncoderChunk, EncoderError, EncoderStream};
use futures::{Async, Poll, Stream};
use header::{ContentTransferEncoding, ContentType, Header, Headers};
use hyper::body::{Chunk, Payload};
use mime::Mime;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
use std::mem::replace;
use std::str::from_utf8;
use textnonce::TextNonce;
use Body;

/// MIME part variants
///
pub enum Part<B = Body> {
    /// Single part with content
    ///
    Single(SinglePart<B>),

    /// Multiple parts of content
    ///
    Multi(MultiPart<B>),
}

/*
impl<B> Default for Part<B> {
    fn default() -> Self {
        Part::Single(SinglePart::default())
    }
}
*/

impl<B> Display for Part<B>
where
    B: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Part::Single(ref part) => part.fmt(f),
            Part::Multi(ref part) => part.fmt(f),
        }
    }
}

impl<B> Part<B> {
    pub fn into_stream(self) -> PartStream<B>
    where
        B: Payload,
    {
        self.into()
    }
}

/// Part stream
pub enum PartStream<B> {
    /// Single part stream
    ///
    Single(SinglePartStream<B>),

    /// Multi part stream
    ///
    Multi(MultiPartStream<B>),
}

impl<B> Stream for PartStream<B>
where
    B: Payload,
    B::Data: Into<Bytes>,
{
    type Item = Bytes;
    type Error = EncoderError<B::Error>;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use self::PartStream::*;
        match self {
            Single(stream) => stream.poll(),
            Multi(stream) => stream.poll(),
        }
    }
}

/// Convert generic part into boxed stream of binary chunks
///
impl<B> From<Part<B>> for PartStream<B>
where
    B: Payload,
{
    fn from(this: Part<B>) -> Self {
        use self::PartStream::*;
        match this {
            Part::Single(part) => Single(part.into_stream()),
            Part::Multi(part) => Multi(part.into_stream()),
        }
    }
}

/// Parts of multipart body
///
pub type Parts<B = Body> = Vec<Part<B>>;

/// Creates builder for single part
///
pub struct SinglePartBuilder {
    headers: Headers,
}

impl SinglePartBuilder {
    /// Creates a default SinglePartBuilder
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
        }
    }

    /// Set a header and move the Part
    ///
    #[inline]
    pub fn header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set the body and move the Part
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn body<T>(self, body: T) -> SinglePart<T> {
        SinglePart {
            headers: self.headers,
            body,
        }
    }
}

impl Default for SinglePartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Single part
///
/// # Example
///
/// ```no_test
/// extern crate mime;
/// extern crate emailmessage;
///
/// use emailmessage::{SinglePart, header};
///
/// let part = SinglePart::builder()
///      .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
///      .header(header::ContentTransferEncoding::Binary)
///      .body("Текст письма в уникоде");
/// ```
///
pub struct SinglePart<B = Body> {
    headers: Headers,
    body: B,
}

impl SinglePart<()> {
    /// Creates a default SinglePartBuilder
    pub fn builder() -> SinglePartBuilder {
        SinglePartBuilder::new()
    }

    /// Creates a SinglePart with 7bit encoding
    ///
    /// Shortcut for `SinglePart::builder().header(ContentTransferEncoding::SevenBit)`.
    pub fn seven_bit() -> SinglePartBuilder {
        Self::builder().header(ContentTransferEncoding::SevenBit)
    }

    /// Creates a SinglePart with quoted-printable encoding
    ///
    /// Shortcut for `SinglePart::builder().header(ContentTransferEncoding::QuotedPrintable)`.
    pub fn quoted_printable() -> SinglePartBuilder {
        Self::builder().header(ContentTransferEncoding::QuotedPrintable)
    }

    /// Creates a SinglePart with base64 encoding
    ///
    /// Shortcut for `SinglePart::builder().header(ContentTransferEncoding::Base64)`.
    pub fn base64() -> SinglePartBuilder {
        Self::builder().header(ContentTransferEncoding::Base64)
    }

    /// Creates a SinglePart with 8-bit encoding
    ///
    /// Shortcut for `SinglePart::builder().header(ContentTransferEncoding::EightBit)`.
    #[inline]
    pub fn eight_bit() -> SinglePartBuilder {
        Self::builder().header(ContentTransferEncoding::EightBit)
    }

    /// Creates a SinglePart with binary encoding
    ///
    /// Shortcut for `SinglePart::builder().header(ContentTransferEncoding::Binary)`.
    #[inline]
    pub fn binary() -> SinglePartBuilder {
        Self::builder().header(ContentTransferEncoding::Binary)
    }
}

impl<B> SinglePart<B> {
    /// Get the transfer encoding
    #[inline]
    pub fn encoding(&self) -> Option<&ContentTransferEncoding> {
        self.headers.get()
    }

    /// Get the headers from the Part
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the headers
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Set a header and move the Part
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set the headers and move the Part
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Set the body
    #[inline]
    pub fn set_body<T: Into<B>>(&mut self, body: T) {
        self.body = body.into();
    }

    /// Set the body and move the Part
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_body<T: Into<B>>(mut self, body: T) -> Self {
        self.set_body(body);
        self
    }

    /// Read the body
    #[inline]
    pub fn body_ref(&self) -> &B {
        &self.body
    }

    /// Streaming single part
    pub fn into_stream(self) -> SinglePartStream<B>
    where
        B: Payload,
    {
        self.into()
    }
}

impl<B> Display for SinglePart<B>
where
    B: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.headers.fmt(f)?;
        "\r\n".fmt(f)?;

        let body = self.body.as_ref().as_bytes().into();
        let mut encoder = EncoderChunk::get(self.encoding());
        let result = encoder
            .encode_chunk(body)
            .map_err(|_| FmtError::default())?;
        let body = from_utf8(&result).map_err(|_| FmtError::default())?;

        body.fmt(f)?;
        "\r\n".fmt(f)
    }
}

/// Stream for single part
///
pub struct SinglePartStream<B> {
    headers: Option<Headers>,
    body: Option<EncoderStream<B>>,
}

impl<B> Stream for SinglePartStream<B>
where
    B: Payload,
    B::Data: Into<Bytes>,
{
    type Item = Bytes;
    type Error = EncoderError<B::Error>;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(headers) = replace(&mut self.headers, None) {
            // stream headers
            let raw = Bytes::from(headers.to_string() + "\r\n");
            Ok(Async::Ready(Some(raw)))
        } else {
            // stream body
            let res = if let Some(body) = &mut self.body {
                body.poll()
            } else {
                // end of data
                return Ok(Async::Ready(None));
            };

            if let Ok(Async::Ready(None)) = &res {
                // end of stream
                self.body = None;
                Ok(Async::Ready(Some("\r\n".into())))
            } else {
                // chunk or error
                res
            }
        }
    }
}

/// Convert single part into boxed stream of binary chunks
///
impl<B> From<SinglePart<B>> for SinglePartStream<B>
where
    B: Payload,
{
    fn from(SinglePart { headers, body }: SinglePart<B>) -> Self {
        let body = {
            let encoding = headers.get();
            EncoderStream::wrap(encoding, body)
        };

        SinglePartStream {
            headers: Some(headers),
            body: Some(body),
        }
    }
}

/// The kind of multipart
///
pub enum MultiPartKind {
    /// Mixed kind to combine unrelated content parts
    ///
    /// For example this kind can be used to mix email message and attachments.
    Mixed,

    /// Alternative kind to join several variants of same email contents.
    ///
    /// That kind is recommended to use for joining plain (text) and rich (HTML) messages into single email message.
    Alternative,

    /// Related kind to mix content and related resources.
    ///
    /// For example, you can include images into HTML content using that.
    Related,
}

impl MultiPartKind {
    fn to_mime<S: AsRef<str>>(&self, boundary: Option<S>) -> Mime {
        let boundary = boundary
            .map(|s| s.as_ref().into())
            .unwrap_or_else(|| TextNonce::sized(68).unwrap().into_string());

        use self::MultiPartKind::*;
        format!(
            "multipart/{}; boundary=\"{}\"",
            match *self {
                Mixed => "mixed",
                Alternative => "alternative",
                Related => "related",
            },
            boundary
        ).parse()
        .unwrap()
    }

    fn from_mime(m: &Mime) -> Option<Self> {
        use self::MultiPartKind::*;
        match m.subtype().as_ref() {
            "mixed" => Some(Mixed),
            "alternative" => Some(Alternative),
            "related" => Some(Related),
            _ => None,
        }
    }
}

impl From<MultiPartKind> for Mime {
    fn from(m: MultiPartKind) -> Self {
        m.to_mime::<String>(None)
    }
}

/// Multipart builder
///
pub struct MultiPartBuilder {
    headers: Headers,
}

impl MultiPartBuilder {
    /// Creates default multipart builder
    #[inline]
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
        }
    }

    /// Set a header
    #[inline]
    pub fn header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set `Content-Type:` header using [`MultiPartKind`]
    #[inline]
    pub fn kind(self, kind: MultiPartKind) -> Self {
        self.header(ContentType(kind.into()))
    }

    /// Set custom boundary
    pub fn boundary<S: AsRef<str>>(self, boundary: S) -> Self {
        let kind = {
            let mime = &self.headers.get::<ContentType>().unwrap().0;
            MultiPartKind::from_mime(mime).unwrap()
        };
        let mime = kind.to_mime(Some(boundary.as_ref()));
        self.header(ContentType(mime))
    }

    /// Creates MultiPart
    #[inline]
    pub fn build<B>(self) -> MultiPart<B> {
        MultiPart {
            headers: self.headers,
            parts: Vec::new(),
        }
    }

    /// Creates MultiPart using part
    #[inline]
    pub fn part<B>(self, part: Part<B>) -> MultiPart<B> {
        self.build().part(part)
    }

    /// Creates MultiPart using singlepart
    #[inline]
    pub fn singlepart<B>(self, part: SinglePart<B>) -> MultiPart<B> {
        self.build().singlepart(part)
    }

    /// Creates MultiPart using multipart
    #[inline]
    pub fn multipart<B>(self, part: MultiPart<B>) -> MultiPart<B> {
        self.build().multipart(part)
    }
}

impl Default for MultiPartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Multipart variant with parts
///
pub struct MultiPart<B = Body> {
    headers: Headers,
    parts: Vec<Part<B>>,
}

impl MultiPart<()> {
    /// Creates MultiPart builder
    #[inline]
    pub fn builder() -> MultiPartBuilder {
        MultiPartBuilder::new()
    }

    /// Creates MultiPart builder of specified kind
    #[inline]
    pub fn builder_kind(kind: MultiPartKind) -> MultiPartBuilder {
        MultiPartBuilder::new().kind(kind)
    }

    /// Creates MultiPart mixed
    ///
    /// Shortcut for `MultiPart::new(MultiPartKind::Mixed)`
    #[inline]
    pub fn mixed() -> MultiPartBuilder {
        MultiPart::builder_kind(MultiPartKind::Mixed)
    }

    /// Creates MultiPart alternative
    ///
    /// Shortcut for `MultiPart::new(MultiPartKind::Alternative)`
    #[inline]
    pub fn alternative() -> MultiPartBuilder {
        MultiPart::builder_kind(MultiPartKind::Alternative)
    }

    /// Creates MultiPart alternative
    ///
    /// Shortcut for `MultiPart::new(MultiPartKind::Related)`
    #[inline]
    pub fn related() -> MultiPartBuilder {
        MultiPart::builder_kind(MultiPartKind::Related)
    }
}

impl<B> MultiPart<B> {
    /// Add a sub-part and move the MultiPart
    #[inline]
    pub fn part(mut self, part: Part<B>) -> Self {
        self.parts.push(part);
        self
    }

    /// Add a single sub-part and move the MultiPart
    #[inline]
    pub fn singlepart(mut self, part: SinglePart<B>) -> Self {
        self.parts.push(Part::Single(part));
        self
    }

    /// Add a multi sub-part and move the MultiPart
    #[inline]
    pub fn multipart(mut self, part: MultiPart<B>) -> Self {
        self.parts.push(Part::Multi(part));
        self
    }

    /// Get the boundary of MultiPart contents
    #[inline]
    pub fn boundary(&self) -> String {
        let content_type = &self.headers.get::<ContentType>().unwrap().0;
        content_type.get_param("boundary").unwrap().as_str().into()
    }

    /// Get the headers from the MultiPart
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the headers
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Get the sub-parts from the MultiPart.
    #[inline]
    pub fn parts(&self) -> &Vec<Part<B>> {
        &self.parts
    }

    /// Get a mutable reference to the sub-parts
    #[inline]
    pub fn parts_mut(&mut self) -> &mut Parts<B> {
        &mut self.parts
    }

    /// Streaming multi part
    pub fn into_stream(self) -> MultiPartStream<B>
    where
        B: Payload,
    {
        self.into()
    }
}

impl<B> Display for MultiPart<B>
where
    B: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.headers.fmt(f)?;
        "\r\n".fmt(f)?;

        let boundary = self.boundary();

        for part in &self.parts {
            "--".fmt(f)?;
            boundary.fmt(f)?;
            "\r\n".fmt(f)?;
            part.fmt(f)?;
        }

        "--".fmt(f)?;
        boundary.fmt(f)?;
        "--\r\n".fmt(f)
    }
}

/// Stream for multi part
///
pub struct MultiPartStream<B> {
    boundary: Bytes,
    headers: Option<Headers>,
    parts: VecDeque<PartStream<B>>,
}

impl<B> Stream for MultiPartStream<B>
where
    B: Payload,
    B::Data: Into<Bytes>,
{
    type Item = Bytes;
    type Error = EncoderError<B::Error>;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(headers) = replace(&mut self.headers, None) {
            // stream headers
            let headers = headers.to_string();
            let has_parts = !self.parts.is_empty();
            let mut chunk = BytesMut::with_capacity(
                headers.len() + 2 // add ending \r\n
                    + if has_parts {
                    // need extra bytes for open boundary
                    self.boundary.len() + 4 // add beginning "--" and ending "\r\n"
                } else {
                    0
                },
            );

            // put headers
            chunk.put(&headers);
            chunk.put_slice(b"\r\n");

            // put open boundary
            if has_parts {
                chunk.put_slice(b"--");
                chunk.put(&self.boundary);
                chunk.put_slice(b"\r\n");
            }

            Ok(Async::Ready(Some(chunk.freeze())))
        } else {
            // stream body
            if self.parts.is_empty() {
                // end of data
                Ok(Async::Ready(None))
            } else {
                let res = self.parts[0].poll();

                if let Ok(Async::Ready(None)) = &res {
                    // end of stream
                    self.parts.pop_front();

                    let has_parts = !self.parts.is_empty();

                    let mut chunk = BytesMut::with_capacity(
                        self.boundary.len() + 6 // add beginning "--" and ending "--\r\n"
                            + if has_parts {
                                self.boundary.len() + 4 // add beginning "--" and ending "\r\n"
                            } else {
                                0
                            },
                    );

                    Ok(Async::Ready(Some(chunk.freeze())))
                } else {
                    // chunk or error
                    res
                }
            }
        }
    }
}

impl<B> Payload for MultiPartStream<B>
where
    B: Payload,
    B::Data: Into<Bytes>,
    B::Error: Error + Send + Sync,
{
    type Data = Chunk;
    type Error = EncoderError<B::Error>;

    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        self.poll()
            .map(|async| async.map(|ready| ready.map(|chunk| chunk.into())))
    }
}

/// Convert single part into boxed stream of binary chunks
///
impl<B> From<MultiPart<B>> for MultiPartStream<B>
where
    B: Payload,
{
    fn from(this: MultiPart<B>) -> Self {
        let boundary = this.boundary().into();
        MultiPartStream {
            boundary,
            headers: Some(this.headers),
            parts: this
                .parts
                .into_iter()
                .map(|part| part.into_stream())
                .collect::<VecDeque<_>>(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{MultiPart, Part, SinglePart};
    use header;

    #[test]
    fn single_part_binary() {
        let part: SinglePart<String> = SinglePart::builder()
            .header(header::ContentType(
                "text/plain; charset=utf8".parse().unwrap(),
            )).header(header::ContentTransferEncoding::Binary)
            .body(String::from("Текст письма в уникоде"));

        assert_eq!(
            format!("{}", part),
            concat!(
                "Content-Type: text/plain; charset=utf8\r\n",
                "Content-Transfer-Encoding: binary\r\n",
                "\r\n",
                "Текст письма в уникоде\r\n"
            )
        );
    }

    #[test]
    fn single_part_quoted_printable() {
        let part: SinglePart<String> = SinglePart::builder()
            .header(header::ContentType(
                "text/plain; charset=utf8".parse().unwrap(),
            )).header(header::ContentTransferEncoding::QuotedPrintable)
            .body(String::from("Текст письма в уникоде"));

        assert_eq!(
            format!("{}", part),
            concat!(
                "Content-Type: text/plain; charset=utf8\r\n",
                "Content-Transfer-Encoding: quoted-printable\r\n",
                "\r\n",
                "=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n",
                "=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5\r\n"
            )
        );
    }

    #[test]
    fn single_part_base64() {
        let part: SinglePart<String> = SinglePart::builder()
            .header(header::ContentType(
                "text/plain; charset=utf8".parse().unwrap(),
            )).header(header::ContentTransferEncoding::Base64)
            .body(String::from("Текст письма в уникоде"));

        assert_eq!(
            format!("{}", part),
            concat!(
                "Content-Type: text/plain; charset=utf8\r\n",
                "Content-Transfer-Encoding: base64\r\n",
                "\r\n",
                "0KLQtdC60YHRgiDQv9C40YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LU=\r\n"
            )
        );
    }

    #[test]
    fn multi_part_mixed() {
        let part: MultiPart<String> = MultiPart::mixed()
            .boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .part(Part::Single(
                SinglePart::builder()
                    .header(header::ContentType(
                        "text/plain; charset=utf8".parse().unwrap(),
                    )).header(header::ContentTransferEncoding::Binary)
                    .body(String::from("Текст письма в уникоде")),
            )).singlepart(
                SinglePart::builder()
                    .header(header::ContentType(
                        "text/plain; charset=utf8".parse().unwrap(),
                    )).header(header::ContentDisposition {
                        disposition: header::DispositionType::Attachment,
                        parameters: vec![header::DispositionParam::Filename(
                            header::Charset::Ext("utf-8".into()),
                            None,
                            "example.c".as_bytes().into(),
                        )],
                    }).header(header::ContentTransferEncoding::Binary)
                    .body(String::from("int main() { return 0; }")),
            );

        assert_eq!(format!("{}", part),
                   concat!("Content-Type: multipart/mixed;",
                           " boundary=\"F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\"\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "Текст письма в уникоде\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Disposition: attachment; filename=\"example.c\"\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "int main() { return 0; }\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK--\r\n"));
    }

    #[test]
    fn multi_part_alternative() {
        let part: MultiPart<String> = MultiPart::alternative()
            .boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .part(Part::Single(SinglePart::builder()
                             .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                             .header(header::ContentTransferEncoding::Binary)
                             .body(String::from("Текст письма в уникоде"))))
            .singlepart(SinglePart::builder()
                             .header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                             .header(header::ContentTransferEncoding::Binary)
                             .body(String::from("<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>")));

        assert_eq!(format!("{}", part),
                   concat!("Content-Type: multipart/alternative;",
                           " boundary=\"F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\"\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "Текст письма в уникоде\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/html; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK--\r\n"));
    }

    #[test]
    fn multi_part_mixed_related() {
        let part: MultiPart<String> = MultiPart::mixed()
            .boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .multipart(MultiPart::related()
                            .boundary("E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh")
                            .singlepart(SinglePart::builder()
                                             .header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                                             .header(header::ContentTransferEncoding::Binary)
                                             .body(String::from("<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>")))
                            .singlepart(SinglePart::builder()
                                             .header(header::ContentType("image/png".parse().unwrap()))
                                             .header(header::ContentLocation("/image.png".into()))
                                             .header(header::ContentTransferEncoding::Base64)
                                             .body(String::from("1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"))))
            .singlepart(SinglePart::builder()
                             .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                             .header(header::ContentDisposition {
                                 disposition: header::DispositionType::Attachment,
                                 parameters: vec![header::DispositionParam::Filename(header::Charset::Ext("utf-8".into()), None, "example.c".as_bytes().into())]
                             })
                             .header(header::ContentTransferEncoding::Binary)
                             .body(String::from("int main() { return 0; }")));

        assert_eq!(format!("{}", part),
                   concat!("Content-Type: multipart/mixed;",
                           " boundary=\"F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\"\r\n",
                           "\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: multipart/related;",
                           " boundary=\"E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh\"\r\n",
                           "\r\n",
                           "--E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh\r\n",
                           "Content-Type: text/html; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>\r\n",
                           "--E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh\r\n",
                           "Content-Type: image/png\r\n",
                           "Content-Location: /image.png\r\n",
                           "Content-Transfer-Encoding: base64\r\n",
                           "\r\n",
                           "MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3\r\n",
                           "ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0\r\n",
                           "NTY3ODkwMTIzNDU2Nzg5MA==\r\n",
                           "--E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh--\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK\r\n",
                           "Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Disposition: attachment; filename=\"example.c\"\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "int main() { return 0; }\r\n",
                           "--F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK--\r\n"));
    }
}

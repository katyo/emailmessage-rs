use std::fmt::{Display, Formatter, Error as FmtError, Result as FmtResult};
use std::str::{from_utf8};
use futures::{Stream, stream};
use mime::{Mime};
use textnonce::{TextNonce};
use encoder::{EncoderChunk, EncoderStream, EncodedBinaryStream};
use header::{Header, Headers, ContentType, ContentTransferEncoding};
use {MailBody, BinaryChunk};

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
/// let part = SinglePart::new()
///      .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
///      .with_header(header::ContentTransferEncoding::Binary)
///      .with_body("Текст письма в уникоде");
/// ```
///
pub struct SinglePart<B = MailBody> {
    headers: Headers,
    body: B,
}

impl<B> Default for SinglePart<B>
where B: Default
{
    fn default() -> Self {
        SinglePart { headers: Headers::new(), body: B::default() }
    }
}

impl<B> SinglePart<B> {
    /// Constructs a default SinglePart
    pub fn new() -> Self where B: Default {
        SinglePart::default()
    }

    /// Constructs a SinglePart with 7bit encoding
    ///
    /// Shortcut for SinglePart::new().with_header(ContentTransferEncoding::SevenBit)
    pub fn seven_bit() -> Self where B: Default {
        SinglePart::new().with_header(ContentTransferEncoding::SevenBit)
    }

    /// Constructs a SinglePart with quoted-printable encoding
    ///
    /// Shortcut for SinglePart::new().with_header(ContentTransferEncoding::QuotedPrintable)
    pub fn quoted_printable() -> Self where B: Default {
        SinglePart::new().with_header(ContentTransferEncoding::QuotedPrintable)
    }

    /// Constructs a SinglePart with base64 encoding
    ///
    /// Shortcut for SinglePart::new().with_header(ContentTransferEncoding::Base64)
    pub fn base64() -> Self where B: Default {
        SinglePart::new().with_header(ContentTransferEncoding::Base64)
    }

    /// Constructs a SinglePart with 8-bit encoding
    ///
    /// Shortcut for SinglePart::new().with_header(ContentTransferEncoding::EightBit)
    pub fn eight_bit() -> Self where B: Default {
        SinglePart::new().with_header(ContentTransferEncoding::EightBit)
    }

    /// Constructs a SinglePart with binary encoding
    ///
    /// Shortcut for SinglePart::new().with_header(ContentTransferEncoding::Binary)
    pub fn binary() -> Self where B: Default {
        SinglePart::new().with_header(ContentTransferEncoding::Binary)
    }

    /// Get the transfer encoding
    #[inline]
    pub fn encoding(&self) -> ContentTransferEncoding {
        self.headers.get::<ContentTransferEncoding>()
            .map(Clone::clone)
            .unwrap_or(ContentTransferEncoding::Binary)
    }

    /// Get the headers from the Part.
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Set a header and move the Part.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set the headers and move the Part.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Set the body.
    #[inline]
    pub fn set_body<T: Into<B>>(&mut self, body: T) {
        self.body = body.into();
    }

    /// Set the body and move the Part.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_body<T: Into<B>>(mut self, body: T) -> Self {
        self.set_body(body);
        self
    }

    /// Read the body.
    #[inline]
    pub fn body_ref(&self) -> &B { &self.body }
}

impl<B> Display for SinglePart<B>
where B: AsRef<str>
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.headers.fmt(f)?;
        "\r\n".fmt(f)?;

        let body = self.body.as_ref().as_bytes().into();
        let mut encoder = EncoderChunk::get(self.encoding());
        let result = encoder.encode_chunk(body).map_err(|_| FmtError::default())?;
        let body = from_utf8(&result).map_err(|_| FmtError::default())?;

        body.fmt(f)?;
        "\r\n".fmt(f)
    }
}

/// Convert single part into boxed stream of binary chunks
///
impl<B, C, E> Into<Box<EncodedBinaryStream<E>>> for SinglePart<B>
where B: Stream<Item = C, Error = E> + 'static,
      C: Into<BinaryChunk>,
      E: 'static,
{
    fn into(self) -> Box<EncodedBinaryStream<E>> {
        Box::new(stream::once(Ok(Vec::from(self.headers.to_string() + "\r\n")))
                 .chain(EncoderStream::wrap(self.encoding(),
                                            self.body.map(|chunk| chunk.into().as_ref().into())))
                 .chain(stream::once(Ok(Vec::from("\r\n"))))
        )
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
        let boundary = boundary.map(|s| s.as_ref().into())
            .unwrap_or_else(|| TextNonce::sized(68).unwrap().into_string());
        
        use self::MultiPartKind::*;
        format!("multipart/{}; boundary=\"{}\"",
                match *self {
                    Mixed => "mixed",
                    Alternative => "alternative",
                    Related => "related",
                },
                boundary
        ).parse().unwrap()
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

/// MIME part variants
///
pub enum Part<B = MailBody> {
    /// Single part with content
    ///
    Single(SinglePart<B>),
    
    /// Multiple parts of content
    ///
    Multi(MultiPart<B>),
}

impl<B> Default for Part<B>
where B: Default
{
    fn default() -> Self {
        Part::Single(SinglePart::default())
    }
}

impl<B> Display for Part<B>
where B: AsRef<str>
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Part::Single(ref part) => part.fmt(f),
            Part::Multi(ref part) => part.fmt(f),
        }
    }
}

/// Convert generic part into boxed stream of binary chunks
///
impl<B, C, E> Into<Box<EncodedBinaryStream<E>>> for Part<B>
where B: Stream<Item = C, Error = E> + 'static,
      C: Into<BinaryChunk>,
      E: 'static,
{
    fn into(self) -> Box<EncodedBinaryStream<E>> {
        match self {
            Part::Single(part) => part.into(),
            Part::Multi(part) => part.into(),
        }
    }
}

pub type Parts<B = MailBody> = Vec<Part<B>>;

pub struct MultiPart<B = MailBody> {
    headers: Headers,
    parts: Parts<B>,
}

impl<B> Default for MultiPart<B> {
    fn default() -> Self {
        MultiPart { headers: Headers::new(), parts: Vec::new() }
    }
}

impl<B> MultiPart<B> {
    /// Constructs MultiPart of specified kind
    #[inline]
    pub fn new(kind: MultiPartKind) -> Self {
        let mut headers = Headers::new();
        
        headers.set(ContentType(kind.into()));
        
        MultiPart { headers, parts: Parts::new() }
    }

    /// Constructs MultiPart mixed
    ///
    /// Shortcut for MultiPart::new(MultiPartKind::Mixed)
    #[inline]
    pub fn mixed() -> Self {
        MultiPart::new(MultiPartKind::Mixed)
    }

    /// Constructs MultiPart alternative
    ///
    /// Shortcut for MultiPart::new(MultiPartKind::Alternative)
    #[inline]
    pub fn alternative() -> Self {
        MultiPart::new(MultiPartKind::Alternative)
    }

    /// Constructs MultiPart alternative
    ///
    /// Shortcut for MultiPart::new(MultiPartKind::Related)
    #[inline]
    pub fn related() -> Self {
        MultiPart::new(MultiPartKind::Related)
    }

    /// Get the boundary of MultiPart contents.
    #[inline]
    pub fn boundary(&self) -> String {
        let content_type = &self.headers.get::<ContentType>().unwrap().0;
        content_type.get_param("boundary").unwrap().as_str().into()
    }

    /// Set a boundary and move the MultiPart.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_boundary<S: AsRef<str>>(self, boundary: S) -> Self {
        let kind = {
            let mime = &self.headers.get::<ContentType>().unwrap().0;
            MultiPartKind::from_mime(mime).unwrap()
        };
        let mime = kind.to_mime(Some(boundary.as_ref()));
        self.with_header(ContentType(mime))
    }

    /// Get the headers from the MultiPart.
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Set a header and move the MultiPart.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set the headers and move the MultiPart.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Get the sub-parts from the MultiPart.
    #[inline]
    pub fn parts(&self) -> &Parts<B> {
        &self.parts
    }

    /// Get a mutable reference to the sub-parts.
    #[inline]
    pub fn parts_mut(&mut self) -> &mut Parts<B> {
        &mut self.parts
    }

    /// Add a sub-part and move the MultiPart.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_part(mut self, part: Part<B>) -> Self {
        self.parts.push(part);
        self
    }

    /// Add a single sub-part and move the MultiPart.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_singlepart(mut self, part: SinglePart<B>) -> Self {
        self.parts.push(Part::Single(part));
        self
    }

    /// Add a multi sub-part and move the MultiPart.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_multipart(mut self, part: MultiPart<B>) -> Self {
        self.parts.push(Part::Multi(part));
        self
    }

    /// Set the parts and move the MultiPart.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_parts(mut self, parts: Parts<B>) -> Self {
        self.parts = parts;
        self
    }
}

impl<B> Display for MultiPart<B>
where B: AsRef<str>
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

/// Convert single part into boxed stream of binary chunks
///
impl<B, C, E> Into<Box<EncodedBinaryStream<E>>> for MultiPart<B>
where B: Stream<Item = C, Error = E> + 'static,
      C: Into<BinaryChunk>,
      E: 'static,
{
    fn into(self) -> Box<EncodedBinaryStream<E>> {
        let boundary = self.boundary();
        let boundary_open = stream::once(Ok(Vec::from(String::from("--") + &boundary + "\r\n")));
        let boundary_close = stream::once(Ok(Vec::from(String::from("--") + &boundary + "--\r\n")));
        
        let mut chain: Box<EncodedBinaryStream<E>> = Box::new(boundary_open);
        
        for part in self.parts {
            chain = Box::new(chain.chain(stream::once(Ok(Vec::from(String::from("--") + &boundary + "\r\n"))))
                             .chain(Into::<Box<EncodedBinaryStream<E>>>::into(part)));
        }
        
        Box::new(chain.chain(boundary_close))
    }
}

#[cfg(test)]
mod test {
    use header;
    use super::{Part, SinglePart, MultiPart, MultiPartKind};

    #[test]
    fn single_part_binary() {
        let part: SinglePart<String> = SinglePart::new()
            .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
            .with_header(header::ContentTransferEncoding::Binary)
            .with_body(String::from("Текст письма в уникоде"));

        assert_eq!(format!("{}", part),
                   concat!("Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Transfer-Encoding: binary\r\n",
                           "\r\n",
                           "Текст письма в уникоде\r\n"));
    }

    #[test]
    fn single_part_quoted_printable() {
        let part: SinglePart<String> = SinglePart::new()
            .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
            .with_header(header::ContentTransferEncoding::QuotedPrintable)
            .with_body(String::from("Текст письма в уникоде"));

        assert_eq!(format!("{}", part),
                   concat!("Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Transfer-Encoding: quoted-printable\r\n",
                           "\r\n",
                           "=D0=A2=D0=B5=D0=BA=D1=81=D1=82 =D0=BF=D0=B8=D1=81=D1=8C=D0=BC=D0=B0 =D0=B2 =\r\n",
                           "=D1=83=D0=BD=D0=B8=D0=BA=D0=BE=D0=B4=D0=B5\r\n"));
    }

    #[test]
    fn single_part_base64() {
        let part: SinglePart<String> = SinglePart::new()
            .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
            .with_header(header::ContentTransferEncoding::Base64)
            .with_body(String::from("Текст письма в уникоде"));

        assert_eq!(format!("{}", part),
                   concat!("Content-Type: text/plain; charset=utf8\r\n",
                           "Content-Transfer-Encoding: base64\r\n",
                           "\r\n",
                           "0KLQtdC60YHRgiDQv9C40YHRjNC80LAg0LIg0YPQvdC40LrQvtC00LU=\r\n"));
    }

    #[test]
    fn multi_part_mixed() {
        let part: MultiPart<String> = MultiPart::new(MultiPartKind::Mixed)
            .with_boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .with_part(Part::Single(SinglePart::new()
                             .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                             .with_header(header::ContentTransferEncoding::Binary)
                             .with_body(String::from("Текст письма в уникоде"))))
            .with_singlepart(SinglePart::new()
                             .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                             .with_header(header::ContentDisposition {
                                 disposition: header::DispositionType::Attachment,
                                 parameters: vec![header::DispositionParam::Filename(header::Charset::Ext("utf-8".into()), None, "example.c".as_bytes().into())]
                             })
                             .with_header(header::ContentTransferEncoding::Binary)
                             .with_body(String::from("int main() { return 0; }")));

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
        let part: MultiPart<String> = MultiPart::new(MultiPartKind::Alternative)
            .with_boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .with_part(Part::Single(SinglePart::new()
                             .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                             .with_header(header::ContentTransferEncoding::Binary)
                             .with_body(String::from("Текст письма в уникоде"))))
            .with_singlepart(SinglePart::new()
                             .with_header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                             .with_header(header::ContentTransferEncoding::Binary)
                             .with_body(String::from("<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>")));

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
        let part: MultiPart<String> = MultiPart::new(MultiPartKind::Mixed)
            .with_boundary("F2mTKN843loAAAAA8porEdAjCKhArPxGeahYoZYSftse1GT/84tup+O0bs8eueVuAlMK")
            .with_multipart(MultiPart::new(MultiPartKind::Related)
                            .with_boundary("E912L4JH3loAAAAAFu/33Gx7PEoTMmhGaxG3FlbVMQHctj96q4nHvBM+7DTtXo/im8gh")
                            .with_singlepart(SinglePart::new()
                                             .with_header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                                             .with_header(header::ContentTransferEncoding::Binary)
                                             .with_body(String::from("<p>Текст <em>письма</em> в <a href=\"https://ru.wikipedia.org/wiki/Юникод\">уникоде</a><p>")))
                            .with_singlepart(SinglePart::new()
                                             .with_header(header::ContentType("image/png".parse().unwrap()))
                                             .with_header(header::ContentLocation("/image.png".into()))
                                             .with_header(header::ContentTransferEncoding::Base64)
                                             .with_body(String::from("1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"))))
            .with_singlepart(SinglePart::new()
                             .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                             .with_header(header::ContentDisposition {
                                 disposition: header::DispositionType::Attachment,
                                 parameters: vec![header::DispositionParam::Filename(header::Charset::Ext("utf-8".into()), None, "example.c".as_bytes().into())]
                             })
                             .with_header(header::ContentTransferEncoding::Binary)
                             .with_body(String::from("int main() { return 0; }")));

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

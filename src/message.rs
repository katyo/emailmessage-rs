use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::{SystemTime};
use futures::{Future, Stream, Sink, stream};
use hyper::{Body as HyperBody, Chunk as HyperChunk};
use tokio_proto::streaming::{Body as StreamingBody};
use header::{Headers, Header, Date, EmailDate};

pub type MailBody = HyperBody;

#[derive(Clone, Debug)]
pub struct Message<B = MailBody> {
    headers: Headers,
    body: B,
}

impl<B> Message<B> {
    /// Constructs a default message
    #[inline]
    pub fn new() -> Self
    where B: Default
    {
        Message::default().with_date(None)
    }

    /// Get the headers from the Message.
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Set a header and move the Message.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Set the headers and move the Message.
    ///
    /// Useful for the "builder-style" pattern.
    #[inline]
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Set a date and move the Message.
    ///
    /// Useful for the "builder-style" pattern.
    ///
    /// `None` value means use current local time as a date.
    #[inline]
    pub fn with_date(self, date: Option<EmailDate>) -> Self {
        let date: EmailDate = date.unwrap_or_else(|| SystemTime::now().into());
        
        self.with_header(Date(date))
    }

    /// Set the body.
    #[inline]
    pub fn set_body<T: Into<B>>(&mut self, body: T) {
        self.body = body.into();
    }

    /// Set the body and move the Message.
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

    /// Create stream from the Message.
    pub fn to_stream<C, E>(self) -> Box<Stream<Item = Vec<u8>, Error = E>>
    where B: Stream<Item = C, Error = E> + 'static,
          C: Into<HyperChunk>,
          E: 'static,
    {
        Box::new(stream::once(Ok(Vec::from(self.headers.to_string())))
                 .chain(self.body.map(|chunk| chunk.into().as_ref().into())))
    }

    pub fn streaming<C, E>(self) -> (StreamingBody<Vec<u8>, E>, Box<Future<Item = (), Error = E>>)
    where B: Stream<Item = C, Error = E> + 'static,
          C: Into<HyperChunk>,
          E: 'static,
    {
        let (sender, body) = StreamingBody::pair();
        
        (body,
         Box::new(sender.send_all(self.to_stream::<C, E>()
                                  .map(Ok).map_err(|_| panic!()))
                  .map(|_| ()).map_err(|_| panic!())))
    }
}

impl<B> Default for Message<B>
where B: Default
{
    fn default() -> Self {
        Message {
            headers: Headers::default(),
            body: B::default()
        }
    }
}

impl<B> Display for Message<B>
where B: Display
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.headers.fmt(f)?;
        write!(f, "{}", self.body)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use header;
    use mailbox::{Mailbox};
    use message::{Message};

    use std::str::from_utf8;
    use futures::{Stream, Future};
    use tokio_core::reactor::{Core};
    
    #[test]
    fn date_header() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();
        
        let email: Message<String> = Message::new()
            .with_date(Some(date))
            .with_body("\r\n");
        
        assert_eq!(format!("{}", email), "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n\r\n");
    }

    #[test]
    fn email_message() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();
        
        let email: Message<String> = Message::new()
            .with_date(Some(date))
            .with_header(header::From(vec![Mailbox::new(Some("Каи".into()), "kayo@example.com".parse().unwrap())]))
            .with_header(header::To(vec!["Pony O.P. <pony@domain.tld>".parse().unwrap()]))
            .with_header(header::Subject("яңа ел белән!".into()))
            .with_body("\r\nHappy new year!");
        
        assert_eq!(format!("{}", email),
                   concat!("Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                           "From: =?utf-8?b?0JrQsNC4?= <kayo@example.com>\r\n",
                           "To: Pony O.P. <pony@domain.tld>\r\n",
                           "Subject: =?utf-8?b?0Y/So9CwINC10Lsg0LHQtdC705nQvSE=?=\r\n",
                           "\r\n",
                           "Happy new year!"));
    }

    #[test]
    fn message_to_stream() {
        let mut core = Core::new().unwrap();
        
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();
        
        let email: Message = Message::new()
            .with_date(Some(date))
            .with_header(header::From(vec![Mailbox::new(Some("Каи".into()), "kayo@example.com".parse().unwrap())]))
            .with_header(header::To(vec!["Pony O.P. <pony@domain.tld>".parse().unwrap()]))
            .with_header(header::Subject("яңа ел белән!".into()))
            .with_body("\r\nHappy new year!");
        
        let body = email.to_stream();
        
        assert_eq!(core.run(body.concat2().map(|b| String::from(from_utf8(&b).unwrap()))).unwrap(),
                   concat!("Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                           "From: =?utf-8?b?0JrQsNC4?= <kayo@example.com>\r\n",
                           "To: Pony O.P. <pony@domain.tld>\r\n",
                           "Subject: =?utf-8?b?0Y/So9CwINC10Lsg0LHQtdC705nQvSE=?=\r\n",
                           "\r\n",
                           "Happy new year!"));
    }
}

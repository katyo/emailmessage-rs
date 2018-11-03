use super::{Body, Mailbox};
use bytes::Bytes;
use encoder::{EncoderError, EncoderStream};
use futures::{Async, Poll, Stream};
use header::{self, EmailDate, Header, Headers, MailboxesHeader};
use hyper::body::Payload;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::mem::replace;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct MessageBuilder {
    headers: Headers,
}

impl MessageBuilder {
    /// Creates a new default message builder
    #[inline]
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
        }
    }

    /// Set custom header to message
    #[inline]
    pub fn header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }

    /// Add mailbox to header
    pub fn mailbox<H: Header + MailboxesHeader>(mut self, header: H) -> Self {
        if self.headers.has::<H>() {
            self.headers.get_mut::<H>().unwrap().join_mailboxes(header);
            self
        } else {
            self.header(header)
        }
    }

    /// Add `Date:` header to message
    ///
    /// Shortcut for `self.header(header::Date(date))`.
    #[inline]
    pub fn date(self, date: EmailDate) -> Self {
        self.header(header::Date(date))
    }

    /// Set `Date:` header using current date/time
    ///
    /// Shortcut for `self.date(SystemTime::now())`.
    #[inline]
    pub fn date_now(self) -> Self {
        self.date(SystemTime::now().into())
    }

    /// Set `Subject:` header to message
    ///
    /// Shortcut for `self.header(header::Subject(subject.into()))`.
    #[inline]
    pub fn subject<S: Into<String>>(self, subject: S) -> Self {
        self.header(header::Subject(subject.into()))
    }

    /// Set `Mime-Version:` header to 1.0
    ///
    /// Shortcut for `self.header(header::MIME_VERSION_1_0)`.
    #[inline]
    pub fn mime_1_0(self) -> Self {
        self.header(header::MIME_VERSION_1_0)
    }

    /// Set `Sender:` header
    ///
    /// Shortcut for `self.header(header::Sender(mbox))`.
    #[inline]
    pub fn sender(self, mbox: Mailbox) -> Self {
        self.header(header::Sender(mbox))
    }

    /// Set or add mailbox to `From:` header
    ///
    /// Shortcut for `self.mailbox(header::From(mbox))`.
    #[inline]
    pub fn from(self, mbox: Mailbox) -> Self {
        self.mailbox(header::From(vec![mbox]))
    }

    /// Set or add mailbox to `ReplyTo:` header
    ///
    /// Shortcut for `self.mailbox(header::ReplyTo(mbox))`.
    #[inline]
    pub fn reply_to(self, mbox: Mailbox) -> Self {
        self.mailbox(header::ReplyTo(vec![mbox]))
    }

    /// Set or add mailbox to `To:` header
    ///
    /// Shortcut for `self.mailbox(header::To(mbox))`.
    #[inline]
    pub fn to(self, mbox: Mailbox) -> Self {
        self.mailbox(header::To(vec![mbox]))
    }

    /// Set or add mailbox to `Cc:` header
    ///
    /// Shortcut for `self.mailbox(header::Cc(mbox))`.
    #[inline]
    pub fn cc(self, mbox: Mailbox) -> Self {
        self.mailbox(header::Cc(vec![mbox]))
    }

    /// Set or add mailbox to `Bcc:` header
    ///
    /// Shortcut for `self.mailbox(header::Bcc(mbox))`.
    #[inline]
    pub fn bcc(self, mbox: Mailbox) -> Self {
        self.mailbox(header::Bcc(vec![mbox]))
    }

    /// Add body and construct [`Message`]
    #[inline]
    pub fn body<T>(self, body: T) -> Message<T> {
        Message {
            headers: self.headers,
            body,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Message<B = Body> {
    headers: Headers,
    body: B,
}

impl Message<()> {
    /// Create a new message builder without headers
    #[inline]
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Constructs a default message builder with date header which filled using current local time
    #[inline]
    pub fn create() -> MessageBuilder {
        Self::builder().date_now()
    }
}

impl<B> Message<B> {
    /// Get the headers from the Message
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the headers
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Set the body
    #[inline]
    pub fn set_body<T: Into<B>>(&mut self, body: T) {
        self.body = body.into();
    }

    /// Read the body
    #[inline]
    pub fn body_ref(&self) -> &B {
        &self.body
    }

    /// Converts message into stream
    pub fn into_stream(self) -> MessageStream<B>
    where
        B: Payload,
    {
        self.into()
    }
}

/// Stream for message
pub struct MessageStream<B> {
    headers: Option<Headers>,
    body: Option<EncoderStream<B>>,
}

impl<B> Stream for MessageStream<B>
where
    B: Payload,
    B::Data: Into<Bytes>,
{
    type Item = Bytes;
    type Error = EncoderError<B::Error>;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(headers) = replace(&mut self.headers, None) {
            // stream headers
            let raw = Bytes::from(headers.to_string());
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
                Ok(Async::Ready(None))
            } else {
                // chunk or error
                res
            }
        }
    }
}

/// Convert message into boxed stream of binary chunks
///
impl<B> From<Message<B>> for MessageStream<B>
where
    B: Payload,
{
    fn from(Message { headers, body }: Message<B>) -> Self {
        let body = {
            let encoding = headers.get();
            EncoderStream::wrap(encoding, body)
        };

        MessageStream {
            headers: Some(headers),
            body: Some(body),
        }
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        MessageBuilder::new()
    }
}

impl<B> Display for Message<B>
where
    B: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.headers.fmt(f)?;
        self.body.fmt(f)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use header;
    use mailbox::Mailbox;
    use message::Message;

    use futures::{Future, Stream};
    use std::str::from_utf8;

    #[test]
    fn date_header() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();

        let email = Message::builder().date(date).body("\r\n");

        assert_eq!(
            format!("{}", email),
            "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n\r\n"
        );
    }

    #[test]
    fn email_message() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();

        let email = Message::builder()
            .date(date)
            .header(header::From(vec![Mailbox::new(
                Some("Каи".into()),
                "kayo@example.com".parse().unwrap(),
            )])).header(header::To(vec![
                "Pony O.P. <pony@domain.tld>".parse().unwrap(),
            ])).header(header::Subject("яңа ел белән!".into()))
            .body("\r\nHappy new year!");

        assert_eq!(
            format!("{}", email),
            concat!(
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                "From: =?utf-8?b?0JrQsNC4?= <kayo@example.com>\r\n",
                "To: Pony O.P. <pony@domain.tld>\r\n",
                "Subject: =?utf-8?b?0Y/So9CwINC10Lsg0LHQtdC705nQvSE=?=\r\n",
                "\r\n",
                "Happy new year!"
            )
        );
    }

    #[test]
    fn message_to_stream() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();

        let email: Message = Message::builder()
            .date(date)
            .header(header::From(vec![Mailbox::new(
                Some("Каи".into()),
                "kayo@example.com".parse().unwrap(),
            )])).header(header::To(vec![
                "Pony O.P. <pony@domain.tld>".parse().unwrap(),
            ])).header(header::Subject("яңа ел белән!".into()))
            .body("\r\nHappy new year!".into());

        let body = email.into_stream();

        assert_eq!(
            body.concat2()
                .map(|b| String::from(from_utf8(&b).unwrap()))
                .wait()
                .unwrap(),
            concat!(
                "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                "From: =?utf-8?b?0JrQsNC4?= <kayo@example.com>\r\n",
                "To: Pony O.P. <pony@domain.tld>\r\n",
                "Subject: =?utf-8?b?0Y/So9CwINC10Lsg0LHQtdC705nQvSE=?=\r\n",
                "\r\n",
                "Happy new year!"
            )
        );
    }
}

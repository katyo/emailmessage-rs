extern crate emailaddress;
extern crate hyper;
extern crate base64;

mod mailbox;
mod utf8_b;
mod header;

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::{SystemTime};
use hyper::{Body};
use hyper::header::{Header, Headers, Date, HttpDate};

pub use mailbox::*;

#[derive(Clone, Debug)]
pub struct Message<B = Body> {
    headers: Headers,
    body: Option<B>,
}

impl<B> Message<B> {
    /// Constructs a default message
    #[inline]
    pub fn new() -> Self {
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
    pub fn with_date(self, date: Option<HttpDate>) -> Self {
        let date: HttpDate = date.unwrap_or_else(|| SystemTime::now().into());
        
        self.with_header(Date(date))
    }

    /// Set the body.
    #[inline]
    pub fn set_body<T: Into<B>>(&mut self, body: T) {
        self.body = Some(body.into());
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
    pub fn body_ref(&self) -> Option<&B> { self.body.as_ref() }

    //pub(crate) fn body_mut(&mut self) -> Option<&mut B> { self.body.as_mut() }
}

impl Message<Body> {
    /// Take the `Body` of this message.
    #[inline]
    pub fn body(self) -> Body {
        self.body.unwrap_or_default()
    }
}

impl<B> Default for Message<B> {
    fn default() -> Self {
        Message {
            headers: Headers::default(),
            body: Option::default()
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.headers.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, Mailbox, header};
    
    #[test]
    fn date_header() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();
        
        let email = Message::new()
            .with_date(Some(date));
        
        assert_eq!(format!("{}", email), "Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n");
    }

    #[test]
    fn email_message() {
        let date = "Tue, 15 Nov 1994 08:12:31 GMT".parse().unwrap();
        
        let email = Message::new()
            .with_date(Some(date))
            .with_header(header::From(vec![Mailbox::new(Some("Каи".into()), "kayo@example.com".parse().unwrap())]))
            .with_header(header::To(vec!["Pony O.P. <pony@domain.tld>".parse().unwrap()]))
            .with_header(header::Subject("яңа ел белән!".into()));
        
        assert_eq!(format!("{}", email),
                   concat!("Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n",
                           "From: =?utf-8?b?0JrQsNC4?= <kayo@example.com>\r\n",
                           "To: Pony O.P. <pony@domain.tld>\r\n",
                           "Subject: =?utf-8?b?0Y/So9CwINC10Lsg0LHQtdC705nQvSE=?=\r\n"));
    }
}

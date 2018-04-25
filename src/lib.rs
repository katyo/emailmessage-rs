//! # Email Message library for Rust
//! 
//! This project aims to provide a proper strongly typed way to build and parse emails.
//! 
//! ## Features
//! 
//! * Typed headers using `hyper::Header`
//! * _TODO_ Streamed building and parsing the message body
//! * _TODO_ Compatibility with most mail delivery systems
//! 
//! ## Usage
//! 
//! ### Format email messages
//! 
//! #### With string body
//! 
//! The easiest way how we can create email message with simple string
//! (see [format\_string.rs](examples/format_string.rs)).
//! 
//! ```rust
//! extern crate emailmessage;
//! 
//! use emailmessage::{header, Message};
//! 
//! fn main() {
//!     let m: Message<String> = Message::new()
//!         .with_header(header::From(vec!["Incognito <nobody@domain.tld>".parse().unwrap()]))
//!         .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
//!         .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
//!         .with_header(header::Subject("Happy new year".into()))
//!         .with_body("\r\nBe happy!");
//!     
//!     println!("{}", m);
//! }
//! ```
//! 
//! Run this example:
//! 
//! ```sh
//! $ cargo run --example format_string
//! 
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! 
//! Be happy!
//! ```
//! 
//! The unicode header data will be encoded using _UTF8-Base64_ encoding.
//! 
//! #### With mime body
//! 
//! ##### Single part
//! 
//! The more complex way is using MIME contents
//! (see [format\_mime.rs](examples/format_mime.rs)).
//! 
//! ```rust
//! extern crate emailmessage;
//! 
//! use emailmessage::{header, Message, SinglePart};
//! 
//! fn main() {
//!     let m: Message<SinglePart<String>> = Message::new()
//!         .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
//!         .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
//!         .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
//!         .with_header(header::Subject("Happy new year".into()))
//!         .with_header(header::MIME_VERSION_1_0)
//!         .with_body(
//!             SinglePart::new()
//!             .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
//!             .with_header(header::ContentTransferEncoding::QuotedPrintable)
//!             .with_body("Привет, мир!")
//!         );
//!     
//!     println!("{}", m);
//! }
//! ```
//! 
//! The body will be encoded using selected `Content-Transfer-Encoding`.
//! 
//! ```sh
//! $ cargo run --example format_mime
//! 
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! MIME-Version: 1.0
//! Content-Type: text/plain; charset=utf8
//! Content-Transfer-Encoding: quoted-printable
//! 
//! =D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!
//! 
//! ```
//! 
//! ##### Multiple parts
//! 
//! And the more advanced way is a using multipart MIME contents
//! (see [format\_multipart.rs](examples/format_multipart.rs)).
//! 
//! ```rust
//! extern crate emailmessage;
//! 
//! use emailmessage::{header, Message, SinglePart, MultiPart};
//! 
//! fn main() {
//!     let m: Message<MultiPart<String>> = Message::new()
//!         .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
//!         .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
//!         .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
//!         .with_header(header::Subject("Happy new year".into()))
//!         .with_header(header::MIME_VERSION_1_0)
//!         .with_body(
//!             MultiPart::mixed()
//!             .with_multipart(
//!                 MultiPart::alternative()
//!                 .with_singlepart(
//!                     SinglePart::quoted_printable()
//!                     .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
//!                     .with_body("Привет, мир!")
//!                 )
//!                 .with_multipart(
//!                     MultiPart::related()
//!                     .with_singlepart(
//!                         SinglePart::eight_bit()
//!                         .with_header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
//!                         .with_body("<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>")
//!                     )
//!                     .with_singlepart(
//!                         SinglePart::base64()
//!                         .with_header(header::ContentType("image/png".parse().unwrap()))
//!                         .with_header(header::ContentDisposition {
//!                             disposition: header::DispositionType::Inline,
//!                             parameters: vec![],
//!                         })
//!                         .with_body("<smile-raw-image-data>")
//!                     )
//!                 )
//!             )
//!             .with_singlepart(
//!                 SinglePart::seven_bit()
//!                 .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
//!                 .with_header(header::ContentDisposition {
//!                                  disposition: header::DispositionType::Attachment,
//!                                  parameters: vec![
//!                                      header::DispositionParam::Filename(
//!                                          header::Charset::Ext("utf-8".into()),
//!                                          None, "example.c".as_bytes().into()
//!                                      )
//!                                  ]
//!                              })
//!                 .with_body(String::from("int main() { return 0; }"))
//!             )
//!         );
//!     
//!     println!("{}", m);
//! }
//! ```
//! 
//! ```sh
//! $ cargo run --example format_multipart
//! 
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! MIME-Version: 1.0
//! Content-Type: multipart/mixed; boundary="RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m"
//! 
//! --RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m
//! Content-Type: multipart/alternative; boundary="qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy"
//! 
//! --qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy
//! Content-Transfer-Encoding: quoted-printable
//! Content-Type: text/plain; charset=utf8
//! 
//! =D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!
//! --qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy
//! Content-Type: multipart/related; boundary="BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8"
//! 
//! --BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8
//! Content-Transfer-Encoding: 8bit
//! Content-Type: text/html; charset=utf8
//! 
//! <p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>
//! --BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8
//! Content-Transfer-Encoding: base64
//! Content-Type: image/png
//! Content-Disposition: inline
//! 
//! PHNtaWxlLXJhdy1pbWFnZS1kYXRhPg==
//! --BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8--
//! --qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy--
//! --RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m
//! Content-Transfer-Encoding: 7bit
//! Content-Type: text/plain; charset=utf8
//! Content-Disposition: attachment; filename="example.c"
//! 
//! int main() { return 0; }
//! --RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m--
//! 
//! ```
//! 
//! #### Use streaming
//! 
//! The RAM-efficient way to formatting ans sending relatively big emails is a using streaming.
//! For example, you would like to send server access logs in attachments,
//! HTML-documents with related media resources or PDFs.
//! 
//! In examples above we actually allocated formatted emails in memory,
//! but usually we don't want to do same for big emails which size measures in MBytes.
//! 
//! ##### Simple string
//! 
//! The simple example below shows actually sent chunks of streamed message
//! (see [format\_stream.rs](examples/format_stream.rs)).
//! 
//! ```rust
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate emailmessage;
//! 
//! use std::str::from_utf8;
//! use futures::{Stream};
//! use tokio_core::reactor::{Core};
//! use emailmessage::{header, Message, BinaryStream};
//! 
//! fn main() {
//!     let mut core = Core::new().unwrap();
//!     
//!     let m: Message = Message::new()
//!         .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
//!         .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
//!         .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
//!         .with_header(header::Subject("Happy new year".into()))
//!         .with_body("\r\nBe happy!");
//!     
//!     let f = Into::<Box<BinaryStream<_>>>::into(m).map(|chunk| {
//!         println!("CHUNK[[\n{}]]", from_utf8(&chunk).unwrap());
//!         chunk
//!     }).concat2();
//! 
//!     core.run(f).unwrap();
//! }
//! ```
//! 
//! ```sh
//! $ cargo run --example format_stream
//! 
//! CHUNK[[
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! ]]
//! CHUNK[[
//! 
//! Be happy!]]
//! ```
//! 
//! In real world app we may do some buffering of stream to prevent too short and too long sendings.
//! 
//! ##### Multipart data
//! 
//! (see [format\_stream\_multipart.rs](examples/format_stream_multipart.rs))
//! 
//! ```rust
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate emailmessage;
//! 
//! use std::str::from_utf8;
//! use futures::{Stream};
//! use tokio_core::reactor::{Core};
//! use emailmessage::{header, Message, SinglePart, MultiPart, EncodedBinaryStream};
//! 
//! fn main() {
//!     let mut core = Core::new().unwrap();
//! 
//!     let b: MultiPart = MultiPart::mixed()
//!         .with_multipart(
//!             MultiPart::alternative()
//!                 .with_singlepart(
//!                     SinglePart::quoted_printable()
//!                         .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
//!                         .with_body("Привет, мир!")
//!                 )
//!                 .with_multipart(
//!                     MultiPart::related()
//!                         .with_singlepart(
//!                             SinglePart::eight_bit()
//!                                 .with_header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
//!                                 .with_body("<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>")
//!                         )
//!                         .with_singlepart(
//!                             SinglePart::base64()
//!                                 .with_header(header::ContentType("image/png".parse().unwrap()))
//!                                 .with_header(header::ContentDisposition {
//!                                     disposition: header::DispositionType::Inline,
//!                                     parameters: vec![],
//!                                 })
//!                                 .with_body("<smile-raw-image-data>")
//!                         )
//!                 )
//!         )
//!         .with_singlepart(
//!             SinglePart::seven_bit()
//!                 .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
//!                 .with_header(header::ContentDisposition {
//!                     disposition: header::DispositionType::Attachment,
//!                     parameters: vec![
//!                         header::DispositionParam::Filename(
//!                             header::Charset::Ext("utf-8".into()),
//!                             None, "example.c".as_bytes().into()
//!                         )
//!                     ]
//!                 })
//!                 .with_body(String::from("int main() { return 0; }"))
//!         );
//!     
//!     let m: Message<Box<EncodedBinaryStream<_>>> = Message::new()
//!         .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
//!         .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
//!         .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
//!         .with_header(header::Subject("Happy new year".into()))
//!         .with_header(header::MIME_VERSION_1_0)
//!         .with_body(b);
//! 
//!     let f = Into::<Box<EncodedBinaryStream<_>>>::into(m).map(|chunk| {
//!         println!("CHUNK[[\n{}]]", from_utf8(&chunk).unwrap());
//!         chunk
//!     }).concat2();
//! 
//!     core.run(f).unwrap();
//! }
//! ```
//! 
//! ```sh
//! $ cargo run --example format_stream_multipart
//! 
//! CHUNK[[
//! From: NoBody <nobody@domain.tld>
//! Reply-To: Yuin <yuin@domain.tld>
//! To: Hei <hei@domain.tld>
//! Subject: Happy new year
//! MIME-Version: 1.0
//! ]]
//! CHUNK[[
//! Content-Type: multipart/mixed; boundary="1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ"
//! ]]
//! CHUNK[[
//! --1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ
//! ]]
//! CHUNK[[
//! --1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ
//! ]]
//! CHUNK[[
//! Content-Type: multipart/alternative; boundary="TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY"
//! ]]
//! CHUNK[[
//! --TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY
//! ]]
//! CHUNK[[
//! --TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY
//! ]]
//! CHUNK[[
//! Content-Transfer-Encoding: quoted-printable
//! Content-Type: text/plain; charset=utf8
//! ]]
//! CHUNK[[
//! =D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!]]
//! CHUNK[[
//! ]]
//! CHUNK[[
//! --TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY
//! ]]
//! CHUNK[[
//! Content-Type: multipart/related; boundary="YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y"
//! ]]
//! CHUNK[[
//! --YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y
//! ]]
//! CHUNK[[
//! --YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y
//! ]]
//! CHUNK[[
//! Content-Transfer-Encoding: 8bit
//! Content-Type: text/html; charset=utf8
//! ]]
//! CHUNK[[
//! <p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>]]
//! CHUNK[[
//! ]]
//! CHUNK[[
//! --YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y
//! ]]
//! CHUNK[[
//! Content-Transfer-Encoding: base64
//! Content-Type: image/png
//! Content-Disposition: inline
//! 
//! ]]
//! CHUNK[[
//! PHNtaWxlLXJhdy1pbWFnZS1kYXRhPg==]]
//! CHUNK[[
//! 
//! ]]
//! CHUNK[[
//! --YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y--
//! ]]
//! CHUNK[[
//! --TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY--
//! ]]
//! CHUNK[[
//! --1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ
//! ]]
//! CHUNK[[
//! Content-Transfer-Encoding: 7bit
//! Content-Type: text/plain; charset=utf8
//! Content-Disposition: attachment; filename="example.c"
//! 
//! ]]
//! CHUNK[[
//! int main() { return 0; }]]
//! CHUNK[[
//! 
//! ]]
//! CHUNK[[
//! --1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ--
//! ]]
//! ```

extern crate emailaddress;
extern crate quoted_printable;
extern crate base64;
extern crate futures;
extern crate hyper;
extern crate mime;
extern crate textnonce;
extern crate tokio_proto;

#[cfg(test)]
extern crate tokio_core;

mod mailbox;
mod utf8_b;
pub mod header;
mod message;
mod encoder;
mod mimebody;

pub use mailbox::*;
pub use message::*;
pub use encoder::*;
pub use mimebody::*;

pub use hyper::{Body as MailBody, Chunk as BinaryChunk};

use futures::{Stream};

/// The stream of binary chunks
///
pub type BinaryStream<E> = Stream<Item = Vec<u8>, Error = E>;

# Email Message library for Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-brightgreen.svg)](https://opensource.org/licenses/MIT)
[![Crate](https://img.shields.io/crates/v/emailmessage.svg)](https://crates.io/crates/emailmessage)
[![Build Status](https://travis-ci.org/katyo/emailmessage-rs.svg?branch=master)](https://travis-ci.org/katyo/emailmessage-rs)

This project aims to provide a proper strongly typed way to build and parse emails.

## Features

* Typed headers using `hyper::Header`
* _TODO_ Streamed building and parsing the message body
* _TODO_ Compatibility with most mail delivery systems

## Usage

### Format email messages

#### With string body

The easiest way how we can create email message with simple string
(see [format\_string.rs](examples/format_string.rs)).

```rust
extern crate emailmessage;

use emailmessage::{header, Message};

fn main() {
    let m: Message<String> = Message::new()
        .with_header(header::From(vec!["Incognito <nobody@domain.tld>".parse().unwrap()]))
        .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
        .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
        .with_header(header::Subject("Happy new year".into()))
        .with_body("\r\nBe happy!");
    
    println!("{}", m);
}
```

Run this example:

```
$ cargo run --example format_string

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year

Be happy!
```

The unicode header data will be encoded using _UTF8-Base64_ encoding.

#### With mime body

##### Single part

The more complex way is using MIME contents
(see [format\_mime.rs](examples/format_mime.rs)).

```rust
extern crate emailmessage;

use emailmessage::{header, Message, SinglePart};

fn main() {
    let m: Message<SinglePart<String>> = Message::new()
        .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
        .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
        .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
        .with_header(header::Subject("Happy new year".into()))
        .with_header(header::MIME_VERSION_1_0)
        .with_body(
            SinglePart::new()
            .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
            .with_header(header::ContentTransferEncoding::QuotedPrintable)
            .with_body("Привет, мир!")
        );
    
    println!("{}", m);
}
```

The body will be encoded using selected `Content-Transfer-Encoding`.

```
$ cargo run --example format_mime

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year
MIME-Version: 1.0
Content-Type: text/plain; charset=utf8
Content-Transfer-Encoding: quoted-printable

=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!

```

##### Multiple parts

And the more advanced way is a using multipart MIME contents
(see [format\_multipart.rs](examples/format_multipart.rs)).

```rust
extern crate emailmessage;

use emailmessage::{header, Message, SinglePart, MultiPart};

fn main() {
    let m: Message<MultiPart<String>> = Message::new()
        .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
        .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
        .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
        .with_header(header::Subject("Happy new year".into()))
        .with_header(header::MIME_VERSION_1_0)
        .with_body(
            MultiPart::mixed()
            .with_multipart(
                MultiPart::alternative()
                .with_singlepart(
                    SinglePart::quoted_printable()
                    .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                    .with_body("Привет, мир!")
                )
                .with_multipart(
                    MultiPart::related()
                    .with_singlepart(
                        SinglePart::eight_bit()
                        .with_header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                        .with_body("<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>")
                    )
                    .with_singlepart(
                        SinglePart::base64()
                        .with_header(header::ContentType("image/png".parse().unwrap()))
                        .with_header(header::ContentDisposition {
                            disposition: header::DispositionType::Inline,
                            parameters: vec![],
                        })
                        .with_body("<smile-raw-image-data>")
                    )
                )
            )
            .with_singlepart(
                SinglePart::seven_bit()
                .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                .with_header(header::ContentDisposition {
                                 disposition: header::DispositionType::Attachment,
                                 parameters: vec![
                                     header::DispositionParam::Filename(
                                         header::Charset::Ext("utf-8".into()),
                                         None, "example.c".as_bytes().into()
                                     )
                                 ]
                             })
                .with_body(String::from("int main() { return 0; }"))
            )
        );
    
    println!("{}", m);
}
```

```
$ cargo run --example format_multipart

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m"

--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m
Content-Type: multipart/alternative; boundary="qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy"

--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy
Content-Transfer-Encoding: quoted-printable
Content-Type: text/plain; charset=utf8

=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!
--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy
Content-Type: multipart/related; boundary="BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8"

--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8
Content-Transfer-Encoding: 8bit
Content-Type: text/html; charset=utf8

<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>
--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8
Content-Transfer-Encoding: base64
Content-Type: image/png
Content-Disposition: inline

PHNtaWxlLXJhdy1pbWFnZS1kYXRhPg==
--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8--
--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy--
--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m
Content-Transfer-Encoding: 7bit
Content-Type: text/plain; charset=utf8
Content-Disposition: attachment; filename="example.c"

int main() { return 0; }
--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m--

```

# Email Message library for Rust

[![License: MIT](https://img.shields.io/badge/License-MIT.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://travis-ci.org/katyo/emailmessage-rs.svg?branch=master)](https://travis-ci.org/katyo/emailmessage-rs)

This project aims to provide a proper strongly typed way to build and parse emails.

## Features

* Typed headers using `hyper::Header`
* _TODO_ Streamed building and parsing the message body
* _TODO_ Compatibility with most mail delivery systems

## Usage

### Format email messages

#### With string body

```rust
extern crate emailmessage;

use emailmessage::{header, Message};

fn main() {
    let m: Message<String> = Message::new()
        .with_header(header::From("NoBody <nobody@domain.tld>".parse().unwrap()))
        .with_header(header::ReplyTo("Yuin <yuin@domain.tld>".parse().unwrap()))
        .with_header(header::To("Hei <hei@domain.tld>".parse().unwrap()))
        .with_header(header::Subject("Happy new year".into()))
        .with_body("\r\nBe happy!".into());
    
    println!("{}", m);
}
```

#### With mime body

##### Single part

```rust
extern crate emailmessage;

use emailmessage::{header, Message, SinglePart};

fn main() {
    let m: Message<SinglePart<String>> = Message::new()
        .with_header(header::From("NoBody <nobody@domain.tld>".parse().unwrap()))
        .with_header(header::ReplyTo("Yuin <yuin@domain.tld>".parse().unwrap()))
        .with_header(header::To("Hei <hei@domain.tld>".parse().unwrap()))
        .with_header(header::Subject("Happy new year".into()))
        .with_header(header::MIME_VERSION_1_0)
        .with_body(SinglePart::new()
                   .with_header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                   .with_header(header::ContentTransferEncoding::QuotedPrintable)
                   .with_body("Привет, мир!"));
    
    println!("{}", m);
}
```

##### Multiple parts

```rust
extern crate emailmessage;

use emailmessage::{header, Message, SinglePart, MultiPart};

fn main() {
    let m: Message<MultiPart<String>> = Message::new()
        .with_header(header::From("NoBody <nobody@domain.tld>".parse().unwrap()))
        .with_header(header::ReplyTo("Yuin <yuin@domain.tld>".parse().unwrap()))
        .with_header(header::To("Hei <hei@domain.tld>".parse().unwrap()))
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
                        .with_body(smile_raw)
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

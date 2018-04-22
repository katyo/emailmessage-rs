# Email Message library for Rust

This project aims to provide a proper strongly typed way to build and parse emails.

## Features

* Typed headers using `hyper::Header`
* _TODO_ Streamed building and parsing the message body
* _TODO_ Compatibility with most mail delivery systems

## Usage

```rust
extern crate emailmessage;

use emailmessage::{header, Message};

fn main() {
    let m = Message::new()
    // add From: header
        .with_header(header::From("NoBody <nobody@domain.tld>".parse().unwrap()))
        .with_header(header::ReplyTo("Yuin <yuin@domain.tld>".parse().unwrap()))
        .with_header(header::To("Hei <hei@domain.tld>".parse().unwrap()))
        .with_header(header::Subject("Happy new year".into()))
        .with_body("Be happy!".into());
    
    println!("{}", m);
}
```

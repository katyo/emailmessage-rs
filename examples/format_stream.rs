extern crate futures;
extern crate tokio_core;
extern crate emailmessage;

use std::str::from_utf8;
use futures::{Stream};
use tokio_core::reactor::{Core};
use emailmessage::{header, Message, BinaryStream};

fn main() {
    let mut core = Core::new().unwrap();
    
    let m: Message = Message::new()
        .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
        .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
        .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
        .with_header(header::Subject("Happy new year".into()))
        .with_body("\r\nBe happy!");
    
    let f = Into::<Box<BinaryStream<_>>>::into(m).map(|chunk| {
        println!("CHUNK[[\n{}]]", from_utf8(&chunk).unwrap());
        chunk
    }).concat2();

    core.run(f).unwrap();
}

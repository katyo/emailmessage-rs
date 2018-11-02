extern crate emailmessage;
extern crate futures;
extern crate tokio;

use emailmessage::{header, BinaryStream, Message};
use futures::{Future, Stream};
use std::str::from_utf8;
use tokio::run;

fn main() {
    let m: Message = Message::new()
        .with_header(header::From(vec![
            "NoBody <nobody@domain.tld>".parse().unwrap(),
        ])).with_header(header::ReplyTo(vec![
            "Yuin <yuin@domain.tld>".parse().unwrap(),
        ])).with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
        .with_header(header::Subject("Happy new year".into()))
        .with_body("\r\nBe happy!");

    let f = Into::<Box<BinaryStream<_>>>::into(m)
        .map(|chunk| {
            println!("CHUNK[[\n{}]]", from_utf8(&chunk).unwrap());
            chunk
        }).concat2()
        .map(|message| {
            println!("MESSAGE[[\n{}]]", from_utf8(&message).unwrap());
        }).map_err(|error| {
            eprintln!("ERROR: {}", error);
        });

    run(f);
}

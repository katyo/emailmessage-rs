extern crate emailmessage;
extern crate futures;
extern crate tokio;

use emailmessage::Message;
use futures::{Future, Stream};
use std::str::from_utf8;
use tokio::run;

fn main() {
    let m: Message = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!".into());

    let f = m
        .into_stream()
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

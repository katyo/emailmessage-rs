extern crate emailmessage;

use emailmessage::{header, Message};

fn main() {
    let m: Message<String> = Message::new()
        .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
        .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
        .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
        .with_header(header::Subject("Happy new year".into()))
        .with_body("\r\nBe happy!");
    
    println!("{}", m);
}

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

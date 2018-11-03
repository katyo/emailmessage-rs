extern crate emailmessage;

use emailmessage::{header, Message, SinglePart};

fn main() {
    let m: Message<SinglePart<&str>> = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .mime_body(
            SinglePart::builder()
                .header(header::ContentType(
                    "text/plain; charset=utf8".parse().unwrap(),
                )).header(header::ContentTransferEncoding::QuotedPrintable)
                .body("Привет, мир!"),
        );

    println!("{}", m);
}

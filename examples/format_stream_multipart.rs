extern crate futures;
extern crate tokio_core;
extern crate emailmessage;

use std::str::from_utf8;
use futures::{Stream};
use tokio_core::reactor::{Core};
use emailmessage::{header, Message, SinglePart, MultiPart, EncodedBinaryStream};

fn main() {
    let mut core = Core::new().unwrap();

    let b: MultiPart = MultiPart::mixed()
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
        );
    
    let m: Message<Box<EncodedBinaryStream<_>>> = Message::new()
        .with_header(header::From(vec!["NoBody <nobody@domain.tld>".parse().unwrap()]))
        .with_header(header::ReplyTo(vec!["Yuin <yuin@domain.tld>".parse().unwrap()]))
        .with_header(header::To(vec!["Hei <hei@domain.tld>".parse().unwrap()]))
        .with_header(header::Subject("Happy new year".into()))
        .with_header(header::MIME_VERSION_1_0)
        .with_body(b);

    let f = Into::<Box<EncodedBinaryStream<_>>>::into(m).map(|chunk| {
        println!("CHUNK[[\n{}]]", from_utf8(&chunk).unwrap());
        chunk
    }).concat2();

    core.run(f).unwrap();
}

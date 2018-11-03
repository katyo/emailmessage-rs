extern crate emailmessage;
extern crate futures;
extern crate tokio;

use emailmessage::{header, Message, MultiPart, SinglePart};
use futures::{Future, Stream};
use std::str::from_utf8;
use tokio::run;

fn main() {
    let b: MultiPart = MultiPart::mixed()
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::quoted_printable()
                        .header(header::ContentType(
                            "text/plain; charset=utf8".parse().unwrap(),
                        )).body("Привет, мир!".into()),
                ).multipart(
                    MultiPart::related()
                        .singlepart(
                            SinglePart::eight_bit()
                                .header(header::ContentType(
                                    "text/html; charset=utf8".parse().unwrap(),
                                )).body(
                                    "<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>".into(),
                                ),
                        ).singlepart(
                            SinglePart::base64()
                                .header(header::ContentType("image/png".parse().unwrap()))
                                .header(header::ContentDisposition {
                                    disposition: header::DispositionType::Inline,
                                    parameters: vec![],
                                }).body("<smile-raw-image-data>".into()),
                        ),
                ),
        ).singlepart(
            SinglePart::seven_bit()
                .header(header::ContentType(
                    "text/plain; charset=utf8".parse().unwrap(),
                )).header(header::ContentDisposition {
                    disposition: header::DispositionType::Attachment,
                    parameters: vec![header::DispositionParam::Filename(
                        header::Charset::Ext("utf-8".into()),
                        None,
                        "example.c".as_bytes().into(),
                    )],
                }).body("int main() { return 0; }".into()),
        );

    let m = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .mime_1_0()
        .body(b.into_stream());

    let f = m
        .into_stream()
        .map(|chunk| {
            println!("CHUNK[[\n{}]]", from_utf8(&chunk).unwrap());
            chunk
        }).concat2()
        .map(|message| {
            println!("MESSSAGE[[\n{}]]", from_utf8(&message).unwrap());
        }).map_err(|error| {
            eprintln!("ERROR: {:?}", error);
        });

    run(f);
}

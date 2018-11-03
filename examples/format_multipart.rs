extern crate emailmessage;

use emailmessage::{header, Message, MultiPart, SinglePart};

fn main() {
    let m: Message<MultiPart<&str>> = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .mime_1_0()
        .body(
            MultiPart::mixed()
            .multipart(
                MultiPart::alternative()
                .singlepart(
                    SinglePart::quoted_printable()
                    .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                    .body("Привет, мир!")
                )
                .multipart(
                    MultiPart::related()
                    .singlepart(
                        SinglePart::eight_bit()
                        .header(header::ContentType("text/html; charset=utf8".parse().unwrap()))
                        .body("<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>")
                    )
                    .singlepart(
                        SinglePart::base64()
                        .header(header::ContentType("image/png".parse().unwrap()))
                        .header(header::ContentDisposition {
                            disposition: header::DispositionType::Inline,
                            parameters: vec![],
                        })
                        .body("<smile-raw-image-data>")
                    )
                )
            )
            .singlepart(
                SinglePart::seven_bit()
                .header(header::ContentType("text/plain; charset=utf8".parse().unwrap()))
                .header(header::ContentDisposition {
                                 disposition: header::DispositionType::Attachment,
                                 parameters: vec![
                                     header::DispositionParam::Filename(
                                         header::Charset::Ext("utf-8".into()),
                                         None, "example.c".as_bytes().into()
                                     )
                                 ]
                             })
                .body("int main() { return 0; }")
            )
        );

    println!("{}", m);
}

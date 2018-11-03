extern crate emailmessage;

use emailmessage::Message;

fn main() {
    let m: Message<&str> = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!");

    println!("{}", m);
}

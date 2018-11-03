/*!

# Email Message library for Rust

This project aims to provide a proper strongly typed way to build emails.

## Features

* Typed headers using [`hyperx::Header`]
* Support for MIME 1.0 multipart contents
* Streaming messages to save memory usage

## Usage

### Format email messages

#### With string body

The easiest way how we can create email message with simple string
(see [format\_string.rs](examples/format_string.rs)).

```rust
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
```

Run this example:

```sh
$ cargo run --example format_string

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year

Be happy!
```

The unicode header data will be encoded using _UTF8-Base64_ encoding.

#### With mime body

##### Single part

The more complex way is using MIME contents
(see [format\_mime.rs](examples/format_mime.rs)).

```rust
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
```

The body will be encoded using selected `Content-Transfer-Encoding`.

```sh
$ cargo run --example format_mime

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year
MIME-Version: 1.0
Content-Type: text/plain; charset=utf8
Content-Transfer-Encoding: quoted-printable

=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!

```

##### Multiple parts

And more advanced way of building message by using multipart MIME contents
(see [format\_multipart.rs](examples/format_multipart.rs)).

```rust
extern crate emailmessage;

use emailmessage::{header, Message, MultiPart, SinglePart};

fn main() {
    let m: Message<MultiPart<&str>> = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .mime_body(
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
```

```sh
$ cargo run --example format_multipart

From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m"

--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m
Content-Type: multipart/alternative; boundary="qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy"

--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy
Content-Transfer-Encoding: quoted-printable
Content-Type: text/plain; charset=utf8

=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!
--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy
Content-Type: multipart/related; boundary="BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8"

--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8
Content-Transfer-Encoding: 8bit
Content-Type: text/html; charset=utf8

<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>
--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8
Content-Transfer-Encoding: base64
Content-Type: image/png
Content-Disposition: inline

PHNtaWxlLXJhdy1pbWFnZS1kYXRhPg==
--BV5RCn9p31oAAAAAUt42E9bYMDEAGCOWlxEz89Bv0qFA5Xsy6rOC3zRahMQ39IFZNnp8--
--qW9QCn9p31oAAAAAodFBg1L1Qrraa5hEl0bDJ6kfJMUcRT2LLSWEoeyhSEbUBIqbjWqy--
--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m
Content-Transfer-Encoding: 7bit
Content-Type: text/plain; charset=utf8
Content-Disposition: attachment; filename="example.c"

int main() { return 0; }
--RTxPCn9p31oAAAAAeQxtr1FbXr/i5vW1hFlH9oJqZRMWxRMK1QLjQ4OPqFk9R+0xUb/m--

```

#### Using streaming

A memory-efficient way of building and sending relatively big emails is a using streaming.

Say, you would like to send server access logs in attachments,
HTML-documents with related media resources or PDFs which may be sourced from external files.

In examples above we actually allocated formatted emails in memory,
but usually we don't want to do same for big emails which size measures in MBytes.

##### Simple string

The simple example below shows actually sent chunks of streamed message
(see [format\_stream.rs](examples/format_stream.rs)).

```rust
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
```

```sh
$ cargo run --example format_stream

CHUNK[[
From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year
]]
CHUNK[[

Be happy!]]
MESSAGE[[
From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year

Be happy!]]
```

In real world app we may do some buffering of stream to prevent too short and too long sendings.

##### Multipart data

(see [format\_stream\_multipart.rs](examples/format_stream_multipart.rs))

```rust
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
        .mime_body(b.into_stream());

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
```

```sh
$ cargo run --example format_stream_multipart

CHUNK[[
From: NoBody <nobody@domain.tld>
Reply-To: Yuin <yuin@domain.tld>
To: Hei <hei@domain.tld>
Subject: Happy new year
MIME-Version: 1.0
]]
CHUNK[[
Content-Type: multipart/mixed; boundary="1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ"
]]
CHUNK[[
--1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ
]]
CHUNK[[
--1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ
]]
CHUNK[[
Content-Type: multipart/alternative; boundary="TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY"
]]
CHUNK[[
--TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY
]]
CHUNK[[
--TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY
]]
CHUNK[[
Content-Transfer-Encoding: quoted-printable
Content-Type: text/plain; charset=utf8
]]
CHUNK[[
=D0=9F=D1=80=D0=B8=D0=B2=D0=B5=D1=82, =D0=BC=D0=B8=D1=80!]]
CHUNK[[
]]
CHUNK[[
--TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY
]]
CHUNK[[
Content-Type: multipart/related; boundary="YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y"
]]
CHUNK[[
--YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y
]]
CHUNK[[
--YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y
]]
CHUNK[[
Content-Transfer-Encoding: 8bit
Content-Type: text/html; charset=utf8
]]
CHUNK[[
<p><b>Hello</b>, <i>world</i>! <img src=smile.png></p>]]
CHUNK[[
]]
CHUNK[[
--YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y
]]
CHUNK[[
Content-Transfer-Encoding: base64
Content-Type: image/png
Content-Disposition: inline

]]
CHUNK[[
PHNtaWxlLXJhdy1pbWFnZS1kYXRhPg==]]
CHUNK[[

]]
CHUNK[[
--YsgeCMR/31oAAAAAanzeyu/dFJGjfzDxpsAOLhRB0RfSw+DXefQybZxGq6HIBEzotZ5Y--
]]
CHUNK[[
--TCMeCMR/31oAAAAAmf7KBuXt4qRk2RnBJCj8YJNdwm2dsadXxjOlC74hlb1tO6U/SqXY--
]]
CHUNK[[
--1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ
]]
CHUNK[[
Content-Transfer-Encoding: 7bit
Content-Type: text/plain; charset=utf8
Content-Disposition: attachment; filename="example.c"

]]
CHUNK[[
int main() { return 0; }]]
CHUNK[[

]]
CHUNK[[
--1S8dCMR/31oAAAAApHRNMETjK2uRsQs4mVVFKVNujcqnm8FHOXWvqARiaYy9ZmnpQ7uQ--
]]
...
```
 */

extern crate base64;
extern crate bytes;
extern crate emailaddress;
extern crate futures;
extern crate hyper;
extern crate hyperx;
extern crate mime;
extern crate quoted_printable;
extern crate textnonce;

#[cfg(test)]
extern crate tokio;

mod encoder;
pub mod header;
mod mailbox;
mod message;
mod mimebody;
mod utf8_b;

pub use encoder::*;
pub use mailbox::*;
pub use message::*;
pub use mimebody::*;

pub use hyper::{Body, Chunk};

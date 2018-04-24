extern crate emailaddress;
extern crate quoted_printable;
extern crate base64;
extern crate futures;
extern crate hyper;
extern crate mime;
extern crate textnonce;
extern crate tokio_proto;

#[cfg(test)]
extern crate tokio_core;

mod mailbox;
mod utf8_b;
pub mod header;
mod message;
mod encoder;
mod mimebody;

pub use mailbox::*;
pub use message::*;
pub use encoder::*;
pub use mimebody::*;

pub use hyper::{Body as MailBody, Chunk as BinaryChunk};

use futures::{Stream};

/// The stream of binary chunks
///
pub type BinaryStream<E> = Stream<Item = Vec<u8>, Error = E>;

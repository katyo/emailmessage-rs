use std::str::{FromStr, from_utf8};
use std::fmt::{Display, Formatter as FmtFormatter, Result as FmtResult};
use hyper::header::{Raw, Header, Formatter as HeaderFormatter};
use hyper::{Result as HyperResult, Error as HyperError};

#[derive(Debug, Clone, PartialEq)]
pub enum ContentTransferEncoding {
    SevenBit,
    QuotedPrintable,
    Base64,
    // 8BITMIME
    EightBit,
    Binary,
}

impl Default for ContentTransferEncoding {
    fn default() -> Self {
        ContentTransferEncoding::SevenBit
    }
}

impl Display for ContentTransferEncoding {
    fn fmt(&self, f: &mut FmtFormatter) -> FmtResult {
        use self::ContentTransferEncoding::*;
        f.write_str(match *self {
            SevenBit => "7bit",
            QuotedPrintable => "quoted-printable",
            Base64 => "base64",
            EightBit => "8bit",
            Binary => "binary",
        })
    }
}

impl FromStr for ContentTransferEncoding {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::ContentTransferEncoding::*;
        match s {
            "7bit" => Ok(SevenBit),
            "quoted-printable" => Ok(QuotedPrintable),
            "base64" => Ok(Base64),
            "8bit" => Ok(EightBit),
            "binary" => Ok(Binary),
            _ => Err(s.into())
        }
    }
}

impl Header for ContentTransferEncoding {
    fn header_name() -> &'static str {
        "MIME-Version"
    }
    
    fn parse_header(raw: &Raw) -> HyperResult<Self> {
        raw.one().ok_or(HyperError::Header)
            .and_then(|r| from_utf8(r).map_err(|_| HyperError::Header))
            .and_then(|s| s.parse::<ContentTransferEncoding>().map_err(|_| HyperError::Header))
    }
    
    fn fmt_header(&self, f: &mut HeaderFormatter) -> FmtResult {
        f.fmt_line(&format!("{}", self))
    }
}

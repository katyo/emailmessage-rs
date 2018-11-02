use hyperx::{
    header::{Formatter as HeaderFormatter, Header, Raw},
    Error as HyperError, Result as HyperResult,
};
use mailbox::Mailbox;
use std::fmt::Result as FmtResult;
use std::str::from_utf8;
use utf8_b;

macro_rules! mailbox_header {
    ( $type_name: ident, $header_name: expr ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type_name(pub Mailbox);

        impl Header for $type_name {
            fn header_name() -> &'static str {
                $header_name
            }

            fn parse_header(raw: &Raw) -> HyperResult<$type_name> {
                raw.one()
                    .ok_or(HyperError::Header)
                    .and_then(parse_mailboxes)
                    .and_then(|l| {
                        if l.is_empty() {
                            Err(HyperError::Header)
                        } else {
                            Ok(l[0].clone())
                        }
                    }).map($type_name)
            }

            fn fmt_header(&self, f: &mut HeaderFormatter) -> FmtResult {
                fmt_mailboxes(&[self.0.clone()], f)
            }
        }
    };
}

macro_rules! mailboxes_header {
    ( $type_name: ident, $header_name: expr ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type_name(pub Vec<Mailbox>);

        impl Header for $type_name {
            fn header_name() -> &'static str {
                $header_name
            }

            fn parse_header(raw: &Raw) -> HyperResult<$type_name> {
                raw.one()
                    .ok_or(HyperError::Header)
                    .and_then(parse_mailboxes)
                    .and_then(|l| {
                        if l.is_empty() {
                            Err(HyperError::Header)
                        } else {
                            Ok(l)
                        }
                    }).map($type_name)
            }

            fn fmt_header(&self, f: &mut HeaderFormatter) -> FmtResult {
                fmt_mailboxes(&self.0, f)
            }
        }
    };
}

mailboxes_header!(From, "From");
mailbox_header!(Sender, "Sender");
mailboxes_header!(ReplyTo, "Reply-To");

mailboxes_header!(To, "To");
mailboxes_header!(Cc, "Cc");
mailboxes_header!(Bcc, "Bcc");

fn parse_mailboxes(raw: &[u8]) -> HyperResult<Vec<Mailbox>> {
    if let Ok(src) = from_utf8(raw) {
        if let Ok(mbs) = src
            .split(',')
            .map(|m| {
                m.trim()
                    .parse()
                    .map_err(|_| ())
                    .and_then(|Mailbox { name, addr }| {
                        if let Some(name) = name {
                            if let Some(name) = utf8_b::decode(&name) {
                                return Ok(Mailbox::new(Some(name), addr));
                            }
                        } else {
                            return Ok(Mailbox::new(None, addr));
                        }
                        Err(())
                    })
            }).collect()
        {
            return Ok(mbs);
        }
    }
    Err(HyperError::Header)
}

fn fmt_mailboxes(m: &[Mailbox], f: &mut HeaderFormatter) -> FmtResult {
    f.fmt_line(&m.iter().fold(String::new(), |s, m| {
        let m = m.recode_name(utf8_b::encode);
        if s.is_empty() {
            format!("{}", m)
        } else {
            format!("{}, {}", s, m)
        }
    }))
}

#[cfg(test)]
mod test {
    use super::{From, Mailbox};
    use hyperx::Headers;

    #[test]
    fn format_single_without_name() {
        let from = vec!["kayo@example.com".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(format!("{}", headers), "From: kayo@example.com\r\n");
    }

    #[test]
    fn format_single_with_name() {
        let from = vec!["K. <kayo@example.com>".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(format!("{}", headers), "From: K. <kayo@example.com>\r\n");
    }

    #[test]
    fn format_multi_without_name() {
        let from = vec![
            "kayo@example.com".parse().unwrap(),
            "pony@domain.tld".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(
            format!("{}", headers),
            "From: kayo@example.com, pony@domain.tld\r\n"
        );
    }

    #[test]
    fn format_multi_with_name() {
        let from = vec![
            "K. <kayo@example.com>".parse().unwrap(),
            "Pony P. <pony@domain.tld>".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(
            format!("{}", headers),
            "From: K. <kayo@example.com>, Pony P. <pony@domain.tld>\r\n"
        );
    }

    #[test]
    fn format_single_with_utf8_name() {
        let from = vec!["Кайо <kayo@example.com>".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set(From(from));

        assert_eq!(
            format!("{}", headers),
            "From: =?utf-8?b?0JrQsNC50L4=?= <kayo@example.com>\r\n"
        );
    }

    #[test]
    fn parse_single_without_name() {
        let from: Vec<Mailbox> = vec!["kayo@example.com".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set_raw("From", "kayo@example.com");

        assert_eq!(headers.get::<From>(), Some(&From(from)));
    }

    #[test]
    fn parse_single_with_name() {
        let from: Vec<Mailbox> = vec!["K. <kayo@example.com>".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set_raw("From", "K. <kayo@example.com>");

        assert_eq!(headers.get::<From>(), Some(&From(from)));
    }

    #[test]
    fn parse_multi_without_name() {
        let from: Vec<Mailbox> = vec![
            "kayo@example.com".parse().unwrap(),
            "pony@domain.tld".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set_raw("From", "kayo@example.com, pony@domain.tld");

        assert_eq!(headers.get::<From>(), Some(&From(from)));
    }

    #[test]
    fn parse_multi_with_name() {
        let from: Vec<Mailbox> = vec![
            "K. <kayo@example.com>".parse().unwrap(),
            "Pony P. <pony@domain.tld>".parse().unwrap(),
        ];

        let mut headers = Headers::new();
        headers.set_raw("From", "K. <kayo@example.com>, Pony P. <pony@domain.tld>");

        assert_eq!(headers.get::<From>(), Some(&From(from)));
    }

    #[test]
    fn parse_single_with_utf8_name() {
        let from: Vec<Mailbox> = vec!["Кайо <kayo@example.com>".parse().unwrap()];

        let mut headers = Headers::new();
        headers.set_raw("From", "=?utf-8?b?0JrQsNC50L4=?= <kayo@example.com>");

        assert_eq!(headers.get::<From>(), Some(&From(from)));
    }
}

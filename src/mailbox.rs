pub use emailaddress::{AddrError, EmailAddress as Address};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::slice::Iter;
use std::str::FromStr;
use utf8_b;

/// Email address with addressee name
#[derive(Debug, Clone, PartialEq)]
pub struct Mailbox {
    pub name: Option<String>,
    pub addr: Address,
}

impl Mailbox {
    /// Create new mailbox using email address and addressee name
    pub fn new(name: Option<String>, addr: Address) -> Self {
        Mailbox { name, addr }
    }

    /// Encode addressee name using function
    pub(crate) fn recode_name<F>(&self, f: F) -> Self
    where
        F: FnOnce(&str) -> String,
    {
        Mailbox::new(self.name.clone().map(|s| f(&s)), self.addr.clone())
    }
}

impl Display for Mailbox {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if let Some(ref name) = self.name {
            let name = name.trim();
            if !name.is_empty() {
                return write!(f, "{} <{}>", name, self.addr);
            }
        }

        write!(f, "{}", self.addr)
    }
}

impl FromStr for Mailbox {
    type Err = String;

    fn from_str(src: &str) -> Result<Mailbox, Self::Err> {
        match (src.find('<'), src.find('>')) {
            (Some(addr_open), Some(addr_close)) => {
                let name = src.split_at(addr_open).0;
                let addr_open = addr_open + 1;
                let addr = src.split_at(addr_open).1.split_at(addr_close - addr_open).0;
                let addr = addr.parse().map_err(|AddrError { msg }| msg)?;
                let name = name.trim();
                let name = if name.is_empty() {
                    None
                } else {
                    Some(name.into())
                };
                Ok(Mailbox::new(name, addr))
            }
            _ => {
                let addr = src.parse().map_err(|AddrError { msg }| msg)?;
                Ok(Mailbox::new(None, addr))
            }
        }
    }
}

/// List or email mailboxes
#[derive(Debug, Clone, PartialEq)]
pub struct Mailboxes(Vec<Mailbox>);

impl Mailboxes {
    /// Create mailboxes list
    pub fn new() -> Self {
        Mailboxes(Vec::new())
    }

    /// Add mailbox to a list
    pub fn add(mut self, mbox: Mailbox) -> Self {
        self.0.push(mbox);
        self
    }

    /// Extract first mailbox
    pub fn into_single(self) -> Option<Mailbox> {
        self.into()
    }

    /// Iterate over mailboxes
    pub fn iter<'a>(&'a self) -> Iter<'a, Mailbox> {
        self.0.iter()
    }
}

impl From<Mailbox> for Mailboxes {
    fn from(single: Mailbox) -> Self {
        Mailboxes(vec![single])
    }
}

impl Into<Option<Mailbox>> for Mailboxes {
    fn into(self) -> Option<Mailbox> {
        self.into_iter().next()
    }
}

impl From<Vec<Mailbox>> for Mailboxes {
    fn from(list: Vec<Mailbox>) -> Self {
        Mailboxes(list)
    }
}

impl Into<Vec<Mailbox>> for Mailboxes {
    fn into(self) -> Vec<Mailbox> {
        self.0
    }
}

impl IntoIterator for Mailboxes {
    type Item = Mailbox;
    type IntoIter = ::std::vec::IntoIter<Mailbox>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<Mailbox> for Mailboxes {
    fn extend<T: IntoIterator<Item = Mailbox>>(&mut self, iter: T) {
        for elem in iter {
            self.0.push(elem);
        }
    }
}

impl Display for Mailboxes {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut iter = self.iter();

        if let Some(mbox) = iter.next() {
            mbox.fmt(f)?;

            while let Some(mbox) = iter.next() {
                f.write_str(", ")?;
                mbox.fmt(f)?;
            }
        }

        Ok(())
    }
}

impl FromStr for Mailboxes {
    type Err = String;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        src.split(',')
            .map(|m| {
                m.trim().parse().and_then(|Mailbox { name, addr }| {
                    if let Some(name) = name {
                        if let Some(name) = utf8_b::decode(&name) {
                            Ok(Mailbox::new(Some(name), addr))
                        } else {
                            Err("Unable to decode utf8b".into())
                        }
                    } else {
                        Ok(Mailbox::new(None, addr))
                    }
                })
            }).collect::<Result<Vec<_>, _>>()
            .map(Mailboxes)
    }
}

#[cfg(test)]
mod test {
    use super::{Address, Mailbox};

    #[test]
    fn mailbox_format_address_only() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(None, Address::new("kayo@example.com").unwrap())
            ),
            "kayo@example.com"
        );
    }

    #[test]
    fn mailbox_format_address_with_name() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(Some("K.".into()), Address::new("kayo@example.com").unwrap())
            ),
            "K. <kayo@example.com>"
        );
    }

    #[test]
    fn format_address_with_empty_name() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(Some("".into()), Address::new("kayo@example.com").unwrap())
            ),
            "kayo@example.com"
        );
    }

    #[test]
    fn format_address_with_name_trim() {
        assert_eq!(
            format!(
                "{}",
                Mailbox::new(
                    Some(" K. ".into()),
                    Address::new("kayo@example.com").unwrap()
                )
            ),
            "K. <kayo@example.com>"
        );
    }

    #[test]
    fn parse_address_only() {
        assert_eq!(
            "kayo@example.com".parse(),
            Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap()))
        );
    }

    #[test]
    fn parse_address_with_name() {
        assert_eq!(
            "K. <kayo@example.com>".parse(),
            Ok(Mailbox::new(
                Some("K.".into()),
                "kayo@example.com".parse().unwrap()
            ))
        );
    }

    #[test]
    fn parse_address_with_empty_name() {
        assert_eq!(
            "<kayo@example.com>".parse(),
            Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap()))
        );
    }

    #[test]
    fn parse_address_with_empty_name_trim() {
        assert_eq!(
            " <kayo@example.com>".parse(),
            Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap()))
        );
    }
}

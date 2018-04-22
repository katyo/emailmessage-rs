use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::{FromStr};
pub use emailaddress::{EmailAddress as Address, AddrError};

#[derive(Clone, PartialEq, Debug)]
pub struct Mailbox {
    pub name: Option<String>,
    pub addr: Address,
}

impl Mailbox {
    pub fn new(name: Option<String>, addr: Address) -> Self {
        Mailbox { name, addr }
    }

    pub fn recode_name<F>(&self, f: F) -> Self
    where F: FnOnce(&str) -> String
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
                let name = if name.is_empty() { None } else { Some(name.into()) };
                Ok(Mailbox::new(name, addr))
            },
            _ => {
                let addr = src.parse().map_err(|AddrError { msg }| msg)?;
                Ok(Mailbox::new(None, addr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Address, Mailbox};

    #[test]
    fn mailbox_format_address_only() {
        assert_eq!(format!("{}", Mailbox::new(None, Address::new("kayo@example.com").unwrap())), "kayo@example.com");
    }

    #[test]
    fn mailbox_format_address_with_name() {
        assert_eq!(format!("{}", Mailbox::new(Some("K.".into()), Address::new("kayo@example.com").unwrap())), "K. <kayo@example.com>");
    }

    #[test]
    fn format_address_with_empty_name() {
        assert_eq!(format!("{}", Mailbox::new(Some("".into()), Address::new("kayo@example.com").unwrap())), "kayo@example.com");
    }

    #[test]
    fn format_address_with_name_trim() {
        assert_eq!(format!("{}", Mailbox::new(Some(" K. ".into()), Address::new("kayo@example.com").unwrap())), "K. <kayo@example.com>");
    }

    #[test]
    fn parse_address_only() {
        assert_eq!("kayo@example.com".parse(), Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap())));
    }

    #[test]
    fn parse_address_with_name() {
        assert_eq!("K. <kayo@example.com>".parse(), Ok(Mailbox::new(Some("K.".into()), "kayo@example.com".parse().unwrap())));
    }

    #[test]
    fn parse_address_with_empty_name() {
        assert_eq!("<kayo@example.com>".parse(), Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap())));
    }

    #[test]
    fn parse_address_with_empty_name_trim() {
        assert_eq!(" <kayo@example.com>".parse(), Ok(Mailbox::new(None, "kayo@example.com".parse().unwrap())));
    }
}

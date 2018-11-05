use super::MailboxError;
use idna::domain_to_ascii;
use regex::Regex;
use std::net::IpAddr;

lazy_static! {
    // Regex from the specs
    // https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
    // It will mark esoteric email addresses like quoted string as invalid
    static ref USER_RE: Regex = Regex::new(r"^(?i)[a-z0-9.!#$%&'*+/=?^_`{|}~-]+\z").unwrap();
    static ref DOMAIN_RE: Regex = Regex::new(
        r"(?i)^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*$"
    ).unwrap();
    // literal form, ipv4 or ipv6 address (SMTP 4.1.3)
    static ref LITERAL_RE: Regex = Regex::new(r"(?i)\[([A-f0-9:\.]+)\]\z").unwrap();
}

pub fn check_user(user: &str) -> Result<(), MailboxError> {
    if USER_RE.is_match(user) {
        Ok(())
    } else {
        Err(MailboxError::InvalidUser)
    }
}

pub fn check_domain(domain: &str) -> Result<(), MailboxError> {
    check_domain_ascii(domain).or_else(|_| {
        domain_to_ascii(domain)
            .map_err(|_| MailboxError::InvalidDomain)
            .and_then(|domain| check_domain_ascii(&domain))
    })
}

fn check_domain_ascii(domain: &str) -> Result<(), MailboxError> {
    use self::MailboxError::*;

    if DOMAIN_RE.is_match(domain) {
        return Ok(());
    }

    if let Some(caps) = LITERAL_RE.captures(domain) {
        if let Some(cap) = caps.get(1) {
            if cap.as_str().parse::<IpAddr>().is_ok() {
                return Ok(());
            }
        }
    }

    Err(InvalidDomain)
}

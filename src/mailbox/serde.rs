use super::{
    check::{check_domain, check_user},
    Address, Mailbox, Mailboxes,
};
#[cfg(feature = "serde")]
use serde::{
    de::{Deserializer, Error as DeError, MapAccess, SeqAccess, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
use std::fmt::{Formatter, Result as FmtResult};

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            User,
            Domain,
        };

        const FIELDS: &'static [&'static str] = &["user", "domain"];

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                        formatter.write_str("'user' or 'domain'")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: DeError,
                    {
                        match value {
                            "user" => Ok(Field::User),
                            "domain" => Ok(Field::Domain),
                            _ => Err(DeError::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct AddressVisitor;

        impl<'de> Visitor<'de> for AddressVisitor {
            type Value = Address;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("email address string or object")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                s.parse().map_err(DeError::custom)
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut user = None;
                let mut domain = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::User => {
                            if user.is_some() {
                                return Err(DeError::duplicate_field("user"));
                            }
                            let val = map.next_value()?;
                            check_user(val).map_err(DeError::custom)?;
                            user = Some(val);
                        }
                        Field::Domain => {
                            if domain.is_some() {
                                return Err(DeError::duplicate_field("domain"));
                            }
                            let val = map.next_value()?;
                            check_domain(val).map_err(DeError::custom)?;
                            domain = Some(val);
                        }
                    }
                }
                let user: &str = user.ok_or_else(|| DeError::missing_field("user"))?;
                let domain: &str = domain.ok_or_else(|| DeError::missing_field("domain"))?;
                Ok(Address::new(user, domain))
            }
        }

        deserializer.deserialize_any(AddressVisitor)
    }
}

impl Serialize for Mailbox {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Mailbox {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Name,
            Email,
        };

        const FIELDS: &'static [&'static str] = &["name", "email"];

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                        formatter.write_str("'name' or 'email'")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: DeError,
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            "email" => Ok(Field::Email),
                            _ => Err(DeError::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct MailboxVisitor;

        impl<'de> Visitor<'de> for MailboxVisitor {
            type Value = Mailbox;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("mailbox string or object")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                s.parse().map_err(DeError::custom)
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut addr = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(DeError::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Email => {
                            if addr.is_some() {
                                return Err(DeError::duplicate_field("email"));
                            }
                            addr = Some(map.next_value()?);
                        }
                    }
                }
                let addr = addr.ok_or_else(|| DeError::missing_field("email"))?;
                Ok(Mailbox::new(name, addr))
            }
        }

        deserializer.deserialize_any(MailboxVisitor)
    }
}

impl Serialize for Mailboxes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Mailboxes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MailboxesVisitor;

        impl<'de> Visitor<'de> for MailboxesVisitor {
            type Value = Mailboxes;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("mailboxes string or sequence")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                s.parse().map_err(DeError::custom)
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut mboxes = Mailboxes::new();
                while let Some(mbox) = seq.next_element()? {
                    mboxes.push(mbox);
                }
                Ok(mboxes)
            }
        }

        deserializer.deserialize_any(MailboxesVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn parse_address_string() {
        let m: Address = from_str(r#""kayo@example.com""#).unwrap();
        assert_eq!(m, "kayo@example.com".parse().unwrap());
    }

    #[test]
    fn parse_address_object() {
        let m: Address = from_str(r#"{ "user": "kayo", "domain": "example.com" }"#).unwrap();
        assert_eq!(m, "kayo@example.com".parse().unwrap());
    }

    #[test]
    fn parse_mailbox_string() {
        let m: Mailbox = from_str(r#""Kai <kayo@example.com>""#).unwrap();
        assert_eq!(m, "Kai <kayo@example.com>".parse().unwrap());
    }

    #[test]
    fn parse_mailbox_object_address_stirng() {
        let m: Mailbox = from_str(r#"{ "name": "Kai", "email": "kayo@example.com" }"#).unwrap();
        assert_eq!(m, "Kai <kayo@example.com>".parse().unwrap());
    }

    #[test]
    fn parse_mailbox_object_address_object() {
        let m: Mailbox =
            from_str(r#"{ "name": "Kai", "email": { "user": "kayo", "domain": "example.com" } }"#)
                .unwrap();
        assert_eq!(m, "Kai <kayo@example.com>".parse().unwrap());
    }

    #[test]
    fn parse_mailboxes_string() {
        let m: Mailboxes =
            from_str(r#""yin@dtb.com, Hei <hei@dtb.com>, Kai <kayo@example.com>""#).unwrap();
        assert_eq!(
            m,
            "<yin@dtb.com>, Hei <hei@dtb.com>, Kai <kayo@example.com>"
                .parse()
                .unwrap()
        );
    }

    #[test]
    fn parse_mailboxes_array() {
        let m: Mailboxes =
            from_str(r#"["yin@dtb.com", { "name": "Hei", "email": "hei@dtb.com" }, { "name": "Kai", "email": { "user": "kayo", "domain": "example.com" } }]"#)
                .unwrap();
        assert_eq!(
            m,
            "<yin@dtb.com>, Hei <hei@dtb.com>, Kai <kayo@example.com>"
                .parse()
                .unwrap()
        );
    }
}

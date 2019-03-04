use std::{
    io::Write,
    ops::{Deref, DerefMut},
    str::{from_utf8, FromStr},
    string::ToString,
};

use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    types::VarChar,
};
use rust_base58::FromBase58;

use bitcoin::network::Network;
use h256::H256;

pub enum AddressType {
    P2PKH,
    P2SH,
}

#[derive(FromSqlRow, AsExpression, Debug, Serialize, Clone)]
pub struct Address(String);

impl Address {
    fn address_type(&self) -> AddressType {
        match self.chars().next().unwrap() {
            '1' | 'm' | 'n' => AddressType::P2PKH,
            '3' | '2' => AddressType::P2SH,
            _ => panic!("invalid bitcion address found"),
        }
    }

    fn network(&self) -> Network {
        match self.chars().next().unwrap() {
            'm' | 'n' => Network::Mainnet,
            '2' => Network::Test,
            _ => panic!("invalid bitcoin address found"),
        }
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Address, Self::Err> {
        let raw = s.from_base58().map_err(|e| format!("{:?}", e))?;

        if raw.len() != 25 {
            return Err(String::from("invalid bitcoin address length"));
        }

        if raw[21..25] != H256::from_data(&raw[0..21])[0..4] {
            return Err(String::from("invalid bitcoin address checksum"));
        }

        // Only support P2PKH for now.
        match s.chars().next().unwrap() {
            '1' | 'm' | 'n' => (),
            _ => return Err(String::from("address type not supported")),
        };

        Ok(Address(s.to_owned()))
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

impl<'de> serde::Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt::{self, Formatter};

        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Address;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("base58 bitcoin address")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Address::from_str(v).map_err(E::custom)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl ToSql<VarChar, Pg> for Address {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        ToSql::<VarChar, Pg>::to_sql(&self.0, out)
    }
}

impl FromSql<VarChar, Pg> for Address {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let bytes = not_none!(bytes);
        match from_utf8(bytes) {
            Ok(s) => Address::from_str(&s).map_err(|e| e.into()),
            Err(e) => Err(e.into()),
        }
    }
}

impl Deref for Address {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Address {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

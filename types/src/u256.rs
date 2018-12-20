use core::cmp::Ordering;
use std::{
    fmt,
    fmt::LowerHex,
    io::Write,
    ops::{Add, Deref, Div, Mul, Sub},
    str::FromStr,
};

use bigdecimal::BigDecimal;
use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Numeric,
};
use rlp::{Encodable, RlpStream};
use uint::{rustc_hex::FromHexError, FromDecStrErr};

construct_uint!(_U256, 4);

impl serde::Serialize for _U256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:x}", self))
    }
}

impl<'de> serde::Deserialize<'de> for _U256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt::{self, Formatter};

        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = _U256;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a U256 hex string or integer")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut hex = v;
                if &v[0..2] == "0x" {
                    hex = &hex[2..]
                }

                _U256::from_str(hex).map_err(E::custom)
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

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(_U256::from(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(_U256::from(v))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy)]
#[sql_type = "Numeric"]
pub struct U256(pub _U256);

impl U256 {
    pub fn from_dec_str(v: &str) -> Result<U256, FromDecStrErr> {
        match _U256::from_dec_str(v) {
            Ok(u) => Ok(U256(u)),
            Err(e) => Err(e),
        }
    }

    pub fn hex(&self) -> String {
        format!("0x{:x}", self)
    }
}

impl fmt::Debug for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl LowerHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl ToSql<Numeric, Pg> for U256 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let num = BigDecimal::from_str(&format!("{}", self))?;
        ToSql::<Numeric, Pg>::to_sql(&num, out)
    }
}

impl FromSql<Numeric, Pg> for U256 {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let num: BigDecimal = FromSql::<Numeric, Pg>::from_sql(bytes)?;
        match U256::from_dec_str(&format!("{}", num)) {
            Ok(u) => Ok(u),
            Err(_) => Err(format!("invalid value for U256").into()),
        }
    }
}

impl FromStr for U256 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<U256, Self::Err> {
        let u = _U256::from_str(s)?;
        Ok(U256(u))
    }
}

impl From<_U256> for U256 {
    fn from(item: _U256) -> Self {
        U256(item)
    }
}

impl From<i32> for U256 {
    fn from(value: i32) -> U256 {
        U256(_U256::from(value))
    }
}

impl From<i64> for U256 {
    fn from(value: i64) -> U256 {
        U256(_U256::from(value))
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> U256 {
        U256(_U256::from(value))
    }
}

impl<'a> From<&'a [u8]> for U256 {
    fn from(value: &[u8]) -> U256 {
        U256(_U256::from(value))
    }
}

impl Deref for U256 {
    type Target = _U256;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Encodable for U256 {
    fn rlp_append(&self, s: &mut RlpStream) {
        let leading_empty_bytes = 32 - (self.0.bits() + 7) / 8;
        let mut buffer = [0u8; 32];
        self.0.to_big_endian(&mut buffer);
        s.encoder().encode_value(&buffer[leading_empty_bytes..]);
    }
}

impl Add for U256 {
    type Output = U256;

    fn add(self, other: U256) -> U256 {
        U256(self.0 + other.0)
    }
}

impl Sub for U256 {
    type Output = U256;

    fn sub(self, other: U256) -> U256 {
        U256(self.0 - other.0)
    }
}

impl Mul for U256 {
    type Output = U256;

    fn mul(self, other: U256) -> Self {
        U256(self.0 * other.0)
    }
}

impl Div for U256 {
    type Output = U256;

    fn div(self, other: U256) -> Self {
        U256(self.0 / other.0)
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &U256) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }

    fn lt(&self, other: &U256) -> bool {
        self.0.lt(&other.0)
    }
    fn le(&self, other: &U256) -> bool {
        self.0.le(&other.0)
    }

    fn gt(&self, other: &U256) -> bool {
        self.0.gt(&other.0)
    }

    fn ge(&self, other: &U256) -> bool {
        self.0.ge(&other.0)
    }
}

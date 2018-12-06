use core::fmt::LowerHex;
use std::cmp::Ordering;
use std::fmt;
use std::io::Write;
use std::ops::{Add, Deref, Div, Mul, Sub};
use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Numeric;
use rlp::{Encodable, RlpStream};
use uint::rustc_hex::FromHexError;
use uint::FromDecStrErr;

construct_uint!(_U128, 2);

impl serde::Serialize for _U128 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:x}", self))
    }
}

impl<'de> serde::Deserialize<'de> for _U128 {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt::{self, Formatter};

        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = _U128;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a U128 hex string or integer")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut hex = v;
                if &v[0..2] == "0x" {
                    hex = &hex[2..]
                }

                _U128::from_str(hex).map_err(E::custom)
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
                Ok(_U128::from(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(_U128::from(v))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy)]
#[sql_type = "Numeric"]
pub struct U128(pub _U128);

impl U128 {
    pub fn from_dec_str(v: &str) -> Result<U128, FromDecStrErr> {
        match _U128::from_dec_str(v) {
            Ok(u) => Ok(U128(u)),
            Err(e) => Err(e),
        }
    }

    pub fn hex(&self) -> String {
        format!("0x{:x}", self)
    }

    pub fn to_little_endian(&self, bytes: &mut [u8]) {
        self.0.to_little_endian(bytes)
    }

    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }
}

impl fmt::Debug for U128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for U128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl LowerHex for U128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl ToSql<Numeric, Pg> for U128 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let num = BigDecimal::from_str(&format!("{}", self))?;
        ToSql::<Numeric, Pg>::to_sql(&num, out)
    }
}

impl FromSql<Numeric, Pg> for U128 {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let num: BigDecimal = FromSql::<Numeric, Pg>::from_sql(bytes)?;
        match U128::from_dec_str(&format!("{}", num)) {
            Ok(u) => Ok(u),
            Err(_) => Err(format!("invalid value for U128").into()),
        }
    }
}

impl FromStr for U128 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<U128, Self::Err> {
        let u = _U128::from_str(s)?;
        Ok(U128(u))
    }
}

impl From<_U128> for U128 {
    fn from(item: _U128) -> Self {
        U128(item)
    }
}

impl From<i32> for U128 {
    fn from(value: i32) -> U128 {
        U128(_U128::from(value))
    }
}

impl From<i64> for U128 {
    fn from(value: i64) -> U128 {
        U128(_U128::from(value))
    }
}

impl From<u64> for U128 {
    fn from(value: u64) -> U128 {
        U128(_U128::from(value))
    }
}

impl Deref for U128 {
    type Target = _U128;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Encodable for U128 {
    fn rlp_append(&self, s: &mut RlpStream) {
        let leading_empty_bytes = 16 - (self.0.bits() + 7) / 8;
        let mut buffer = [0u8; 16];
        self.0.to_big_endian(&mut buffer);
        s.encoder().encode_value(&buffer[leading_empty_bytes..]);
    }
}

impl Add for U128 {
    type Output = U128;

    fn add(self, other: U128) -> U128 {
        U128(self.0 + other.0)
    }
}

impl Sub for U128 {
    type Output = U128;

    fn sub(self, other: U128) -> U128 {
        U128(self.0 - other.0)
    }
}

impl Mul for U128 {
    type Output = U128;

    fn mul(self, other: U128) -> Self {
        U128(self.0 * other.0)
    }
}

impl Div for U128 {
    type Output = U128;

    fn div(self, other: U128) -> Self {
        U128(self.0 / other.0)
    }
}

impl PartialOrd for U128 {
    fn partial_cmp(&self, other: &U128) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }

    fn lt(&self, other: &U128) -> bool {
        self.0.lt(&other.0)
    }
    fn le(&self, other: &U128) -> bool {
        self.0.le(&other.0)
    }

    fn gt(&self, other: &U128) -> bool {
        self.0.gt(&other.0)
    }

    fn ge(&self, other: &U128) -> bool {
        self.0.ge(&other.0)
    }
}

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
use ethereum_types::U128 as _U128;
use serde::{
    de::{self, Deserializer}, Deserialize,
};

#[derive(FromSqlRow, AsExpression, Serialize, Hash, Eq, PartialEq, Clone)]
#[sql_type = "Numeric"]
pub struct U128(pub _U128);

impl U128 {
    pub fn from_dec_str(value: &str) -> Result<U128, String> {
        let u =
            _U128::from_dec_str(value).map_err(|_| String::from("Failed to convert str to U128"))?;
        Ok(U128(u))
    }

    pub fn from_hex_str(value: &str) -> Result<U128, String> {
        let i = i64::from_str_radix(&value[2..], 16)
            .map_err(|_| String::from("Failed to convert hex str to U128"))?;
        U128::from_dec_str(&format!("{}", i))
    }
}

impl fmt::Debug for U128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
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
        let _u128 = _U128::from_dec_str(&format!("{}", num))
            .map_err(|_| String::from("Failed to construct u128 from bigdecimal string"))?;
        Ok(U128(_u128))
    }
}

impl fmt::Display for U128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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

impl<'de> Deserialize<'de> for U128 {
    fn deserialize<D>(deserializer: D) -> Result<U128, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();

        if s.len() > 2 && &s[..2] == "0x" {
            return U128::from_hex_str(&s)
                .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&s), &r#""U128""#));
        }

        U128::from_dec_str(&format!("{}", s))
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&s), &r#""U128""#))
    }
}

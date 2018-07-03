use std::fmt;
use std::io::Write;
use std::ops::Deref;

use bigdecimal::BigDecimal;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Numeric;
use num_traits::cast::ToPrimitive;
use serde::ser::{Serialize, Serializer};
use web3::types::U256 as _U256;

#[derive(FromSqlRow, AsExpression, Deserialize, Hash, Eq, PartialEq, Clone)]
#[sql_type = "Numeric"]
pub struct U256(pub _U256);

impl fmt::Debug for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let u = self.low_u64();
        write!(f, "{}", u.to_string())
    }
}

impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let u = self.low_u64();
        write!(f, "{}", u.to_string())
    }
}

impl Serialize for U256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.low_u64())
    }
}

impl ToSql<Numeric, Pg> for U256 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let num = BigDecimal::from(self.low_u64());
        ToSql::<Numeric, Pg>::to_sql(&num, out)
    }
}

impl FromSql<Numeric, Pg> for U256 {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let num: BigDecimal = FromSql::<Numeric, Pg>::from_sql(bytes)
            .map_err(|_| String::from("Failed to convert to bigdecimal."))?;
        let num = num.to_u64().ok_or("Failed to parse big number.")?;
        let u256 = _U256::from(num);
        Ok(U256(u256))
    }
}

impl Deref for U256 {
    type Target = _U256;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<_U256> for U256 {
    fn from(u: _U256) -> U256 {
        U256(u)
    }
}

impl From<u32> for U256 {
    fn from(n: u32) -> U256 {
        U256(_U256::from(n))
    }
}

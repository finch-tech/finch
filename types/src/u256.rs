use std::fmt;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Numeric;
use ethereum_types::U256 as _U256;

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
#[sql_type = "Numeric"]
pub struct U256(pub _U256);

impl U256 {
    pub fn from_dec_str(value: &str) -> Result<U256, String> {
        let u =
            _U256::from_dec_str(value).map_err(|_| String::from("Failed to convert str to U256"))?;
        Ok(U256(u))
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

impl ToSql<Numeric, Pg> for U256 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let num = BigDecimal::from_str(&format!("{}", self))?;
        ToSql::<Numeric, Pg>::to_sql(&num, out)
    }
}

impl FromSql<Numeric, Pg> for U256 {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let num: BigDecimal = FromSql::<Numeric, Pg>::from_sql(bytes)?;
        let u256 = _U256::from_dec_str(&format!("{}", num))
            .map_err(|_| String::from("Failed to construct u256 from bigdecimal string"))?;
        Ok(U256(u256))
    }
}

impl From<u32> for U256 {
    fn from(value: u32) -> U256 {
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

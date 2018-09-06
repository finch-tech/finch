use std::fmt;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Numeric;
use ethereum_types::U128 as _U128;

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
#[sql_type = "Numeric"]
pub struct U128(pub _U128);

impl U128 {
    pub fn from_dec_str(value: &str) -> Result<U128, String> {
        let u =
            _U128::from_dec_str(value).map_err(|_| String::from("Failed to convert str to U128"))?;
        Ok(U128(u))
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

use std::convert::Into;
use std::fmt;
use std::io::Write;
use std::ops::Deref;
use std::str::{from_utf8, FromStr};

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::types::VarChar;
use digest::Digest;
use ethereum_types::H160 as _H160;
use ripemd160::Ripemd160;
use rustc_hex::FromHexError;
use sha2::Sha256;

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
#[sql_type = "VarChar"]
pub struct H160(pub _H160);

impl H160 {
    pub fn from_data(data: &[u8]) -> Self {
        let mut output = [0; 20];

        let mut hasher = Sha256::new();
        hasher.input(data);
        let sha2 = hasher.result();

        let mut ripemd = Ripemd160::new();
        ripemd.input(&sha2);
        let ripemd_res = ripemd.result();

        output.copy_from_slice(ripemd_res.as_slice());

        H160(_H160(output))
    }
}

impl fmt::Debug for H160 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for H160 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToSql<VarChar, Pg> for H160 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        ToSql::<VarChar, Pg>::to_sql(&format!("{:?}", self), out)
    }
}

impl FromSql<VarChar, Pg> for H160 {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let bytes = not_none!(bytes);
        match from_utf8(bytes) {
            Ok(h160) => match _H160::from_str(&h160[2..]) {
                Ok(h160) => Ok(H160(h160)),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }
    }
}

impl FromStr for H160 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match _H160::from_str(&s[2..]) {
            Ok(hash) => Ok(H160(hash)),
            Err(e) => Err(e),
        }
    }
}

impl Deref for H160 {
    type Target = _H160;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<_H160> for H160 {
    fn into(self) -> _H160 {
        self.0
    }
}

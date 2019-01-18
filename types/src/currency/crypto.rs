use std::{fmt, io::Write, str::FromStr};

use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    types::VarChar,
};

#[derive(
    FromSqlRow, AsExpression, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash,
)]
#[serde(rename_all = "snake_case")]
#[sql_type = "VarChar"]
pub enum Crypto {
    Btc,
    Eth,
}

impl Crypto {
    pub fn to_str(&self) -> &str {
        match *self {
            Crypto::Btc => "btc",
            Crypto::Eth => "eth",
        }
    }
}

impl fmt::Display for Crypto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl ToSql<VarChar, Pg> for Crypto {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = self.to_str();

        ToSql::<VarChar, Pg>::to_sql(&text, out)
    }
}

impl FromSql<VarChar, Pg> for Crypto {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let s: String = FromSql::<VarChar, Pg>::from_sql(bytes)?;

        Crypto::from_str(&s).map_err(|e| e.into())
    }
}

impl FromStr for Crypto {
    type Err = String;

    fn from_str(s: &str) -> Result<Crypto, Self::Err> {
        match s.as_ref() {
            "btc" => Ok(Crypto::Btc),
            "eth" => Ok(Crypto::Eth),
            _ => Err(String::from("invalid value for crypto")),
        }
    }
}

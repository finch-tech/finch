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
pub enum Fiat {
    Usd,
}

impl Fiat {
    pub fn to_str(&self) -> &str {
        match *self {
            Fiat::Usd => "usd",
        }
    }
}

impl fmt::Display for Fiat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl ToSql<VarChar, Pg> for Fiat {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = self.to_str();

        ToSql::<VarChar, Pg>::to_sql(&text, out)
    }
}

impl FromSql<VarChar, Pg> for Fiat {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let s: String = FromSql::<VarChar, Pg>::from_sql(bytes)?;

        Fiat::from_str(&s).map_err(|e| e.into())
    }
}

impl FromStr for Fiat {
    type Err = String;

    fn from_str(s: &str) -> Result<Fiat, Self::Err> {
        match s.as_ref() {
            "usd" => Ok(Fiat::Usd),
            _ => Err(String::from("invalid value for fiat")),
        }
    }
}

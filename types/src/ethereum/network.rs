use std::{fmt, io::Write, str::FromStr};

use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    types::VarChar,
};

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
#[sql_type = "VarChar"]
pub enum Network {
    Main,
    Ropsten,
}

impl Network {
    pub fn chain_id(&self) -> u64 {
        match self {
            Network::Main => 1,
            Network::Ropsten => 3,
        }
    }

    pub fn to_str(&self) -> &str {
        match *self {
            Network::Main => "main",
            Network::Ropsten => "ropsten",
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl ToSql<VarChar, Pg> for Network {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = self.to_str();

        ToSql::<VarChar, Pg>::to_sql(&text, out)
    }
}

impl FromSql<VarChar, Pg> for Network {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let s: String = FromSql::<VarChar, Pg>::from_sql(bytes)?;

        Network::from_str(&s).map_err(|e| e.into())
    }
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Network, Self::Err> {
        match s.as_ref() {
            "main" => Ok(Network::Main),
            "ropsten" => Ok(Network::Ropsten),
            _ => Err(String::from("Invalid value for ethereum network.")),
        }
    }
}

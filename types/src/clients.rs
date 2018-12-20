use std::fmt;
use std::io::Write;

use diesel::{pg::Pg,serialize::{self, Output, ToSql},types::VarChar,deserialize::{self, FromSql}};

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(rename_all = "snake_case")]
#[sql_type = "VarChar"]
pub enum Client {
    Web,
}

impl Client {
    pub fn to_str(&self) -> &str {
        match *self {
            Client::Web => "web",
        }
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl ToSql<VarChar, Pg> for Client {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = self.to_str();

        ToSql::<VarChar, Pg>::to_sql(&text, out)
    }
}

impl FromSql<VarChar, Pg> for Client {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let text: String = FromSql::<VarChar, Pg>::from_sql(bytes)?;

        match text.as_ref() {
            "web" => Ok(Client::Web),
            v => Err(format!("Unknown value {} for client found", v).into()),
        }
    }
}

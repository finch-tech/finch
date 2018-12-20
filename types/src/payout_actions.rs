use std::{fmt, io::Write};

use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Text,
    types::VarChar,
};

#[derive(
    FromSqlRow, AsExpression, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash,
)]
#[serde(rename_all = "snake_case")]
#[sql_type = "VarChar"]
pub enum PayoutAction {
    Payout,
    Refund,
}

impl PayoutAction {
    pub fn to_str(&self) -> &str {
        match *self {
            PayoutAction::Payout => "payout",
            PayoutAction::Refund => "refund",
        }
    }
}

impl fmt::Display for PayoutAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl ToSql<Text, Pg> for PayoutAction {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = self.to_str();

        ToSql::<Text, Pg>::to_sql(&text, out)
    }
}

impl FromSql<Text, Pg> for PayoutAction {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let text: String = FromSql::<Text, Pg>::from_sql(bytes)?;

        match text.as_ref() {
            "payout" => Ok(PayoutAction::Payout),
            "refund" => Ok(PayoutAction::Refund),
            v => Err(format!("Unknown value {} for PayoutAction found", v).into()),
        }
    }
}

use std::{fmt, io::Write};

use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Text,
    types::VarChar,
};

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[sql_type = "VarChar"]
pub enum PayoutStatus {
    Pending,
    PaidOut,
    Refunded,
}

impl fmt::Display for PayoutStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PayoutStatus::Pending => "pending",
                PayoutStatus::PaidOut => "paid_out",
                PayoutStatus::Refunded => "refunded",
            }
        )
    }
}

impl ToSql<Text, Pg> for PayoutStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = match *self {
            PayoutStatus::Pending => "pending",
            PayoutStatus::PaidOut => "paid_out",
            PayoutStatus::Refunded => "refunded",
        };

        ToSql::<Text, Pg>::to_sql(&text, out)
    }
}

impl FromSql<Text, Pg> for PayoutStatus {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let text: String = FromSql::<Text, Pg>::from_sql(bytes)
            .map_err(|_| String::from("failed to convert to text"))?;

        match text.as_ref() {
            "pending" => Ok(PayoutStatus::Pending),
            "paid_out" => Ok(PayoutStatus::PaidOut),
            "refunded" => Ok(PayoutStatus::Refunded),
            v => Err(format!("unknown value {} for Currency found", v).into()),
        }
    }
}

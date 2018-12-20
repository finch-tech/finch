use std::{fmt, io::Write};

use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Text,
    types::VarChar,
};

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[sql_type = "VarChar"]
pub enum PaymentStatus {
    Pending,
    Paid,
    Confirmed,
    Completed,
    InsufficientAmount,
    Expired,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PaymentStatus::Pending => "pending",
                PaymentStatus::Paid => "paid",
                PaymentStatus::Confirmed => "confirmed",
                PaymentStatus::Completed => "completed",
                PaymentStatus::InsufficientAmount => "insufficient_amount",
                PaymentStatus::Expired => "expired",
            }
        )
    }
}

impl ToSql<Text, Pg> for PaymentStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = format!("{}", self);

        ToSql::<Text, Pg>::to_sql(&text, out)
    }
}

impl FromSql<Text, Pg> for PaymentStatus {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let text: String = FromSql::<Text, Pg>::from_sql(bytes)
            .map_err(|_| String::from("Failed to convert to text."))?;

        match text.as_ref() {
            "pending" => Ok(PaymentStatus::Pending),
            "paid" => Ok(PaymentStatus::Paid),
            "confirmed" => Ok(PaymentStatus::Confirmed),
            "completed" => Ok(PaymentStatus::Completed),
            "insufficient_amount" => Ok(PaymentStatus::InsufficientAmount),
            "expired" => Ok(PaymentStatus::Expired),
            v => Err(format!("Unknown value {} for PaymentStatus found", v).into()),
        }
    }
}

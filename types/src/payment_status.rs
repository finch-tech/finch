use std::fmt;
use std::io::Write;

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::types::VarChar;

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[sql_type = "VarChar"]
pub enum PaymentStatus {
    Pending,
    Paid,
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
                PaymentStatus::InsufficientAmount => "insufficient_amount",
                PaymentStatus::Expired => "expired",
            }
        )
    }
}

impl ToSql<Text, Pg> for PaymentStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = match *self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Paid => "paid",
            PaymentStatus::InsufficientAmount => "insufficient_amount",
            PaymentStatus::Expired => "expired",
        };

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
            "insufficient_amount" => Ok(PaymentStatus::InsufficientAmount),
            "expired" => Ok(PaymentStatus::Expired),
            v => Err(format!("Unknown value {} for Currency found", v).into()),
        }
    }
}

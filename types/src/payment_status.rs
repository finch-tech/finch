use std::error::Error;
use std::fmt;
use std::io::Write;

use diesel::deserialize::{self, FromSql, Queryable};
use diesel::dsl::AsExprOf;
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::row::Row;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::types::FromSqlRow;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Paid,
    // InsufficientAmount,
    PaidOut,
    Refunded,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PaymentStatus::Pending => "pending",
                PaymentStatus::Paid => "paid",
                PaymentStatus::PaidOut => "paid_out",
                PaymentStatus::Refunded => "refunded",
            }
        )
    }
}

impl FromSqlRow<Text, Pg> for PaymentStatus {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
        match String::build_from_row(row)?.as_ref() {
            "pending" => Ok(PaymentStatus::Pending),
            "paid" => Ok(PaymentStatus::Paid),
            "paid_out" => Ok(PaymentStatus::PaidOut),
            "refunded" => Ok(PaymentStatus::Refunded),
            v => Err(format!("Unknown value {} for PaymentStatus found", v).into()),
        }
    }
}

impl ToSql<Text, Pg> for PaymentStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = match *self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Paid => "paid",
            PaymentStatus::PaidOut => "paid_out",
            PaymentStatus::Refunded => "refunded",
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
            "paid_out" => Ok(PaymentStatus::PaidOut),
            "refunded" => Ok(PaymentStatus::Refunded),
            v => Err(format!("Unknown value {} for Currency found", v).into()),
        }
    }
}

impl Queryable<Text, Pg> for PaymentStatus {
    type Row = Self;

    fn build(row: Self::Row) -> Self {
        row
    }
}

impl AsExpression<Text> for PaymentStatus {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

impl<'a> AsExpression<Text> for &'a PaymentStatus {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

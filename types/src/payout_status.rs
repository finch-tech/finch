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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
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

impl FromSqlRow<Text, Pg> for PayoutStatus {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
        match String::build_from_row(row)?.as_ref() {
            "pending" => Ok(PayoutStatus::Pending),
            "paid_out" => Ok(PayoutStatus::PaidOut),
            "refunded" => Ok(PayoutStatus::Refunded),
            v => Err(format!("Unknown value {} for PayoutStatus found", v).into()),
        }
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
            .map_err(|_| String::from("Failed to convert to text."))?;

        match text.as_ref() {
            "pending" => Ok(PayoutStatus::Pending),
            "paid_out" => Ok(PayoutStatus::PaidOut),
            "refunded" => Ok(PayoutStatus::Refunded),
            v => Err(format!("Unknown value {} for Currency found", v).into()),
        }
    }
}

impl Queryable<Text, Pg> for PayoutStatus {
    type Row = Self;

    fn build(row: Self::Row) -> Self {
        row
    }
}

impl AsExpression<Text> for PayoutStatus {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

impl<'a> AsExpression<Text> for &'a PayoutStatus {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

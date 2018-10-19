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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Currency {
    Btc,
    Eth,
    Usd,
}

impl Currency {
    pub fn to_str(&self) -> &str {
        match *self {
            Currency::Btc => "btc",
            Currency::Eth => "eth",
            Currency::Usd => "usd",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl FromSqlRow<Text, Pg> for Currency {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
        match String::build_from_row(row)?.as_ref() {
            "btc" => Ok(Currency::Btc),
            "eth" => Ok(Currency::Eth),
            "usd" => Ok(Currency::Usd),
            v => Err(format!("Unknown value {} for Currency found", v).into()),
        }
    }
}

impl ToSql<Text, Pg> for Currency {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = self.to_str();

        ToSql::<Text, Pg>::to_sql(&text, out)
    }
}

impl FromSql<Text, Pg> for Currency {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let text: String = FromSql::<Text, Pg>::from_sql(bytes)?;

        match text.as_ref() {
            "btc" => Ok(Currency::Btc),
            "eth" => Ok(Currency::Eth),
            "usd" => Ok(Currency::Usd),
            v => Err(format!("Unknown value {} for Currency found", v).into()),
        }
    }
}

impl Queryable<Text, Pg> for Currency {
    type Row = Self;

    fn build(row: Self::Row) -> Self {
        row
    }
}

impl AsExpression<Text> for Currency {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

impl<'a> AsExpression<Text> for &'a Currency {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

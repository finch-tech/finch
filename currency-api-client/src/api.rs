use std::error::Error;
use std::fmt;
use std::io::Write;
use std::str::FromStr;

use actix_web::{client, HttpMessage};
use bigdecimal::BigDecimal;
use diesel::deserialize::{self, FromSql, Queryable};
use diesel::dsl::AsExprOf;
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::row::Row;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::types::FromSqlRow;
use futures::future::{err, ok, Future};
use serde_json::{self, Value};
use url::Url;

use errors::Error as ApiClientError;
use types::Currency;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Api {
    CoinApi,
}

impl Api {
    pub fn to_str(&self) -> &str {
        match *self {
            Api::CoinApi => "coin_api",
        }
    }

    pub fn auth_header(&self) -> &str {
        match self {
            Api::CoinApi => "X-CoinAPI-Key",
        }
    }

    pub fn get_rate(
        &self,
        from: &Currency,
        to: &Currency,
        key: &str,
    ) -> Box<Future<Item = BigDecimal, Error = ApiClientError>> {
        // Currently only supports CoinAPI.
        // Add support for other external APIs later.
        let mut url = self.base_url();

        url.set_path(&format!(
            "/v1/exchangerate/{}/{}",
            format!("{}", from.to_string().to_uppercase()),
            format!("{}", to.to_string().to_uppercase())
        ));

        let req = match client::ClientRequest::get(url.as_str())
            .header(self.auth_header(), key)
            .finish()
        {
            Ok(req) => req,
            Err(_) => return Box::new(err(ApiClientError::ResponseError)),
        };

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => {
                        return err(ApiClientError::from(e));
                    }
                };

                match body.get("rate") {
                    Some(rate) => ok(BigDecimal::from_str(&format!(
                        "{}",
                        rate.as_f64().to_owned().unwrap()
                    ))
                    .unwrap()),
                    None => err(ApiClientError::ResponseError),
                }
            })
        }))
    }

    pub fn base_url(&self) -> Url {
        match self {
            Api::CoinApi => Url::from_str("https://rest.coinapi.io").unwrap(),
        }
    }
}

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl FromSqlRow<Text, Pg> for Api {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
        match String::build_from_row(row)?.as_ref() {
            "coin_api" => Ok(Api::CoinApi),
            v => Err(format!("Unknown value {} for currency api found", v).into()),
        }
    }
}

impl ToSql<Text, Pg> for Api {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let text = self.to_str();

        ToSql::<Text, Pg>::to_sql(&text, out)
    }
}

impl FromSql<Text, Pg> for Api {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let text: String = FromSql::<Text, Pg>::from_sql(bytes)?;

        match text.as_ref() {
            "coin_api" => Ok(Api::CoinApi),
            v => Err(format!("Unknown value {} for currency api found", v).into()),
        }
    }
}

impl Queryable<Text, Pg> for Api {
    type Row = Self;

    fn build(row: Self::Row) -> Self {
        row
    }
}

impl AsExpression<Text> for Api {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

impl<'a> AsExpression<Text> for &'a Api {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "coin_api" => Ok(Api::CoinApi),
            v => Err(format!("Unknown value {} for currency api found", v).into()),
        }
    }
}

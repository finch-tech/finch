use std::fmt;
use std::io::Write;
use std::str::FromStr;

use actix_web::{client, HttpMessage};
use bigdecimal::BigDecimal;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use futures::future::{err, ok, Future};
use serde_json::{self, Value};
use url::Url;

use errors::Error as ApiClientError;
use types::currency::{Crypto, Fiat};

#[derive(FromSqlRow, AsExpression, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Api {
    CoinApi,
    CryptoCompare,
}

impl Api {
    fn auth_header(&self) -> &str {
        match self {
            Api::CoinApi => "X-CoinAPI-Key",
            Api::CryptoCompare => "Authorization",
        }
    }

    fn base_url(&self) -> Url {
        match self {
            Api::CoinApi => Url::from_str("https://rest.coinapi.io").unwrap(),
            Api::CryptoCompare => Url::from_str("https://min-api.cryptocompare.com").unwrap(),
        }
    }

    fn rate_endpoint(&self, from: Fiat, to: Crypto) -> Url {
        let mut url = self.base_url();

        match self {
            Api::CoinApi => {
                url.set_path(&format!(
                    "/v1/exchangerate/{}/{}",
                    format!("{}", from.to_string().to_uppercase()),
                    format!("{}", to.to_string().to_uppercase())
                ));
            }
            Api::CryptoCompare => {
                url.set_path(&format!("/data/price"));
                url.query_pairs_mut()
                    .append_pair("fsym", &from.to_string().to_uppercase());
                url.query_pairs_mut()
                    .append_pair("tsyms", &to.to_string().to_uppercase());
            }
        };
        url
    }

    fn rate_field(&self, _: Fiat, to: Crypto) -> String {
        match self {
            Api::CoinApi => "rate".to_owned(),
            Api::CryptoCompare => to.to_string().to_uppercase(),
        }
    }

    pub fn to_str(&self) -> &str {
        match *self {
            Api::CoinApi => "coinapi",
            Api::CryptoCompare => "cryptocompare",
        }
    }

    pub fn get_rate(
        &self,
        from: Fiat,
        to: Crypto,
        key: &str,
    ) -> Box<Future<Item = BigDecimal, Error = ApiClientError>> {
        let url = self.rate_endpoint(from, to);

        let req = match client::ClientRequest::get(url.as_str())
            .header(self.auth_header(), key)
            .finish()
        {
            Ok(req) => req,
            Err(_) => return Box::new(err(ApiClientError::ResponseError)),
        };

        let rate_field = self.rate_field(from, to);

        Box::new(req.send().from_err().and_then(move |resp| {
            resp.body().from_err().and_then(move |body| {
                let body: Value = match serde_json::from_slice(&body) {
                    Ok(body) => body,
                    Err(e) => {
                        return err(ApiClientError::from(e));
                    }
                };

                match body.get(&rate_field) {
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
}

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
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
            "coinapi" => Ok(Api::CoinApi),
            v => Err(format!("unknown value {} for currency api found", v).into()),
        }
    }
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "coinapi" => Ok(Api::CoinApi),
            v => Err(format!("unknown value {} for currency api found", v).into()),
        }
    }
}

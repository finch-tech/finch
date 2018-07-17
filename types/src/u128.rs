use std::fmt;
use std::ops::Deref;

use ethereum_types::U128 as _U128;

#[derive(FromSqlRow, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct U128(pub _U128);

impl U128 {
    pub fn from_dec_str(value: &str) -> Result<U128, String> {
        let u =
            _U128::from_dec_str(value).map_err(|_| String::from("Failed to convert str to U128"))?;
        Ok(U128(u))
    }
}

impl fmt::Debug for U128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for U128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for U128 {
    fn from(value: u64) -> U128 {
        U128(_U128::from(value))
    }
}

impl Deref for U128 {
    type Target = _U128;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

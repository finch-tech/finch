use core::cmp;
use libc;
use std::fmt;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::str::{from_utf8, FromStr};

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::types::VarChar;
use digest::Digest;
use ripemd160::Ripemd160;
use rlp::{Encodable, RlpStream};
use rustc_hex::{FromHex, FromHexError};
use sha2::Sha256;

#[derive(FromSqlRow, AsExpression, Copy, Eq)]
#[sql_type = "VarChar"]
pub struct H160(pub [u8; 20]);

impl H160 {
    pub fn new() -> Self {
        H160([0; 20])
    }

    #[inline]
    pub fn clone_from_slice(&mut self, src: &[u8]) -> usize {
        let min = cmp::min(20, src.len());
        self.0[..min].copy_from_slice(&src[..min]);
        min
    }

    pub fn from_slice(src: &[u8]) -> Self {
        let mut r = Self::new();
        r.clone_from_slice(src);
        r
    }

    pub fn from_data(data: &[u8]) -> Self {
        let mut output = [0; 20];

        let mut hasher = Sha256::new();
        hasher.input(data);
        let sha2 = hasher.result();

        let mut ripemd = Ripemd160::new();
        ripemd.input(&sha2);
        let ripemd_res = ripemd.result();

        output.copy_from_slice(ripemd_res.as_slice());

        H160(output)
    }

    pub fn from_hash(data: [u8; 20]) -> Self {
        H160(data)
    }

    pub fn from_hex(s: &str) -> Result<H160, HexError> {
        let mut hex = s.as_bytes();

        if hex.len() == 42 && &s[0..2] == "0x" {
            hex = &hex[2..]
        }

        if hex.len() != 40 {
            return Err(HexError::BadLength(s.len()));
        }

        let mut ret = [0; 20];
        for (i, byte) in ret.iter_mut().enumerate() {
            let hi = match hex[2 * i] {
                c @ b'A'...b'F' => (c - b'A' + 10) as u8,
                c @ b'a'...b'f' => (c - b'a' + 10) as u8,
                c @ b'0'...b'9' => (c - b'0') as u8,
                c => return Err(HexError::BadCharacter(c as char)),
            };

            let lo = match hex[2 * i + 1] {
                c @ b'A'...b'F' => (c - b'A' + 10) as u8,
                c @ b'a'...b'f' => (c - b'a' + 10) as u8,
                c @ b'0'...b'9' => (c - b'0') as u8,
                c => return Err(HexError::BadCharacter(c as char)),
            };

            *byte = hi << 4 | lo;
        }

        Ok(H160(ret))
    }

    pub fn hex(&self) -> String {
        format!("0x{}", self)
    }
}

impl serde::Serialize for H160 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("0x{}", self))
    }
}

#[derive(Debug, Fail)]
pub enum HexError {
    #[fail(display = "bad length {} for ripemd160 hex string", _0)]
    BadLength(usize),
    #[fail(display = "bad character {} in ripemd160 hex string", _0)]
    BadCharacter(char),
}

impl<'de> serde::Deserialize<'de> for H160 {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt::{self, Formatter};

        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = H160;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a Ripemd160 hash")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                H160::from_hex(v).map_err(E::custom)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Encodable for H160 {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.encoder().encode_value(self);
    }
}

impl fmt::Debug for H160 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self[..] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl fmt::Display for H160 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self[..] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl From<[u8; 20]> for H160 {
    fn from(bytes: [u8; 20]) -> Self {
        H160(bytes)
    }
}

impl From<H160> for [u8; 20] {
    fn from(s: H160) -> Self {
        s.0
    }
}

impl Deref for H160 {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for H160 {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl AsRef<[u8]> for H160 {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Clone for H160 {
    fn clone(&self) -> H160 {
        let mut ret = H160::new();
        ret.0.copy_from_slice(&self.0);
        ret
    }
}

impl PartialEq for H160 {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            libc::memcmp(
                self.0.as_ptr() as *const libc::c_void,
                other.0.as_ptr() as *const libc::c_void,
                20,
            ) == 0
        }
    }
}

impl Ord for H160 {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        let r = unsafe {
            libc::memcmp(
                self.0.as_ptr() as *const libc::c_void,
                other.0.as_ptr() as *const libc::c_void,
                20,
            )
        };
        if r < 0 {
            return ::core::cmp::Ordering::Less;
        }
        if r > 0 {
            return ::core::cmp::Ordering::Greater;
        }
        return ::core::cmp::Ordering::Equal;
    }
}

impl PartialOrd for H160 {
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ::core::hash::Hash for H160 {
    fn hash<H>(&self, state: &mut H)
    where
        H: ::core::hash::Hasher,
    {
        state.write(&self.0);
        state.finish();
    }
}

impl From<u64> for H160 {
    fn from(mut value: u64) -> H160 {
        let mut ret = H160::new();
        for i in 0..8 {
            if i < 20 {
                ret.0[20 - i - 1] = (value & 0xff) as u8;
                value >>= 8;
            }
        }
        ret
    }
}

impl<'a> From<&'a [u8]> for H160 {
    fn from(s: &'a [u8]) -> H160 {
        H160::from_slice(s)
    }
}

impl FromStr for H160 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<H160, FromHexError> {
        let a = s.from_hex()?;
        if a.len() != 20 {
            return Err(FromHexError::InvalidHexLength);
        }

        let mut ret = [0; 20];
        ret.copy_from_slice(&a);
        Ok(H160(ret))
    }
}

impl ToSql<VarChar, Pg> for H160 {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        ToSql::<VarChar, Pg>::to_sql(&format!("{:?}", self), out)
    }
}

impl FromSql<VarChar, Pg> for H160 {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let bytes = not_none!(bytes);
        match from_utf8(bytes) {
            Ok(s) => H160::from_str(&s).map_err(|e| e.into()),
            Err(e) => Err(e.into()),
        }
    }
}

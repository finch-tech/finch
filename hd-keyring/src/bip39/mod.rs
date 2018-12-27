mod errors;

use std::{collections::HashMap, iter::Iterator};

use bit_vec::BitVec;
use rand::{thread_rng, Rng};
use ring::{digest, pbkdf2};
use sha2::{Digest, Sha256};

pub use self::errors::Error;

static PBKDF2_ROUNDS: u32 = 2048;
static PBKDF2_BYTES: usize = 64;

lazy_static! {
    static ref ENGLISH_WORDLIST: Vec<&'static str> = {
        include_str!("wordlist/english.txt")
            .split_whitespace()
            .collect()
    };
    static ref FRENCH_WORDLIST: Vec<&'static str> = {
        include_str!("wordlist/french.txt")
            .split_whitespace()
            .collect()
    };
    static ref ENGLISH_WORDMAP: HashMap<&'static str, u16> = {
        ENGLISH_WORDLIST
            .iter()
            .enumerate()
            .map(|(i, word)| (*word, i as u16))
            .collect()
    };
    static ref FRENCH_WORDMAP: HashMap<&'static str, u16> = {
        FRENCH_WORDLIST
            .iter()
            .enumerate()
            .map(|(i, word)| (*word, i as u16))
            .collect()
    };
}

pub enum MnemonicType {
    Words12,
    Words24,
}

impl MnemonicType {
    pub fn from_word_count(count: usize) -> Result<Self, Error> {
        let typ = match count {
            12 => MnemonicType::Words12,
            24 => MnemonicType::Words24,
            _ => Err(Error::InvalidWordLength(count))?,
        };

        Ok(typ)
    }

    fn entropy_length(&self) -> usize {
        match *self {
            MnemonicType::Words12 => 128,
            MnemonicType::Words24 => 256,
        }
    }

    fn checksum_length(&self) -> usize {
        match *self {
            MnemonicType::Words12 => 4,
            MnemonicType::Words24 => 8,
        }
    }

    fn word_count(&self) -> usize {
        match *self {
            MnemonicType::Words12 => 12,
            MnemonicType::Words24 => 24,
        }
    }
}

#[derive(Debug)]
pub enum Language {
    English,
    French,
}

impl Language {
    fn get_wordlist(&self) -> &'static Vec<&'static str> {
        match *self {
            Language::English => &*ENGLISH_WORDLIST,
            Language::French => &*FRENCH_WORDLIST,
        }
    }

    fn get_wordmap(&self) -> &'static HashMap<&'static str, u16> {
        match *self {
            Language::English => &*ENGLISH_WORDMAP,
            Language::French => &*FRENCH_WORDMAP,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Seed {
    bytes: Vec<u8>,
}

impl Seed {
    pub fn new(entropy: &[u8], pass: String) -> Self {
        let salt = format!("mnemonic{}", pass);

        let mut bytes = vec![0u8; PBKDF2_BYTES];

        static DIGEST_ALG: &'static digest::Algorithm = &digest::SHA512;

        pbkdf2::derive(
            DIGEST_ALG,
            PBKDF2_ROUNDS,
            salt.as_bytes(),
            entropy,
            &mut bytes,
        );

        Seed { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

#[derive(Debug)]
pub struct Mnemonic {
    phrase: String,
    lang: Language,
    entropy: Vec<u8>,
    seed: Seed,
}

impl Mnemonic {
    pub fn new<T>(typ: MnemonicType, lang: Language, pass: T) -> Result<Self, Error>
    where
        T: Into<String>,
    {
        let pass = pass.into();

        let mut rng = thread_rng();
        let entropy_length = typ.entropy_length();

        let mut entropy = vec![0u8; entropy_length / 8];
        rng.fill_bytes(&mut entropy);

        let wordlist = lang.get_wordlist();

        let mut with_checksum = BitVec::from_bytes(
            &entropy
                .iter()
                .chain(Some(&Sha256::digest(&entropy)[0]))
                .cloned()
                .collect::<Vec<u8>>(),
        );

        with_checksum.truncate(11 * typ.word_count());

        let mut phrase = Vec::new();
        let mut index_bits = BitVec::new();

        for bit in with_checksum {
            index_bits.push(bit);

            if index_bits.len() == 11 {
                let index = u32::from_str_radix(&format!("{:?}", index_bits), 2)?;
                let word = wordlist.get(index as usize).unwrap().clone();

                phrase.push(word);

                index_bits = BitVec::default();
            }
        }

        let seed = Seed::new(&phrase.join(" ").as_bytes(), pass);

        Ok(Mnemonic {
            phrase: phrase.join(" "),
            lang,
            entropy,
            seed,
        })
    }

    pub fn from_string<T>(phrase: T, lang: Language, pass: T) -> Result<Self, Error>
    where
        T: Into<String>,
    {
        let pass = pass.into();
        let phrase = phrase.into();

        let typ = MnemonicType::from_word_count(
            phrase
                .split(" ")
                .map(|s| s.to_owned())
                .collect::<Vec<String>>()
                .len(),
        )?;

        let checksum_length = typ.checksum_length();

        let wordmap = lang.get_wordmap();

        let mut indexes = Vec::new();

        for word in phrase.split(" ") {
            match wordmap.get(&word) {
                Some(index) => indexes.push(index),
                None => return Err(Error::InvalidWord(word.to_owned())),
            }
        }

        let mut with_checksum = BitVec::new();
        for index in indexes {
            for x in (0..11).rev() {
                let bit = ((index >> x as u32) & 1) == 1;
                with_checksum.push(bit);
            }
        }

        let mut checksum = BitVec::new();
        for _ in 0..checksum_length {
            checksum.push(with_checksum.pop().unwrap());
        }

        checksum = checksum.iter().rev().collect::<BitVec>();

        with_checksum.truncate(11 * typ.word_count());
        let entropy = with_checksum.to_bytes();

        let mut expected_checksum = BitVec::from_bytes(&vec![Sha256::digest(&entropy)[0]]);
        expected_checksum.truncate(checksum_length);

        if checksum != expected_checksum {
            return Err(Error::InvalidChecksum);
        }

        let seed = Seed::new(&phrase.as_bytes(), pass);

        Ok(Mnemonic {
            phrase: phrase.to_owned(),
            lang,
            entropy,
            seed,
        })
    }

    pub fn phrase(&self) -> String {
        self.phrase.to_owned()
    }

    pub fn seed(&self) -> Seed {
        self.seed.to_owned()
    }
}

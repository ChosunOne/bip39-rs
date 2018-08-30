use std::path::PathBuf;
use std::io::Read;
use std::fs::File;
use std::collections::HashMap;

use serde_json::de;

use bitreader::BitReader;
use bit_vec::BitVec;

use data_encoding::HEXUPPER;

use ::crypto::{gen_random_bytes, sha256};
use ::error::{Error, ErrorKind};
use ::mnemonic_type::MnemonicType;
//use ::language::Language;
use ::util::bit_from_u16_as_u11;
use ::seed::Seed;

/// The primary type in this crate, most tasks require creating or using one.
///
/// To create a *new* [`Mnemonic`][Mnemonic] from a randomly generated key, call [`Mnemonic::new()`][Mnemonic::new()].
///
/// To get a [`Mnemonic`][Mnemonic] instance for an existing mnemonic phrase, including
/// those generated by other software or hardware wallets, use [`Mnemonic::from_string()`][Mnemonic::from_string()].
///
/// You can get the HD wallet [`Seed`][Seed] from a [`Mnemonic`][Mnemonic] by calling [`Mnemonic::get_seed()`][Mnemonic::get_seed()],
/// from there you can either get the raw byte value with [`Seed::as_bytes()`][Seed::as_bytes()], or the hex
/// representation with [`Seed::as_hex()`][Seed::as_hex()].
///
/// You can also get the original entropy value back from a [`Mnemonic`][Mnemonic] with [`Mnemonic::to_entropy()`][Mnemonic::to_entropy()],
/// but beware that the entropy value is **not the same thing** as an HD wallet seed, and should
/// *never* be used that way.
///
///
/// [Mnemonic]: ./mnemonic/struct.Mnemonic.html
/// [Mnemonic::new()]: ./mnemonic/struct.Mnemonic.html#method.new
/// [Mnemonic::from_string()]: ./mnemonic/struct.Mnemonic.html#method.from_string
/// [Mnemonic::get_seed()]: ./mnemonic/struct.Mnemonic.html#method.get_seed
/// [Mnemonic::to_entropy()]: ./mnemonic/struct.Mnemonic.html#method.to_entropy
/// [Seed]: ./seed/struct.Seed.html
/// [Seed::as_bytes()]: ./seed/struct.Seed.html#method.as_bytes
/// [Seed::as_hex()]: ./seed/struct.Seed.html#method.as_hex
///
#[derive(Debug, Clone)]
pub struct Mnemonic {
    string: String,
    seed: Seed,
    word_list: WordList,
    entropy: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
struct WordList {
    pub language: String,
    pub words: Vec<String>
}

impl WordList {
    pub fn gen_wordmap(&self) -> HashMap<String, u16> {

        let mut word_map: HashMap<String, u16> = HashMap::new();
        for (i, item) in self.words.into_iter().enumerate() {
            word_map.insert(item.to_owned(), i as u16);
        }
        word_map
    }
}

impl Mnemonic {

    /// Generates a new `Mnemonic`
    ///
    /// Can be used to get the [`Seed`][Seed] using [`Mnemonic::get_seed()`][Mnemonic::get_seed()].
    ///
    /// Can also be used to get the original entropy value. Use [`Mnemonic::as_entropy()`][Mnemonic::as_entropy()] for a slice, or
    /// [Mnemonic::get_entropy()][Mnemonic::get_entropy()] for an owned `Vec<u8>`.
    ///
    ///
    /// use bip39::{Mnemonic, MnemonicType, Language};
    ///
    /// let mnemonic_type = MnemonicType::for_word_count(12).unwrap();
    ///
    /// let mnemonic = match Mnemonic::new(mnemonic_type, Language::English, "") {
    ///     Ok(b) => b,
    ///     Err(e) => { println!("e: {}", e); return }
    /// };
    ///
    /// let phrase = mnemonic.get_string();
    /// let seed = mnemonic.get_seed();
    /// let seed_bytes: &[u8] = seed.as_bytes();
    /// let seed_hex: &str = seed.as_hex();
    /// println!("phrase: {}", phrase);
    /// ```
    /// [Seed]: ../seed/struct.Seed.html
    /// [Mnemonic::get_seed()]: ./mnemonic/struct.Mnemonic.html#method.get_seed
    /// [Mnemonic::as_entropy()]: ./mnemonic/struct.Mnemonic.html#method.as_entropy
    /// [Mnemonic::get_entropy()]: ./mnemonic/struct.Mnemonic.html#method.get_entropy
    pub fn new<S>(mnemonic_type: MnemonicType,
                  path: PathBuf,
                  password: S) -> Result<Mnemonic, Error> where S: Into<String> {

        let file = File::open(path)?;
        let word_list: WordList;
        match de::from_reader(file) {
            Ok(w) => word_list = w,
            Err(e) => return Err()
        }
        let entropy_bits = mnemonic_type.entropy_bits();

        let entropy = gen_random_bytes(entropy_bits / 8)?;

        Mnemonic::from_entropy(&entropy, mnemonic_type, word_list, password)
    }

    /// Create a [`Mnemonic`][Mnemonic] from generated entropy
    ///
    /// # Example
    ///
    /// ```
    /// use bip39::{Mnemonic, MnemonicType, Language};
    ///
    /// let entropy = &[0x33, 0xE4, 0x6B, 0xB1, 0x3A, 0x74, 0x6E, 0xA4, 0x1C, 0xDD, 0xE4, 0x5C, 0x90, 0x84, 0x6A, 0x79];
    /// let mnemonic = Mnemonic::from_entropy(entropy, MnemonicType::for_key_size(128).unwrap(), Language::English, "").unwrap();
    ///
    /// assert_eq!("crop cash unable insane eight faith inflict route frame loud box vibrant", mnemonic.as_str());
    /// ```
    ///
    /// [Mnemonic]: ../mnemonic/struct.Mnemonic.html
    pub fn from_entropy<S>(entropy: &[u8],
                           mnemonic_type: MnemonicType,
                           word_list: WordList,
                           password: S) -> Result<Mnemonic, Error> where S: Into<String> {
        let entropy_length_bits = entropy.len() * 8;

        if entropy_length_bits != mnemonic_type.entropy_bits() {
            return Err(ErrorKind::InvalidEntropyLength(entropy_length_bits, mnemonic_type).into())
        }

        let num_words = mnemonic_type.word_count();

        let entropy_hash = sha256(entropy);

        // we put both the entropy and the hash of the entropy (in that order) into a single vec
        // and then just read 11 bits at a time out of the entire thing `num_words` times. We
        // can do that because:
        //
        // 12 words * 11bits = 132bits
        // 15 words * 11bits = 165bits
        //
        // ... and so on. It grabs the entropy and then the right number of hash bits and no more.

        let mut combined = Vec::from(entropy);
        combined.extend(&entropy_hash);

        let mut reader = BitReader::new(&combined);

        let mut words: Vec<&str> = Vec::new();
        for _ in 0..num_words {
            let n = reader.read_u16(11);
            words.push(word_list[n.unwrap() as usize].as_ref());
        }

        let string = words.join(" ");

        Mnemonic::from_string(string, word_list, password.into())
    }

    /// Create a [`Mnemonic`][Mnemonic] from generated entropy hexadecimal representation
    ///
    /// # Example
    ///
    /// ```
    /// use bip39::{Mnemonic, MnemonicType, Language};
    ///
    /// let entropy = "33E46BB13A746EA41CDDE45C90846A79";
    /// let mnemonic = Mnemonic::from_entropy_hex(entropy, MnemonicType::for_key_size(128).unwrap(), Language::English, "").unwrap();
    ///
    /// assert_eq!("crop cash unable insane eight faith inflict route frame loud box vibrant", mnemonic.as_str());
    /// ```
    ///
    /// [Mnemonic]: ../mnemonic/struct.Mnemonic.html
    pub fn from_entropy_hex<S>(entropy: &str,
                           mnemonic_type: MnemonicType,
                           word_list: WordList,
                           password: S) -> Result<Mnemonic, Error> where S: Into<String> {

        Mnemonic::from_entropy(&HEXUPPER.decode(entropy.as_ref())?, mnemonic_type, word_list, password)
    }

    /// Create a [`Mnemonic`][Mnemonic] from an existing mnemonic phrase
    ///
    /// The phrase supplied will be checked for word length and validated according to the checksum
    /// specified in BIP0039
    ///
    /// # Example
    ///
    /// ```
    /// use bip39::{Mnemonic, Language};
    ///
    /// let test_mnemonic = "park remain person kitchen mule spell knee armed position rail grid ankle";
    ///
    /// let mnemonic = Mnemonic::from_string(test_mnemonic, Language::English, "").unwrap();
    /// ```
    ///
    /// [Mnemonic]: ../mnemonic/struct.Mnemonic.html
    pub fn from_string<S>(string: S,
                          word_list: WordList,
                          password: S) -> Result<Mnemonic, Error> where S: Into<String> {

        let m = string.into();
        let p = password.into();

        // this also validates the checksum and phrase length before returning the entropy so we
        // can store it. We don't use the validate function here to avoid having a public API that
        // takes a phrase string and returns the entropy directly. See the Mnemonic::entropy()
        // docs for the reason.
        let entropy = Mnemonic::entropy(&*m, word_list)?;
        let seed = Seed::generate(&m.as_bytes(), &p);

        let mnemonic = Mnemonic {
            string: (&m).clone(),
            seed,
            word_list,
            entropy
        };

        Ok(mnemonic)
    }

    /// Validate a mnemonic phrase
    ///
    /// The phrase supplied will be checked for word length and validated according to the checksum
    /// specified in BIP0039
    ///
    /// Note: you cannot use this function to determine anything more than whether the mnemonic
    /// phrase itself is intact, it does not check the password or compute the seed value. For that,
    /// you should use [`Mnemonic::from_string()`][Mnemonic::from_string()].
    ///
    /// # Example
    ///
    /// ```
    /// use bip39::{Mnemonic, Language};
    ///
    /// let test_mnemonic = "park remain person kitchen mule spell knee armed position rail grid ankle";
    ///
    /// match Mnemonic::validate(test_mnemonic, Language::English) {
    ///     Ok(_) => { println!("valid: {}", test_mnemonic); },
    ///     Err(e) => { println!("e: {}", e); return }
    /// }
    /// ```
    ///
    /// [Mnemonic::from_string()]: ../mnemonic/struct.Mnemonic.html#method.from_string
    pub fn validate<S>(string: S,
                       word_list: WordList) -> Result<(), Error> where S: Into<String> {
        Mnemonic::entropy(string, word_list).and(Ok(()))
    }

    /// Calculate the checksum, verify it and return the entropy
    ///
    /// Only intended for internal use, as returning a `Vec<u8>` that looks a bit like it could be
    /// used as the seed is likely to cause problems for someone eventually. All the other functions
    /// that return something like that are explicit about what it is and what to use it for.
    fn entropy<S>(string: S,
                  word_list: WordList) -> Result<Vec<u8>, Error> where S: Into<String> {
        let m = string.into();

        let mnemonic_type = MnemonicType::for_phrase(&*m)?;
        let entropy_bits = mnemonic_type.entropy_bits();
        let checksum_bits = mnemonic_type.checksum_bits();

        let word_map = word_list.gen_wordmap();

        let mut to_validate: BitVec = BitVec::new();

        for word in m.split(" ").into_iter() {
            let n = match word_map.get(word) {
                Some(n) => n,
                None => return Err(ErrorKind::InvalidWord.into())
            };
            for i in 0..11 {
                let bit = bit_from_u16_as_u11(*n, i);
                to_validate.push(bit);
            }
        }

        let mut checksum_to_validate = BitVec::new();
        &checksum_to_validate.extend((&to_validate).into_iter().skip(entropy_bits).take(checksum_bits));
        assert!(checksum_to_validate.len() == checksum_bits, "invalid checksum size");

        let mut entropy_to_validate = BitVec::new();
        &entropy_to_validate.extend((&to_validate).into_iter().take(entropy_bits));
        assert!(entropy_to_validate.len() == entropy_bits, "invalid entropy size");

        let entropy = entropy_to_validate.to_bytes();

        let hash = sha256(entropy.as_ref());

        let entropy_hash_to_validate_bits = BitVec::from_bytes(hash.as_ref());


        let mut new_checksum = BitVec::new();
        &new_checksum.extend(entropy_hash_to_validate_bits.into_iter().take(checksum_bits));
        assert!(new_checksum.len() == checksum_bits, "invalid new checksum size");
        if !(new_checksum == checksum_to_validate) {
            return Err(ErrorKind::InvalidChecksum.into())
        }

        Ok(entropy)
    }

    /// Get the original entropy value of the mnemonic phrase as an owned Vec<u8>
    ///
    /// # Example
    ///
    /// ```
    /// use bip39::{Mnemonic, Language};
    ///
    /// let test_mnemonic = "park remain person kitchen mule spell knee armed position rail grid ankle";
    ///
    /// let mnemonic = Mnemonic::from_string(test_mnemonic, Language::English, "").unwrap();
    ///
    /// let entropy: Vec<u8> = mnemonic.get_entropy();
    /// ```
    ///
    /// Note: this function clones the internal entropy bytes
    pub fn get_entropy(&self) -> Vec<u8> {
        self.entropy.clone()
    }

    /// Get the mnemonic phrase as a string reference
    pub fn as_str(&self) -> &str {
        self.string.as_ref()
    }

    /// Get the mnemonic phrase as an owned string
    ///
    /// Note: this clones the internal Mnemonic String instance
    pub fn get_string(&self) -> String {
        self.string.clone()
    }

    /// Get a reference to the internal [`Seed`][Seed]
    ///
    /// [Seed]: ../seed/struct.Seed.html
    pub fn as_seed(&self) -> &Seed {
        &self.seed
    }

    /// Get an owned [`Seed`][Seed].
    ///
    /// Note: this clones the internal [`Seed`][Seed] instance
    /// [Seed]: ../seed/struct.Seed.html
    pub fn get_seed(&self) -> Seed {
        self.seed.to_owned()
    }

    /// Get the original entropy used to create the Mnemonic as a hex string
    ///
    /// Note: this allocates a new String
    pub fn get_entropy_hex(&self) -> String {

        let hex = HEXUPPER.encode(self.as_entropy());

        hex
    }

    /// Get the original entropy value of the mnemonic phrase as a slice
    ///
    /// # Example
    ///
    /// ```
    /// use bip39::{Mnemonic, Language};
    ///
    /// let test_mnemonic = "park remain person kitchen mule spell knee armed position rail grid ankle";
    ///
    /// let mnemonic = Mnemonic::from_string(test_mnemonic, Language::English, "").unwrap();
    ///
    /// let entropy: &[u8] = mnemonic.as_entropy();
    /// ```
    ///
    /// Note: this function clones the internal entropy bytes
    pub fn as_entropy(&self) -> &[u8] {
        self.entropy.as_ref()
    }
}

impl AsRef<str> for Mnemonic {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

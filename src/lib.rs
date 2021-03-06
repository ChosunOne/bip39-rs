//!
//! This is a Rust implementation of the [bip39][bip39-standard] standard for Bitcoin HD wallet
//! mnemonic phrases.
//!
//!
//! [bip39-standard]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
//!
//! ## Quickstart
//!
//! ```rust
//! use bip39::{Mnemonic, MnemonicType, Seed};
//! use std::path::PathBuf;
//! use std::env;
//!
//! /// determines the number of words in the mnemonic phrase
//! let mnemonic_type = MnemonicType::Type12Words;
//! 
//! /// get the path where a language .json file is located
//! let mut path = PathBuf::from(env::current_dir().unwrap());
//! path.push("src/english.json");
//! 
//! /// create a new randomly generated mnemonic phrase
//! let mnemonic = match Mnemonic::new(mnemonic_type, path, "") {
//!     Ok(b) => b,
//!     Err(e) => { println!("e: {}", e); return }
//! };
//!
//! /// get the phrase as a string
//! let phrase = mnemonic.get_string();
//! println!("phrase: {}", phrase);
//!
//! /// get the HD wallet seed
//! let seed = mnemonic.get_seed();
//!
//! // get the HD wallet seed as raw bytes
//! let seed_bytes: &[u8] = seed.as_ref();
//!
//! // get the HD wallet seed as a hex string
//! let seed_hex: &str = seed.as_hex();
//!
//! // get an owned Seed instance
//! let owned_seed: Seed = seed.to_owned();
//!
//! ```
//!
#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
extern crate data_encoding;
extern crate bitreader;
extern crate bit_vec;
extern crate ring;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

mod mnemonic;
mod error;
mod mnemonic_type;
mod util;
mod seed;

mod crypto;

pub use mnemonic::Mnemonic;
pub use mnemonic_type::MnemonicType;
pub use seed::Seed;
pub use error::Error;
pub use error::ErrorKind;

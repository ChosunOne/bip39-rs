extern crate bip39;
extern crate serde;
extern crate serde_json;

use std::env;
use std::path::PathBuf;
use std::fs::File;
use ::bip39::Mnemonic;


#[derive(Debug, Clone)]
pub struct WordList {
    pub language: String,
    pub words: Vec<String>
}

#[test]
fn validate_12_english() {
    let test_mnemonic = "park remain person kitchen mule spell knee armed position rail grid ankle";

    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src/english.json");

    let file = File::open(path).unwrap();
    let word_list = serde_json::from_reader(file).expect("Could not read file");

    let _ = match Mnemonic::from_string(test_mnemonic, word_list, "") {
        Ok(b) => b,
        Err(_) => { assert!(false); return }
    };
}

#[test]
fn validate_15_english() {
    let test_mnemonic = "any paddle cabbage armor atom satoshi fiction night wisdom nasty they midnight chicken play phone";

    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src/english.json");

    let file = File::open(path).unwrap();
    let word_list = serde_json::from_reader(file).expect("Could not read file");

    let _ = match Mnemonic::from_string(test_mnemonic, word_list, "") {
        Ok(b) => b,
        Err(_) => { assert!(false); return }
    };
}

#[test]
fn validate_18_english() {
    let test_mnemonic = "soda oak spy claim best oppose gun ghost school use sign shock sign pipe vote follow category filter";

    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src/english.json");

    let file = File::open(path).unwrap();
    let word_list = serde_json::from_reader(file).expect("Could not read file");

    let _ = match Mnemonic::from_string(test_mnemonic, word_list, "") {
        Ok(b) => b,
        Err(_) => { assert!(false); return }
    };
}


#[test]
fn validate_21_english() {
    let test_mnemonic = "quality useless orient offer pole host amazing title only clog sight wild anxiety gloom market rescue fan language entry fan oyster";

    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src/english.json");

    let file = File::open(path).unwrap();
    let word_list = serde_json::from_reader(file).expect("Could not read file");

    let _ = match Mnemonic::from_string(test_mnemonic, word_list, "") {
        Ok(b) => b,
        Err(_) => { assert!(false); return }
    };
}


#[test]
fn validate_24_english() {
    let test_mnemonic = "always guess retreat devote warm poem giraffe thought prize ready maple daughter girl feel clay silent lemon bracket abstract basket toe tiny sword world";

    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src/english.json");

    let file = File::open(path).unwrap();
    let word_list = serde_json::from_reader(file).expect("Could not read file");

    let _ = match Mnemonic::from_string(test_mnemonic, word_list, "") {
        Ok(b) => b,
        Err(_) => { assert!(false); return }
    };
}


#[test]
fn validate_12_english_uppercase() {
    let invalid_mnemonic = "Park remain person kitchen mule spell knee armed position rail grid ankle";

    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push("src/english.json");

    let file = File::open(path).unwrap();
    let word_list = serde_json::from_reader(file).expect("Could not read file");

    let _ = match Mnemonic::from_string(invalid_mnemonic, word_list, "") {
        Ok(_) => { assert!(false); return },
        Err(_) => {},
    };
}

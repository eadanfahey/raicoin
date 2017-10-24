use secp256k1::key::{SecretKey, PublicKey};
use secp256k1::Secp256k1;
use rand::OsRng;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use serialize::{serialize, deserialize};


#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

#[derive(Serialize, Deserialize)]
pub struct Wallets(HashMap<String, Wallet>);

pub fn hash_public_key(pubkey: &PublicKey) -> String {
    let secp = Secp256k1::new();
    let mut hash = Sha256::new();
    hash.input(&pubkey.serialize_vec(&secp, true)[..]);

    hash.result_str()
}

impl Wallet {
    pub fn new() -> Wallet {
        let secp = Secp256k1::new();
        let pair = secp.generate_keypair(&mut OsRng::new().unwrap()).unwrap();

        Wallet {
            secret_key: pair.0,
            public_key: pair.1,
        }
    }
}

impl Wallets {
    pub fn save(&self) {
        let mut file = File::create("wallets.json").unwrap();
        file.write_all(serialize(self).as_bytes()).unwrap()
    }

    pub fn open() -> Wallets {
        let file = File::open("wallets.json");

        match file {
            Ok(mut f) => {
                let mut contents = String::new();
                f.read_to_string(&mut contents).unwrap();

                deserialize(&contents)
            },
            Err(_) => {
                Wallets(HashMap::new())
            }
        }

    }

    pub fn add(&mut self, wallet: Wallet) {
        let address = hash_public_key(&wallet.public_key);
        self.0.insert(address, wallet);
    }

    pub fn get(&self, address: &str) -> Option<&Wallet> {
        self.0.get(address)
    }
}

impl Drop for Wallets {
    fn drop(&mut self) {
        self.save();
    }
}

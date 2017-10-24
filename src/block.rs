use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde_json;
use serialize::serialize;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use num::bigint::BigInt;
use num::traits::One;
use num::Num;
use constants::DIFFICULTY;

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub timestamp: i64,
    pub data: String,
    pub prev_block_hash: String,
    pub nonce: i64,
}

impl Block {
    pub fn hash(&self) -> String {
        let mut hash = Sha256::new();
        hash.input_str(&serialize(&self));

        hash.result_str()
    }

    pub fn mine(data: String, prev_block_hash: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut block = Block {
            timestamp,
            data,
            prev_block_hash,
            nonce: 0,
        };

        let target = BigInt::one() << (256 - DIFFICULTY);

        loop {
            let hash_int = BigInt::from_str_radix(&block.hash(), 16).unwrap();
            if hash_int < target {
                break;
            } else {
                block.nonce += 1
            }
        }

        block
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&serde_json::to_string_pretty(self).unwrap())
    }
}

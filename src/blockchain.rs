use block::Block;
use std::collections::HashMap;
use num::bigint::BigInt;
use num::traits::One;
use num::Num;
use constants::{DIFFICULTY, BLOCKCHAIN};
use std::fs::File;
use std::io::prelude::*;
use serialize::{deserialize, serialize};
use transaction::{TX, CoinbaseTX};

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    blocks: HashMap<String, Block>,
    pub last_block_hash: String,
}


impl Blockchain {
    pub fn get_block(&self, hash: &str) -> Option<&Block> {
        self.blocks.get(hash)
    }

    fn validate_block(&self, prev_block_hash: &str, block: &Block) {
        let target = BigInt::one() << (256 - DIFFICULTY);
        let hash_int = BigInt::from_str_radix(&block.hash(), 16).unwrap();

        if block.prev_block_hash != prev_block_hash {
            panic!("Error: invalid previous_block_hash")
        } else if hash_int > target {
            panic!("Error: invalid nonce")
        }

        for tx in block.transactions.iter() {
            match tx {
                &TX::Coinbase(_) => {},
                &TX::Standard(ref tx) => {
                    if !tx.verify(&self) {
                        panic!("Error: invalid transaction")
                    }
                }
            }
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.validate_block(&self.last_block_hash, &block);
        self.last_block_hash = block.hash();
        self.blocks.insert(block.hash(), block);
    }

    fn validate_chain(&self) {
        let chain: Vec<(&str, &Block)> = self.iter().collect();
        let mut prev_hash = "".to_owned();

        for &(hash, block) in chain.iter().rev() {
            self.validate_block(&prev_hash, block);
            prev_hash = hash.to_owned();
        }
    }

    pub fn open() -> Blockchain {
        let mut file = File::open(BLOCKCHAIN).expect(
            "A blockchain does not exist. Create one!"
        );
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let blockchain: Blockchain = deserialize(&contents);
        blockchain.validate_chain();

        blockchain
    }

    pub fn save(&self) {
        let mut file = File::create(BLOCKCHAIN).unwrap();
        file.write_all(serialize(self).as_bytes()).unwrap()
    }

    pub fn new(genesis_address: &str) -> Blockchain {
        let mut blockchain = Blockchain {
            blocks: HashMap::new(),
            last_block_hash: String::new(),
        };

        let prev_block_hash = "".to_owned();
        let tx = TX::Coinbase(CoinbaseTX::new(genesis_address.to_owned()));
        let genesis = Block::mine(vec![tx], prev_block_hash);
        blockchain.add_block(genesis);

        blockchain
    }

    pub fn find_transaction(&self, txid: &str) -> Option<&TX> {
        self.iter()
            .flat_map(|(_, block)| block.transactions.iter())
            .find(|tx| tx.id() == txid)
    }

    pub fn iter(&self) -> IterBlockchain {
        IterBlockchain {
            blockchain: &self,
            current_hash: &self.last_block_hash,
        }
    }
}

impl Drop for Blockchain {
    fn drop(&mut self) {
        self.save();
    }
}

pub struct IterBlockchain<'a> {
    blockchain: &'a Blockchain,
    current_hash: &'a str,
}

impl<'a> Iterator for IterBlockchain<'a> {
    type Item = (&'a str, &'a Block);

    fn next(&mut self) -> Option<Self::Item> {
        let hash = &self.current_hash.clone();
        let block = self.blockchain.get_block(&hash);
        match block {
            Some(v) => {
                self.current_hash = &v.prev_block_hash;
                Some((hash, v))
            }
            None => None,
        }
    }
}

use block::Block;
use std::collections::HashMap;


#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    blocks: HashMap<String, Block>,
    pub last_block_hash: String,
}

impl Blockchain {
    pub fn get_block(&self, hash: &str) -> Option<&Block> {
        self.blocks.get(hash)
    }

    fn validate_block(&self, block: &Block) {
        if block.prev_block_hash != self.last_block_hash {
            panic!("Error: invalid previous_block_hash")
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.validate_block(&block);
        self.last_block_hash = block.hash();
        self.blocks.insert(block.hash(), block);
    }

    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            blocks: HashMap::new(),
            last_block_hash: String::new(),
        };

        let data = "Genesis block".to_owned();
        let prev_block_hash = "".to_owned();
        let genesis = Block::mine(data, prev_block_hash);
        blockchain.add_block(genesis);

        blockchain
    }

    pub fn iter(&self) -> IterBlockchain {
        IterBlockchain {
            blockchain: &self,
            current_hash: &self.last_block_hash,
        }
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

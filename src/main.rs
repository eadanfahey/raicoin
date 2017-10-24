extern crate raicoin;
use raicoin::blockchain::Blockchain;
use raicoin::block::Block;

fn main() {

    let mut blockchain = Blockchain::new();

    let block1 = Block::mine(
        "Infinte Jest".to_owned(),
        blockchain.last_block_hash.clone(),
    );
    blockchain.add_block(block1);

    let block2 = Block::mine(
        "Gravity's Rainbow".to_owned(),
        blockchain.last_block_hash.clone(),
    );
    blockchain.add_block(block2);

    for (hash, block) in blockchain.iter() {
        println!("==============================\n");
        println!("hash: {}\ncontents: {}\n", hash, block);
    }
}

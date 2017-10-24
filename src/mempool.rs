use transaction::*;
use std::fs::File;
use std::io::prelude::*;
use serialize::{serialize, deserialize};
use std::collections::{HashSet, VecDeque};
use blockchain::Blockchain;
use error::Result;
use constants::MEMPOOL;

#[derive(Serialize, Deserialize)]
pub struct MemPool {
    txs: VecDeque<TX>,
    tx_ids: HashSet<String>,
}


impl MemPool {
    fn save(&self) {
        let mut file = File::create(MEMPOOL).unwrap();
        file.write_all(serialize(self).as_bytes()).unwrap()
    }

    pub fn open() -> MemPool {
        let file = File::open(MEMPOOL);

        match file {
            Ok(mut f) => {
                let mut contents = String::new();
                f.read_to_string(&mut contents).unwrap();

                deserialize(&contents)
            }
            Err(_) => MemPool {
                txs: VecDeque::new(),
                tx_ids: HashSet::new(),
            },
        }
    }

    pub fn push(&mut self, bc: &Blockchain, tx: TX) -> Result<()> {

        let txid = tx.id();
        match tx {
            TX::Coinbase(_) => (),
            TX::Standard(ref stx) => {
                match stx.verify(bc){
                    Ok(()) => {
                        if self.tx_ids.insert(txid) {
                            self.txs.push_back(tx.clone());
                        }
                    }
                    Err(e) => return Err(e)
                }
            }
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Option<TX> {
        let tx = self.txs.pop_front();
        match tx {
            Some(ref tx) => { self.tx_ids.remove(&tx.id()); },
            None => (),
        }

        tx
    }
}


impl Drop for MemPool {
    fn drop(&mut self) {
        self.save();
    }
}

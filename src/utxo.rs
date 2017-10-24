use transaction::{TX, TXOutput};
use blockchain::Blockchain;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct UTXOEntry {
    pub vout: usize,
    pub output: TXOutput,
}

pub type UTXOEntries = Vec<UTXOEntry>;

pub type UTXO = HashMap<String, UTXOEntries>;


fn new_entries(tx: &TX) -> UTXOEntries {
    // Make a fresh collection of UTXO entries for a transaction
    tx.outputs()
        .iter()
        .enumerate()
        .map(|(i, output)| {
            UTXOEntry {
                vout: i,
                output: output.clone(),
            }
        })
        .collect()
}

pub fn find(bc: &Blockchain) -> UTXO {
    let mut utxo: UTXO = HashMap::new();

    // First, add all outputs to the UTXO set
    for (_, block) in bc.iter() {
        for tx in block.transactions.iter() {
            utxo.insert(tx.id(), new_entries(tx));
        }
    }

    // Then, filter the UTXO set to remove outputs referenced in an input
    for (_, block) in bc.iter() {
        for tx in block.transactions.iter() {
            match tx {
                &TX::Coinbase(_) => (),
                &TX::Standard(ref tx) => {
                    for input in tx.inputs.iter() {
                        utxo.get_mut(&input.txid)
                            .unwrap()
                            .retain(|entry| entry.vout != input.vout);
                    }
                }
            }
        }
    }

    utxo
}

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serialize::serialize;
use utxo;
use blockchain::Blockchain;
use rand::OsRng;
use rand::Rng;

const REWARD: u64 = 50;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXInput {
    pub txid: String,
    pub vout: usize,
    pub sender: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXOutput {
    pub value: u64,
    pub recipient: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CoinbaseTX {
    outputs: Vec<TXOutput>,
    rand: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StandardTX {
    pub outputs: Vec<TXOutput>,
    pub inputs: Vec<TXInput>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TX {
    Coinbase(CoinbaseTX),
    Standard(StandardTX),
}

impl CoinbaseTX {
    pub fn new(to: String) -> CoinbaseTX {
        let out = TXOutput {
            value: REWARD,
            recipient: to,
        };
        let outputs = vec![out];

        let rand = OsRng::new().unwrap().next_u64();

        CoinbaseTX { outputs, rand }
    }
}

impl StandardTX {
    pub fn new(bc: &Blockchain, from: &str, to: &str, amount: u64) -> StandardTX {
        let utxo = utxo::find(bc);

        // Find the outputs needed for the new transaction inputs
        let mut acc_amount = 0;
        let old_outputs: Vec<(String, usize, TXOutput)> = utxo.iter()
            .flat_map(|(txid, entries)| entries.iter().map(move |entry| (txid, entry)))
            .filter(|&(_, entry)| entry.output.recipient == from)
            .take_while(move |_| acc_amount <= amount)
            .map(|(txid, entry)| {
                acc_amount += entry.output.value;
                (txid.to_owned(), entry.vout, entry.output.clone())
            })
            .collect();

        if acc_amount < amount {
            panic!("Insufficient funds");
        }

        // Make the new transaction outputs
        let send = TXOutput {
            value: amount,
            recipient: to.to_owned(),
        };
        let change = TXOutput {
            value: acc_amount - amount,
            recipient: from.to_owned(),
        };
        let new_outputs = vec![send, change];

        // Make the new transaction inputs
        let new_inputs = old_outputs
            .into_iter()
            .map(|(txid, vout, _)| {
                TXInput {
                    txid,
                    vout,
                    sender: from.to_owned()
                }
            })
            .collect();

        StandardTX {
            outputs: new_outputs,
            inputs: new_inputs,
        }
    }
}


impl TX {
    pub fn id(&self) -> String {
        let mut hash = Sha256::new();
        match self {
            &TX::Coinbase(ref tx) => hash.input_str(&serialize(&tx)),
            &TX::Standard(ref tx) => hash.input_str(&serialize(&tx)),
        };
        hash.result_str()
    }

    pub fn outputs(&self) -> &Vec<TXOutput> {
        match self {
            &TX::Standard(ref tx) => &tx.outputs,
            &TX::Coinbase(ref tx) => &tx.outputs,
        }
    }
}

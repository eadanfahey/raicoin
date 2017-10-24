use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serialize::serialize;
use utxo;
use blockchain::Blockchain;
use secp256k1::key::{PublicKey, SecretKey};
use secp256k1::{Secp256k1, Message, Signature};
use wallet::{hash_public_key, Wallet};
use rand::OsRng;
use rand::Rng;
use error::{Error, Result};

const REWARD: u64 = 50;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXInput {
    pub txid: String,
    pub vout: usize,
    pub signature: Signature,
    pub pubkey: PublicKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXOutput {
    pub value: u64,
    pub pubkey_hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CoinbaseTX {
    outputs: Vec<TXOutput>,
    rand: u64
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

#[derive(Serialize, Deserialize)]
struct TransactionData {
    pubkey_hash: String,
    outputs: Vec<TXOutput>,
}

impl TransactionData {
    fn id(&self) -> Vec<u8> {
        let mut hash = Sha256::new();
        hash.input_str(&serialize(&self));
        let mut res = vec![0; 32];
        hash.result(&mut res);

        res
    }

    fn sign(&self, sk: &SecretKey) -> Signature {
        let secp = Secp256k1::new();
        let data = self.id();
        let msg = Message::from_slice(&data).unwrap();
        let signature = secp.sign(&msg, sk).unwrap();

        signature
    }

    fn verify(&self, sig: &Signature, pk: &PublicKey) -> bool {
        let data = self.id();
        let msg = Message::from_slice(&data).unwrap();
        let secp = Secp256k1::new();

        match secp.verify(&msg, sig, pk) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl CoinbaseTX {
    pub fn new(to: String) -> CoinbaseTX {
        let out = TXOutput {
            value: REWARD,
            pubkey_hash: to,
        };
        let outputs = vec![out];

        let rand = OsRng::new().unwrap().next_u64();

        CoinbaseTX { outputs, rand }
    }
}

impl StandardTX {
    pub fn new(bc: &Blockchain, wallet: &Wallet, to: &str, amount: u64) -> Result<StandardTX> {
        let utxo = utxo::find(bc);

        // Find the outputs needed for the new transaction inputs
        let mut acc_amount = 0;
        let old_outputs: Vec<(String, usize, TXOutput)> = utxo.iter()
            .flat_map(|(txid, entries)| {
                entries.iter().map(move |entry| (txid, entry))
            })
            .filter(|&(_, entry)| {
                entry.output.pubkey_hash == hash_public_key(&wallet.public_key)
            })
            .take_while(move |_| acc_amount <= amount)
            .map(|(txid, entry)| {
                acc_amount += entry.output.value;
                (txid.to_owned(), entry.vout, entry.output.clone())
            })
            .collect();

        if acc_amount < amount {
            return Err(Error::InsufficientFunds);
        }

        // Make the new transaction outputs
        let send = TXOutput {
            value: amount,
            pubkey_hash: to.to_owned(),
        };
        let change = TXOutput {
            value: acc_amount - amount,
            pubkey_hash: hash_public_key(&wallet.public_key),
        };
        let new_outputs = vec![send, change];

        // Make the new tranaction inputs
        let new_inputs = old_outputs
            .into_iter()
            .map(|(txid, vout, output)| {
                let data = TransactionData {
                    pubkey_hash: output.pubkey_hash,
                    outputs: new_outputs.clone(),
                };

                TXInput {
                    txid: txid,
                    vout,
                    signature: data.sign(&wallet.secret_key),
                    pubkey: wallet.public_key.clone(),
                }

            })
            .collect();

        Ok(StandardTX {
            inputs: new_inputs,
            outputs: new_outputs,
        })
    }

    fn verify_input(&self, input: &TXInput, bc: &Blockchain) -> Result<()> {
        use self::Error::*;
        match bc.find_transaction(&input.txid) {
            None => { return Err(TransactionMissing); },
            Some(prev_tx) => {
                match prev_tx.outputs().get(input.vout) {
                    None => { return Err(NoTXOutput); },
                    Some(prev_output) => {
                        let data = TransactionData {
                            pubkey_hash: prev_output.pubkey_hash.clone(),
                            outputs: self.outputs.clone(),
                        };
                        if !data.verify(&input.signature, &input.pubkey) {
                            return Err(PubkeySignatureMismatch);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn verify(&self, bc: &Blockchain) -> Result<()> {
        for input in self.inputs.iter() {
            match self.verify_input(input, bc) {
                Ok(_) => {},
                Err(e) => { return Err(e); }
            }

        }

        Ok(())
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

use clap::{App, Arg};
use blockchain::Blockchain;
use block::Block;
use utxo;
use transaction::*;
use wallet::{Wallet, Wallets, hash_public_key};
use std::collections::HashMap;
use mempool::MemPool;
use error::{Result, Error};

enum Operation {
    NewChain,
    PrintChain,
    Balances,
    Send(String, String, u64),
    NewWallet,
    Mine(String)
}

fn parse_args() -> Operation {

    let matches = App::new("raicoin")
        .arg(
            Arg::with_name("operation")
                .help("The type of operation")
                .possible_values(&["newchain", "printchain", "balance", "send", "newwallet", "mine"])
                .required(true),
        )
        .arg(
            Arg::with_name("from")
                .help("Address to send from")
                .required_if("operation", "send")
                .long("from")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("to")
                .help("Address to send to")
                .required_if("operation", "send")
                .long("to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("amount")
                .help("Amount to send")
                .required_if("operation", "send")
                .long("amount")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("rewardto")
                .help("The address to send the block reward to")
                .required_if("operation", "mine")
                .long("rewardto")
                .takes_value(true)
        )
        .get_matches();

    let operation = matches.value_of("operation").unwrap();


    if operation == "printchain" {
        Operation::PrintChain
    } else if operation == "newchain" {
        Operation::NewChain
    } else if operation == "balance" {
        Operation::Balances
    } else if operation == "send" {
        let from = matches.value_of("from").unwrap();
        let to = matches.value_of("to").unwrap();
        let amount = matches.value_of("amount").unwrap().parse::<u64>().expect(
            "Amount must be a positive integer",
        );
        Operation::Send(from.to_owned(), to.to_owned(), amount)
    } else if operation == "newwallet" {
        Operation::NewWallet
    } else if operation == "mine" {
        let reward_to = matches.value_of("rewardto").unwrap();
        Operation::Mine(reward_to.to_owned())
    }
    else {
        panic!("Unknown argument {}", operation)
     }
}

fn get_balances(bc: &Blockchain) -> HashMap<String, u64> {
    utxo::find(bc)
        .iter()
        .flat_map(|(_, entries)| entries.iter())
        .fold(HashMap::new(), |mut acc, entry| {
            let pubkey_hash = entry.output.pubkey_hash.clone();
            let value = entry.output.value;
            *acc.entry(pubkey_hash).or_insert(0) += value;
            acc
        })
}

fn send(mp: &mut MemPool, bc: &Blockchain, from: &str, to:&str, amount: u64) -> Result<()> {

    let wallets = Wallets::open();
    let from_wallet = wallets.get(from).ok_or(Error::NoWalletForAddress)?;

    let tx = TX::Standard(StandardTX::new(bc, from_wallet, to, amount)?);
    mp.push(bc, tx)?;

    Ok(())
}

fn mine(mp: &mut MemPool, bc: &mut Blockchain, reward_to: &str) -> Result<()> {
    let reward = TX::Coinbase(CoinbaseTX::new(reward_to.to_owned()));

    let transactions = match mp.pop() {
        Some(tx) => vec![tx, reward],
        None => vec![reward]
    };
    let prev_block_hash = bc.last_block_hash.clone();

    let block = Block::mine(transactions, prev_block_hash);
    bc.add_block(block)?;

    Ok(())
}

pub fn run() -> Result<()> {
    let operation = parse_args();

    match operation {
        Operation::NewChain => {
            let mut wallets = Wallets::open();
            let wallet = Wallet::new();
            let address = hash_public_key(&wallet.public_key);
            Blockchain::new(&address)?;
            wallets.add(wallet);
            println!("Created a new blockchain");
        }
        Operation::PrintChain => {
            let blockchain = Blockchain::open()?;
            for (hash, block) in blockchain.iter() {
                println!("==============================\n");
                println!("hash: {}\ncontents: {}\n", hash, block);
            }
        }
        Operation::Balances => {
            let bc = Blockchain::open()?;
            for (address, balance) in get_balances(&bc) {
                println!("{}: {}", address, balance);
            }
        }
        Operation::Send(from, to, amount) => {
            let mut bc = Blockchain::open()?;
            let mut mp = MemPool::open();
            send(&mut mp, &mut bc, &from, &to, amount)?;
            println!("Sent transaction to the mempool")
        }
        Operation::NewWallet => {
            let mut wallets = Wallets::open();
            let wallet = Wallet::new();
            let address = hash_public_key(&wallet.public_key);
            println!("Created wallet:\n{}", address);
            wallets.add(wallet);

        }
        Operation::Mine(reward_to) => {
            let bc = &mut Blockchain::open()?;
            let mp = &mut MemPool::open();
            mine(mp, bc, &reward_to)?;
            println!("Mined a block and added it to the blockchain");
        }
    }

    Ok(())
}

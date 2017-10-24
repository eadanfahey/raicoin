use clap::{App, Arg};
use blockchain::Blockchain;
use block::Block;
use utxo;
use transaction::*;
use std::collections::HashMap;

enum Operation {
    NewChain,
    PrintChain,
    Balances,
    Send(String, String, u64),
}

fn parse_args() -> Operation {

    let matches = App::new("raicoin")
        .arg(
            Arg::with_name("operation")
                .help("The type of operation")
                .possible_values(&["newchain", "printchain", "balance", "send"])
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
    } else {
        panic!("Unknown argument {}", operation)
    }
}

fn get_balances(bc: &Blockchain) -> HashMap<String, u64> {
    utxo::find(bc)
        .iter()
        .flat_map(|(_, entries)| entries.iter())
        .fold(HashMap::new(), |mut acc, entry| {
            let recipient = entry.output.recipient.clone();
            let value = entry.output.value;
            *acc.entry(recipient).or_insert(0) += value;
            acc
        })
}

fn send(bc: &mut Blockchain, from: &str, to: &str, amount: u64) {
    let tx = TX::Standard(StandardTX::new(bc, from, to, amount));
    let block = Block::mine(vec![tx], bc.last_block_hash.to_owned());

    bc.add_block(block);
}

pub fn run() {
    let operation = parse_args();

    match operation {
        Operation::NewChain => {
            Blockchain::new();
            println!("Created a new blockchain");
        }
        Operation::PrintChain => {
            let blockchain = Blockchain::open();
            for (hash, block) in blockchain.iter() {
                println!("==============================\n");
                println!("hash: {}\ncontents: {}\n", hash, block);
            }
        }
        Operation::Balances => {
            let bc = Blockchain::open();
            for (address, balance) in get_balances(&bc) {
                println!("{}: {}", address, balance);
            }
        }
        Operation::Send(from, to, amount) => {
            let mut bc = Blockchain::open();
            send(&mut bc, &from, &to, amount);
        }
    }
}

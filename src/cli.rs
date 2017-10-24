use clap::{App, Arg};
use blockchain::Blockchain;
use block::Block;

enum Operation {
    NewChain,
    PrintChain,
    AddBlock(String),
}

fn parse_args() -> Operation {

    let matches = App::new("raicoin")
        .arg(
            Arg::with_name("operation")
                .help("The type of operation")
                .possible_values(&["newchain", "printchain", "addblock"])
                .required(true),
        )
        .arg(
            Arg::with_name("data")
                .help("The data to add to the blockchain")
                .required_if("operation", "addblock")
                .long("data")
                .takes_value(true),
        )
        .get_matches();

    let operation = matches.value_of("operation").unwrap();

    if operation == "printchain" {
        Operation::PrintChain
    } else if operation == "newchain" {
        Operation::NewChain
    } else if operation == "addblock" {
        let data = matches.value_of("data").unwrap().to_owned();
        Operation::AddBlock(data)
    } else {
        panic!("Unknown argument {}", operation)
    }
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
        Operation::AddBlock(data) => {
            let mut blockchain = Blockchain::open();
            let block = Block::mine(data, blockchain.last_block_hash.to_owned());
            blockchain.add_block(block);
        }
    }
}

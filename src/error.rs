use std::result;

pub enum Error {
    InsufficientFunds,
    TransactionMissing,
    NoTXOutput,
    PubkeySignatureMismatch,
    InvalidPreviousHash,
    InvalidNonce,
    TooManyCoinbase,
    NoWalletForAddress,
}

impl Error {
    pub fn to_string(&self) -> String {
        use self::Error::*;
        let x = match self {
            &InsufficientFunds => "insufficient funds",
            &TransactionMissing => "transaction does not exist",
            &NoTXOutput => "transaction output does not exist",
            &PubkeySignatureMismatch => "public key does not match the signature",
            &InvalidPreviousHash => "previous_block_hash of the block is incorrect",
            &InvalidNonce => "the block nonce is incorrect",
            &TooManyCoinbase => "too many coinbase transactions in the block",
            &NoWalletForAddress => "a wallet does not exist for this address",
        };

        format!("Error: {}", x)
    }
}

pub type Result<T> = result::Result<T, Error>;

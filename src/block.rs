use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::PartialEq;
use std::fmt;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Block {
    pub hash: String,
    pub prev_hash: String,
    pub transactions: Vec<Transaction>,
    pub time: SystemTime,
    pub nonce: u64,
}

impl Block {
    pub fn new(prev: String, txs: Vec<Transaction>, nonce: u64, ms: SystemTime) -> Self {
        Block {
            hash: String::new(),
            prev_hash: prev,
            transactions: txs,
            nonce: nonce,
            time: ms,
        }
    }

    pub fn generate_hash(&mut self) -> String {
        let block_string = serde_json::to_string(&self);

        let hashed = Sha256::new().chain_update(block_string.unwrap()).finalize();

        self.hash = format!("{:x}", hashed);
        self.hash.clone()
    }

    pub fn is_valid(&self, prev_block: &Block) -> bool {
        self.prev_hash == prev_block.hash
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let mut result_string = String::new();
        result_string.push_str(&("=".repeat(30) + "\r\n"));
        result_string.push_str(&("Hash: ".to_owned() + &self.hash + "\r\n"));
        result_string.push_str(&("Prev Hash: ".to_owned() + &self.prev_hash + "\r\n"));
        result_string
            .push_str(&("Tx len: ".to_owned() + &self.transactions.len().to_string() + "\r\n"));
        result_string
            .push_str(&("Nonce: ".to_owned() + &self.nonce.to_string() + "\r\n"));
        result_string.push_str(
            &("Time: ".to_owned() + &self.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64().to_string() + "\r\n"),
        );
        result_string.push_str(&("=".repeat(30) + "\r\r\n\n"));

        write!(f, "{}", result_string)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{block::Block, transaction::Transaction};
    use std::time::SystemTime;

    pub fn generate_blocks() -> Vec<Block> {
        let time_now = SystemTime::now();

        let tx1 = Transaction {
            from: String::from("Alice"),
            to: String::from("Bob"),
            time: time_now,
            amount: 32,
        };

        let new_block = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![tx1.clone()],
            time: time_now,
            nonce: 0,
        };

        let same_block = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![tx1.clone()],
            time: time_now,
            nonce: 0,
        };

        let next_block = Block {
            hash: String::from("new_hash"),
            prev_hash: String::from("123"),
            transactions: vec![tx1.clone()],
            time: time_now,
            nonce: 0,
        };

        vec![new_block, same_block, next_block]
    }

    #[test]
    fn test_block() {
        let time_now = SystemTime::now();

        let tx1 = Transaction {
            from: String::from("Alice"),
            to: String::from("Bob"),
            time: time_now,
            amount: 32,
        };

        let blocks = generate_blocks();
        let mut new_block = blocks[0].clone();
        let next_block = blocks[2].clone();
        let mut same_block = blocks[1].clone();

        assert!(next_block.is_valid(&new_block));

        let first_block_digest = new_block.generate_hash();
        assert_eq!(first_block_digest, same_block.generate_hash());

        let time_now2 = SystemTime::now();
        let second_block_time_differ = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![tx1.clone()],
            time: time_now2,
            nonce: 0,
        };

        assert_ne!(
            first_block_digest,
            second_block_time_differ.clone().generate_hash()
        );

        let tx2 = Transaction {
            from: String::from("Bob"),
            to: String::from("Alice"),
            time: time_now2,
            amount: 32,
        };
        let second_block_txs_differ = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![tx1, tx2],
            time: time_now2,
            nonce: 0,
        };

        assert_ne!(
            second_block_time_differ.clone().generate_hash(),
            second_block_txs_differ.clone().generate_hash()
        );
    }
}

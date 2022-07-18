use crate::{block::Block, transaction::Transaction};
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt;
use std::time::SystemTime;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    difficulty: usize,
    concurrent_hashes: u64,
    min_tx_per_block: u8,
}

impl Blockchain {
    pub fn new(min_tx_per_block: u8, difficulty: usize, concurrent_hashes: u64) -> Self {
        Blockchain {
            chain: vec![],
            difficulty,
            concurrent_hashes,
            min_tx_per_block,
        }
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn add_block(&mut self, block: Block) {
        if self.chain.len() == 0 {
            return;
        }

        let latest_block = self.chain.last().unwrap();
        if block.is_valid(latest_block) {
            self.chain.push(block);
        } 
        // Here in else we should add this block to orphans, but we will not do it
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            if !(self
                .chain
                .get(i)
                .unwrap()
                .is_valid(self.chain.get(i - 1).unwrap()))
            {
                return false;
            }
        }

        true
    }

    pub fn try_mine(&mut self, txs: Vec<Transaction>) -> bool {
        let success;
        if txs.len() < self.min_tx_per_block.into() {
            println!(
                "Not enough txs to mine block. Current txs {}, Current min is {}",
                txs.len(),
                self.min_tx_per_block
            );
            success = false;
        } else {
            let mut nonce = 0;
            loop {
                let time = SystemTime::now();

                let block = self.mine_block(nonce, time, txs.clone());
                match block {
                    Some(block) => {
                        self.chain.push(block);
                        success = true;
                        break;
                    }
                    None => {}
                };

                nonce += self.concurrent_hashes;
            }
        }
        success
    }

    fn mine_block(
        &self,
        nonce: u64,
        time: SystemTime,
        txs: Vec<Transaction>
    ) -> Option<Block> {
        const CHARSET: &[u8] = b"abcdef\
                            0123456789";
        let mut rng = rand::thread_rng();

        let mut mine_target: String = (0..self.difficulty)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        mine_target = mine_target.to_lowercase();

        let nonces: Vec<u64> = (0..self.concurrent_hashes).map(|x| x + nonce).collect();

        let prev = match self.chain.len() {
            0 => String::new(),
            _ => self.chain.get(self.chain.len() - 1).unwrap().hash.clone(),
        };

        nonces.par_iter().find_map_any(move |&nonce| {
            let mut block = Block::new(prev.clone(), txs.clone(), nonce, time);

            let hash = block.generate_hash();

            if hash.starts_with(&mine_target) {
                println!("\nMined! {}\n", block.hash.clone());
                return Some(block);
            }

            None
        })
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let mut display_chain = String::new();

        for i in 0..self.chain.len() {
            display_chain.push_str(&("-".repeat(14)));
            display_chain.push_str(&(i.to_string()));
            display_chain.push_str(&("-".repeat(15) + "\r\n"));
            display_chain.push_str(&self.chain[i].to_string());
        }
        if self.chain.len() == 0 {
            display_chain.push_str("Chain is empty for now. Try to generate few transactions");
        }
        write!(f, "{}\n", display_chain)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::block::tests::generate_blocks;
    use crate::{blockchain::Blockchain, transaction::Transaction};
    use std::time::SystemTime;

    pub fn generate_blockchain() -> Blockchain {
        let blocks = generate_blocks();
        let chain = Blockchain {
            chain: blocks,
            difficulty: 5,
            min_tx_per_block: 3,
            concurrent_hashes: 256,
        };

        chain
    }

    #[test]
    fn test_chain_validity() {
        let blocks = crate::block::tests::generate_blocks();

        let chain = Blockchain {
            chain: blocks,
            difficulty: 1,
            min_tx_per_block: 1,
            concurrent_hashes: 256,
        };

        assert!(chain.is_valid());
    }

    #[test]
    fn test_display() {
        let blocks = crate::block::tests::generate_blocks();

        let chain = Blockchain {
            chain: blocks,
            difficulty: 1,
            min_tx_per_block: 1,
            concurrent_hashes: 256,
        };

        println!("{}", chain);
    }
    #[test]
    fn test_mining() {
        let mut txs: Vec<Transaction> = vec![];
        for i in 0..10 {
            txs.push(Transaction {
                from: String::from("test"),
                to: String::from(i.to_string()),
                amount: i,
                time: SystemTime::now(),
            });
        }

        let concurrent_hashes = 256;
        let chain = Blockchain::new(5, 3, concurrent_hashes);
        let mut _nonce = 0;
        let time = SystemTime::now();

        let mut cntr = 0;
        loop {
            cntr += 1;

            chain.mine_block(1, time, txs.clone());
            _nonce += concurrent_hashes;
            if cntr == 100 {
                break;
            }
        }
    }
}

use std::time::SystemTime;

use crate::blockchain::Blockchain;

pub struct Node {
    pub blockchain: Blockchain,
    pub last_time_synced: f64,
}

impl Node {
    pub fn resolve_chain_conflict(&mut self, other: &Blockchain) {
        let own_valid = self.blockchain.is_valid();
        let other_valid = other.is_valid();
        let correct_chain;
        if own_valid && other_valid {
            if self.blockchain.len() >= other.len() {
                correct_chain = &self.blockchain;
            } else {
                correct_chain = &other;
            }
        } else if other_valid {
            correct_chain = &other;
        } else if own_valid {
            correct_chain = &self.blockchain;
        } else {
            panic!("All chains are invalid");
        }
        self.last_time_synced = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();
		

		self.blockchain.chain = correct_chain.chain.clone();
    }
}

#[cfg(test)]
mod tests {
    use crate::block::tests::generate_blocks;
    use crate::blockchain::tests::generate_blockchain;
	use crate::node::Node;

    #[test]
    fn test_conflict(){
        let chain = generate_blockchain();

        assert!(chain.is_valid());

        let mut invalid_chain = generate_blockchain();

        invalid_chain.chain.append(&mut generate_blocks());

        assert!(!invalid_chain.is_valid());

        let mut node = Node{
            blockchain: invalid_chain,
            last_time_synced: 0.0,
        };

        node.resolve_chain_conflict(&chain);

        assert!(node.blockchain == chain);

    }
}

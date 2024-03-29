use crate::block::{calculate_hash, hash_to_binary_representation, Block, DIFFICULTY_PREFIX};
use chrono::Utc;

use felipeum_transaction_pool::pool::{Pool, PoolError, PoolTransaction};
use log::{error, warn};

#[derive(Debug)]
pub struct Chain {
    pub blocks: Vec<Block>,
    pub pool: Pool,
}

impl Chain {
    pub fn new(pool: Pool) -> Self {
        Self {
            blocks: vec![],
            pool,
        }
    }

    pub fn genesis(&mut self) {
        let genesis_block = Block {
            id: 0,
            hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
            previous_hash: String::from("genesis"),
            timestamp: Utc::now().timestamp(),
            data: String::from("genesis"),
            nonce: 2836,
        };

        self.blocks.push(genesis_block);
    }

    fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
        if block.previous_hash != previous_block.hash {
            warn!("block with id: {} has wrong previous hash", block.id);
            return false;
        } else if !hash_to_binary_representation(
            &hex::decode(&block.hash).expect("can decode from hex"),
        )
        .starts_with(DIFFICULTY_PREFIX)
        {
            warn!("block with id: {} has invalid difficulty", block.id);
            return false;
        } else if block.id != previous_block.id + 1 {
            warn!(
                "block with id: {} is not the next block after the latest: {}",
                block.id, previous_block.id
            );
            return false;
        } else if hex::encode(calculate_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data,
            block.nonce,
        )) != block.hash
        {
            warn!("block with id: {} has invalid hash", block.id);
            return false;
        }

        true
    }

    fn is_chain_valid(&self, chain: &[Block]) -> bool {
        for i in 0..chain.len() {
            if i == 0 {
                continue;
            }
            let first = chain.get(i - 1).expect("has to exist");
            let second = chain.get(i).expect("has to exist");
            if !self.is_block_valid(second, first) {
                return false;
            }
        }
        true
    }

    pub fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                local
            } else {
                remote
            }
        } else if is_remote_valid && !is_local_valid {
            remote
        } else if !is_remote_valid && is_remote_valid {
            local
        } else {
            panic!("local and remote chains are both invalid");
        }
    }

    pub fn add_new_pool_transaction(
        &self,
        tx: PoolTransaction,
    ) -> Result<PoolTransaction, PoolError> {
        // TODO: here we are checking before insert to avoid a loop betweeen peers,
        // ideally we don't want to broadcast to all nodes but for now it's eaier this way
        match self.pool.get(tx.transaction_id.clone()) {
            Some(_) => Err(PoolError::DiscardedOnInsert("already in pool".to_string())),
            None => self.pool.add_transaction(tx),
        }
    }

    pub fn try_add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("there is at least one block");
        if self.is_block_valid(&block, latest_block) {
            self.blocks.push(block);
        } else {
            error!("could not add block - invalid");
        }
    }
}

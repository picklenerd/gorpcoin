pub mod block;
pub mod block_data;

pub use block::GorpcoinBlock;
pub use block_data::GorpcoinBlockData;

use crate::error::{GorpcoinError, GorpcoinResult};
use crate::{utils, Transaction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GorpcoinBlockchain {
    blocks: Vec<GorpcoinBlock>,
}

impl GorpcoinBlockchain {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn blocks(&self) -> &[GorpcoinBlock] {
        &self.blocks
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn last_hash(&self) -> Vec<u8> {
        self.blocks
            .last()
            .map(|block| block.hash())
            .unwrap_or_else(|| vec![0])
    }

    pub fn current_difficulty(&self) -> u8 {
        difficulty_function(self.len())
    }

    pub fn add_block(&mut self, block: GorpcoinBlock) -> GorpcoinResult<()> {
        let hash = block.hash();
        let difficulty = self.current_difficulty();

        if !utils::has_valid_prefix(&hash, difficulty) {
            return Err(GorpcoinError::IncorrectDifficulty);
        }

        let expected_previous_hash = match self.blocks.last() {
            Some(previous_block) => previous_block.hash(),
            None => vec![0],
        };

        if expected_previous_hash != block.previous_hash() {
            return Err(GorpcoinError::InvalidPreviousHash);
        }

        self.blocks.push(block);

        Ok(())
    }

    pub fn is_transaction_valid(&self, transaction: &Transaction) -> bool {        
        let mut input_total = 0;

        for block in &self.blocks {
            for input in transaction.inputs() {
                if let Some(transaction_data) = block.data().transactions().get(input) {
                    // This is wrong
                    // We need signatures. I think it'll be more work to avoid it.
                    input_total += transaction_data.output_total();
                }
            }
        }

        input_total >= transaction.output_total()
    }
}

fn difficulty_function(length: usize) -> u8 {
    if length == 0 {
        return 1;
    }

    let length = length as u32;
    let length = f64::from(length + 1);
    let difficulty = length.log10() + 1.0;

    difficulty as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_function() {
        assert_eq!(difficulty_function(0), 1);
        assert_eq!(difficulty_function(10), 2);
        assert_eq!(difficulty_function(50), 2);
        assert_eq!(difficulty_function(100), 3);
    }
}

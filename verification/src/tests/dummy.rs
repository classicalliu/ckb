use bigint::{H256, U256};
use chain::chain::{ChainProvider, TipHeader};
use chain::consensus::Consensus;
use chain::error::Error;
use core::block::IndexedBlock;
use core::cell::{CellProvider, CellState};
use core::extras::BlockExt;
use core::header::{BlockNumber, IndexedHeader};
use core::transaction::{
    Capacity, CellOutput, IndexedTransaction, OutPoint, ProposalShortId, Transaction,
};
use core::transaction_meta::TransactionMeta;
use core::uncle::UncleBlock;
use std::collections::HashMap;
use tests::util::RwLock;

pub struct DummyChainClient {
    pub transaction_fees: HashMap<H256, Result<Capacity, Error>>,
    pub block_reward: Capacity,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DummyCellState {
    Head(CellOutput),
    Tail,
    Unknown,
}

impl CellState for DummyCellState {
    fn tail() -> Self {
        DummyCellState::Tail
    }

    fn unknown() -> Self {
        DummyCellState::Unknown
    }

    fn head(&self) -> Option<&CellOutput> {
        match *self {
            DummyCellState::Head(ref output) => Some(output),
            _ => None,
        }
    }

    fn take_head(self) -> Option<CellOutput> {
        match self {
            DummyCellState::Head(output) => Some(output),
            _ => None,
        }
    }

    fn is_head(&self) -> bool {
        match *self {
            DummyCellState::Head(_) => true,
            _ => false,
        }
    }
    fn is_unknown(&self) -> bool {
        match *self {
            DummyCellState::Unknown => true,
            _ => false,
        }
    }
    fn is_tail(&self) -> bool {
        match *self {
            DummyCellState::Tail => true,
            _ => false,
        }
    }
}

impl ChainProvider for DummyChainClient {
    fn block_reward(&self, _block_number: BlockNumber) -> Capacity {
        self.block_reward
    }

    fn calculate_transaction_fee(&self, transaction: &Transaction) -> Result<Capacity, Error> {
        self.transaction_fees[&transaction.hash()].clone()
    }

    fn process_block(&self, _b: &IndexedBlock) -> Result<(), Error> {
        panic!("Not implemented!");
    }

    fn block_header(&self, _hash: &H256) -> Option<IndexedHeader> {
        panic!("Not implemented!");
    }

    fn block_ext(&self, _hash: &H256) -> Option<BlockExt> {
        panic!("Not implemented!");
    }

    fn genesis_hash(&self) -> H256 {
        panic!("Not implemented!");
    }

    fn consensus(&self) -> &Consensus {
        panic!("Not implemented!");
    }

    fn calculate_difficulty(&self, _last: &IndexedHeader) -> Option<U256> {
        panic!("Not implemented!");
    }

    fn get_ancestor(&self, _base: &H256, _number: BlockNumber) -> Option<IndexedHeader> {
        panic!("Not implemented!");
    }

    fn block_body(&self, _hash: &H256) -> Option<Vec<IndexedTransaction>> {
        panic!("Not implemented!");
    }

    fn block_proposal_txs_ids(&self, _hash: &H256) -> Option<Vec<ProposalShortId>> {
        panic!("Not implemented!");
    }

    fn block_hash(&self, _height: u64) -> Option<H256> {
        panic!("Not implemented!");
    }

    fn output_root(&self, _hash: &H256) -> Option<H256> {
        panic!("Not implemented!");
    }

    fn block_number(&self, _hash: &H256) -> Option<BlockNumber> {
        panic!("Not implemented!");
    }

    fn block(&self, _hash: &H256) -> Option<IndexedBlock> {
        panic!("Not implemented!");
    }

    fn get_tip_uncles(&self) -> Vec<UncleBlock> {
        panic!("Not implemented!");
    }

    fn tip_header(&self) -> &RwLock<TipHeader> {
        panic!("Not implemented!");
    }

    fn get_transaction(&self, _hash: &H256) -> Option<IndexedTransaction> {
        panic!("Not implemented!");
    }

    fn contain_transaction(&self, _hash: &H256) -> bool {
        panic!("Not implemented!");
    }

    fn get_transaction_meta(&self, _hash: &H256) -> Option<TransactionMeta> {
        panic!("Not implemented!");
    }

    fn get_transaction_meta_at(&self, _hash: &H256, _parent: &H256) -> Option<TransactionMeta> {
        panic!("Not implemented!");
    }
}

impl CellProvider for DummyChainClient {
    type State = DummyCellState;

    fn cell(&self, _o: &OutPoint) -> DummyCellState {
        panic!("Not implemented!");
    }

    fn cell_at(&self, _out_point: &OutPoint, _parent: &H256) -> DummyCellState {
        panic!("Not implemented!");
    }
}
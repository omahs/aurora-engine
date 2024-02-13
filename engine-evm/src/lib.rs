#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code, unused_variables)]

extern crate alloc;

use aurora_engine_precompiles::Precompiles;
use aurora_engine_sdk::env::Env;
use aurora_engine_sdk::io::IO;
use aurora_engine_sdk::promise::PromiseHandler;
use aurora_engine_types::account_id::AccountId;
use aurora_engine_types::parameters::engine::{SubmitResult, TransactionStatus};
use aurora_engine_types::types::Wei;
use aurora_engine_types::Box;
use aurora_engine_types::Vec;
use aurora_engine_types::{H160, H256, U256};

#[cfg(feature = "evm-revm")]
mod revm;
#[cfg(feature = "evm-sputnikvm")]
mod sputnikvm;

pub use crate::sputnikvm::errors::{TransactErrorKind, TransactExecutionResult};

#[cfg(feature = "evm-revm")]
/// Init REVM
pub fn init_evm<'tx, 'env, I: IO + Copy, E: Env, H: PromiseHandler>(
    io: I,
    env: &'env E,
    transaction: &'env TransactionInfo,
    block: &'env BlockInfo,
    _precompiles: Precompiles<'env, I, E, H::ReadOnly>,
) -> EngineEVM<'env, I, E, revm::REVMHandler<'env, I, E>> {
    let handler = revm::REVMHandler::new(io, env, transaction, block);
    EngineEVM::new(io, env, transaction, block, handler)
}

#[cfg(feature = "evm-sputnikvm")]
/// Init SputnikVM
pub fn init_evm<'env, I: IO + Copy, E: Env, H: PromiseHandler>(
    io: I,
    env: &'env E,
    transaction: &'env TransactionInfo,
    block: &'env BlockInfo,
    precompiles: Precompiles<'env, I, E, H::ReadOnly>,
    remove_eth_fn: Option<Box<dyn FnOnce(Wei) + 'env>>,
) -> EngineEVM<sputnikvm::SputnikVMHandler<'env, I, E, H>> {
    let handler =
        sputnikvm::SputnikVMHandler::new(io, env, transaction, block, precompiles, remove_eth_fn);
    EngineEVM::new(handler)
}

#[cfg(feature = "integration-test")]
pub use sputnikvm::ApplyModify;

#[cfg(feature = "integration-test")]
pub fn apply<I: IO + Copy, E: Env>(io: I, env: &E, state_change: sputnikvm::ApplyModify) {
    use evm::backend::ApplyBackend;
    let tx = TransactionInfo::default();
    let block = BlockInfo::default();
    let mut contract_state = sputnikvm::ContractState::new(io, env, &tx, &block, None);
    let state_change = evm::backend::Apply::Modify {
        address: state_change.address,
        basic: evm::backend::Basic {
            balance: state_change.basic_balance,
            nonce: state_change.basic_nonce,
        },
        code: None,
        storage: core::iter::empty(),
        reset_storage: false,
    };
    contract_state.apply(core::iter::once(state_change), core::iter::empty(), false);
}

pub struct TransactResult {
    pub submit_result: SubmitResult,
    pub logs: Vec<Log>,
}

pub trait EVMHandler {
    fn transact_create(&mut self) -> TransactExecutionResult<TransactResult>;
    fn transact_call(&mut self) -> TransactExecutionResult<TransactResult>;
    fn view(&mut self) -> TransactExecutionResult<TransactionStatus>;
}

#[derive(Default, Debug, Clone)]
pub struct TransactionInfo {
    pub origin: H160,
    pub value: Wei,
    pub input: Vec<u8>,
    pub address: Option<H160>,
    pub gas_limit: u64,
    pub access_list: Vec<(H160, Vec<H256>)>,
}

#[derive(Default, Debug, Clone)]
pub struct BlockInfo {
    pub gas_price: U256,
    pub current_account_id: AccountId,
    pub chain_id: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Log {
    pub address: H160,
    pub topics: Vec<H256>,
    pub data: Vec<u8>,
}

pub struct EngineEVM<H: EVMHandler> {
    handler: H,
}

impl<H: EVMHandler> EngineEVM<H> {
    /// Initialize Engine EVM.
    /// Where `handler` initialized from the feature flag.
    pub fn new(handler: H) -> Self {
        Self { handler }
    }
}

impl<H: EVMHandler> EVMHandler for EngineEVM<H> {
    /// Invoke EVM transact-create
    fn transact_create(&mut self) -> TransactExecutionResult<TransactResult> {
        self.handler.transact_create()
    }

    /// Invoke EVM transact-call
    fn transact_call(&mut self) -> TransactExecutionResult<TransactResult> {
        self.handler.transact_call()
    }

    /// View call
    fn view(&mut self) -> TransactExecutionResult<TransactionStatus> {
        self.handler.view()
    }
}

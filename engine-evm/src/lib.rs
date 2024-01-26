#![cfg_attr(not(any(feature = "std", feature = "contracts-std")), no_std)]
#![allow(dead_code, unused_variables)]
extern crate alloc;

use crate::revm::REVMHandler;
use aurora_engine_sdk::env::Env;
use aurora_engine_sdk::io::IO;
use aurora_engine_types::types::{Address, Wei};
use aurora_engine_types::{Box, Vec};
use aurora_engine_types::{H160, H256, U256};

#[cfg(feature = "revm")]
mod revm;

pub trait EVMHandler {
    fn transact_create(&mut self);
    fn transact_create_fixed(&mut self);
    fn transact_call(&mut self);
}

#[derive(Clone, Debug)]
pub struct TransactionInfo {
    pub gas_price: U256,
    pub origin: Address,
    pub value: Wei,
    pub input: Vec<u8>,
    pub address: Option<Address>,
    pub gas_limit: u64,
    pub access_list: Vec<(H160, Vec<H256>)>,
}

pub struct EngineEVM<'tx, 'env, I: IO, E: Env, H: EVMHandler> {
    io: I,
    env: &'env E,
    handler: H,
    transaction: &'tx TransactionInfo,
}

pub fn init_evm<'tx, 'env, I: IO + Copy, E: Env>(
    io: &I,
    env: &'env E,
    transaction: &'tx TransactionInfo,
) -> EngineEVM<'tx, 'env, I, E, REVMHandler<'env, I, E>> {
    let handler = REVMHandler::new(io, env, &transaction);
    EngineEVM::new(io, env, transaction, handler)
}

impl<'tx, 'env, I: IO + Copy, E: Env, H: EVMHandler> EngineEVM<'tx, 'env, I, E, H> {
    /// Initialize Engine EVM.
    /// Where `handler` initialized from the feature flag.
    pub fn new(io: &I, env: &'env E, transaction: &'tx TransactionInfo, handler: H) -> Self {
        // #[cfg(feature = "revm")]
        // let handler = REVMHandler::new(io, env.clone(), &transaction);
        Self {
            io: *io,
            env,
            handler,
            transaction,
        }
    }

    /// Invoke EVM transact-create
    pub fn transact_create(&mut self) {
        self.handler.transact_create();
    }

    /// Invoke EVM transact-create-fixed
    pub fn transact_create_fixed(&mut self) {
        self.handler.transact_create_fixed();
    }

    /// Invoke EVM transact-call
    pub fn transact_call(&mut self) {
        self.handler.transact_call();
    }
}

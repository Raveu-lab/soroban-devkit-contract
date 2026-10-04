//! Event-rich test fixture contract.
//!
//! Emits events covering every XDR ScVal type so EventDecoder in
//! soroban-devkit-core can be tested against real on-chain data.
//! Contains no business logic.
//!
//! Uses `Events::publish` directly (no separate events.rs — this contract's
//! whole purpose is exercising every event shape). `Events::publish` is
//! deprecated in favor of the `#[contractevent]` macro; migrating would
//! change the emitted topic/data shape, so it's deferred.
#![no_std]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contractimpl, Address, Bytes, Duration, Env, Error, Map, String, Symbol, Timepoint,
    Vec, I256, U256,
};

#[contract]
pub struct EventRichContract;

#[contractimpl]
impl EventRichContract {
    /// Emit events using primitive types: u32, i32, u64, i64, bool
    pub fn emit_primitive_types(env: Env) {
        env.events()
            .publish((Symbol::new(&env, "u32_event"),), 42u32);
        env.events()
            .publish((Symbol::new(&env, "i32_event"),), -42i32);
        env.events()
            .publish((Symbol::new(&env, "u64_event"),), 1_000_000u64);
        env.events()
            .publish((Symbol::new(&env, "i64_event"),), -1_000_000i64);
        env.events()
            .publish((Symbol::new(&env, "bool_true"),), true);
        env.events()
            .publish((Symbol::new(&env, "bool_false"),), false);
    }

    /// Emit events using 128-bit integers
    pub fn emit_big_numbers(env: Env) {
        env.events()
            .publish((Symbol::new(&env, "u128_event"),), u128::MAX);
        env.events()
            .publish((Symbol::new(&env, "i128_event"),), i128::MIN);
    }

    /// Emit events using string types: Symbol, String, Bytes
    pub fn emit_strings(env: Env) {
        env.events().publish(
            (Symbol::new(&env, "symbol_event"),),
            Symbol::new(&env, "hello"),
        );
        env.events().publish(
            (Symbol::new(&env, "string_event"),),
            String::from_str(&env, "hello from soroban"),
        );
        env.events().publish(
            (Symbol::new(&env, "bytes_event"),),
            Bytes::from_slice(&env, &[0xde, 0xad, 0xbe, 0xef]),
        );
    }

    /// Emit events using Vec and Map collection types
    pub fn emit_collections(env: Env) {
        let mut vec: Vec<u32> = Vec::new(&env);
        vec.push_back(1);
        vec.push_back(2);
        vec.push_back(3);
        env.events().publish((Symbol::new(&env, "vec_event"),), vec);

        let mut map: Map<Symbol, i128> = Map::new(&env);
        map.set(Symbol::new(&env, "amount"), 1_000_000);
        map.set(Symbol::new(&env, "fee"), 100);
        env.events().publish((Symbol::new(&env, "map_event"),), map);
    }

    /// Emit an event containing an Address
    pub fn emit_address(env: Env, addr: Address) {
        env.events()
            .publish((Symbol::new(&env, "address_event"), addr.clone()), addr);
    }

    /// Emit events using 256-bit integers
    pub fn emit_wider_numbers(env: Env) {
        env.events().publish(
            (Symbol::new(&env, "u256_event"),),
            U256::from_u32(&env, u32::MAX).add(&U256::from_u32(&env, 1)),
        );
        env.events().publish(
            (Symbol::new(&env, "i256_event"),),
            I256::from_i32(&env, i32::MIN).sub(&I256::from_i32(&env, 1)),
        );
    }

    /// Emit events using Timepoint and Duration
    pub fn emit_time_types(env: Env) {
        env.events().publish(
            (Symbol::new(&env, "timepoint_event"),),
            Timepoint::from_unix(&env, 1_700_000_000),
        );
        env.events().publish(
            (Symbol::new(&env, "duration_event"),),
            Duration::from_seconds(&env, 3600),
        );
    }

    /// Emit an event containing a contract Error value — the same shape a
    /// real diagnostic event's topic carries for any failed invocation
    /// (confirmed live: calling a non-existent method on a deployed
    /// contract produces a diagnostic event whose topic is exactly this
    /// ScVal type).
    pub fn emit_error(env: Env) {
        env.events().publish(
            (Symbol::new(&env, "error_event"),),
            Error::from_contract_error(6),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_emit_all_types() {
        let env = Env::default();
        let contract_id = env.register(EventRichContract, ());
        let client = EventRichContractClient::new(&env, &contract_id);

        // These should all succeed without panicking
        client.emit_primitive_types();
        client.emit_big_numbers();
        client.emit_strings();
        client.emit_collections();

        let addr = Address::generate(&env);
        client.emit_address(&addr);
    }

    #[test]
    fn test_emit_wider_numbers() {
        // This contract's whole purpose is exercising every XDR ScVal type
        // soroban-devkit-core's EventDecoder can decode — it was missing
        // scvU256/scvI256 even after the decoder gained support for them.
        let env = Env::default();
        let contract_id = env.register(EventRichContract, ());
        let client = EventRichContractClient::new(&env, &contract_id);
        client.emit_wider_numbers();
    }

    #[test]
    fn test_emit_time_types() {
        // Same gap as above, for scvTimepoint/scvDuration.
        let env = Env::default();
        let contract_id = env.register(EventRichContract, ());
        let client = EventRichContractClient::new(&env, &contract_id);
        client.emit_time_types();
    }

    #[test]
    fn test_emit_error() {
        // Same gap as above, for scvError — soroban-devkit-core's decoder
        // only just gained support for it.
        let env = Env::default();
        let contract_id = env.register(EventRichContract, ());
        let client = EventRichContractClient::new(&env, &contract_id);
        client.emit_error();
    }
}

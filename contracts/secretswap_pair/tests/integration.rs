//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::{from_binary, Coin, HandleResponse, HumanAddr, InitResponse};
use cosmwasm_vm::testing::{
    handle, init, mock_dependencies, mock_env, query, MockApi, MockQuerier, MockStorage,
    MOCK_CONTRACT_ADDR,
};
use cosmwasm_vm::Instance;

use secretswap::{AssetInfo, PairInfo, PairInitMsg};
use secretswap_pair::msg::{HandleMsg, QueryMsg};

// This line will test the output of cargo wasm
static WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/secretswap_pair.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../contract.wasm");

const DEFAULT_GAS_LIMIT: u64 = 500_000;

pub fn mock_instance(
    wasm: &[u8],
    contract_balance: &[Coin],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    // TODO: check_wasm is not exported from cosmwasm_vm
    // let terra_features = features_from_csv("staking,terra");
    // check_wasm(wasm, &terra_features).unwrap();
    let deps = mock_dependencies(20, contract_balance);
    Instance::from_code(wasm, deps, DEFAULT_GAS_LIMIT).unwrap()
}

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = PairInitMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: HumanAddr::from("asset0000"),
            },
        ],
        token_code_id: 10u64,
        init_hook: None,
    };

    let env = mock_env("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res: InitResponse = init(&mut deps, env, msg).unwrap();

    // cannot change it after post intialization
    let msg = HandleMsg::PostInitialize {};
    let env = mock_env("liquidity0000", &[]);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // it worked, let's query the state
    let res = query(&mut deps, QueryMsg::Pair {}).unwrap();
    let pair_info: PairInfo = from_binary(&res).unwrap();
    assert_eq!(MOCK_CONTRACT_ADDR, pair_info.contract_addr.as_str());
    assert_eq!(
        [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: HumanAddr::from("asset0000"),
            },
        ],
        pair_info.asset_infos
    );

    assert_eq!("liquidity0000", pair_info.liquidity_token.as_str());
}

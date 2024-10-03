use multiversx_sc_scenario::imports::*;

use rust_challenge::*;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const OTHER_ADDRESS1: TestAddress = TestAddress::new("address1");
const OTHER_ADDRESS2: TestAddress = TestAddress::new("address2");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");
const CONTRACT_ADDRESS: TestSCAddress = TestSCAddress::new("rust-challenge");
const CODE_PATH: MxscPath = MxscPath::new("output/rust_challenge.mxsc.json");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CODE_PATH, rust_challenge::ContractBuilder);

    blockchain.account(OWNER_ADDRESS).balance(4).nonce(1);
    blockchain.account(OTHER_ADDRESS1).balance(5).nonce(1);
    blockchain.account(OTHER_ADDRESS2).balance(6).nonce(1);
    blockchain.account(RECEIVER_ADDRESS).nonce(1);

    blockchain
}

fn deploy() -> ScenarioWorld {
    let mut world = world();

    let contract_address = world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .init()
        .code(CODE_PATH)
        .new_address(CONTRACT_ADDRESS)
        .returns(ReturnsNewAddress)
        .run();

    assert_eq!(contract_address, CONTRACT_ADDRESS.to_address());

    world
}

fn deposit_fail_required() {
    let mut world = deploy();

    world
        .tx()
        .from(OWNER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .set_fee(1u16)
        .run();

    world
        .tx()
        .from(OWNER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .deposit(RECEIVER_ADDRESS)
        .egld(0)
        .with_result(ExpectError(4, "Payments must be greater than fee"))
        .run();
}

fn fail_set_fee_required() {
    let mut world = world();

    world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .init()
        .code(CODE_PATH)
        .new_address(CONTRACT_ADDRESS)
        .run();

    world
        .tx()
        .from(RECEIVER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .set_fee(1u16)
        .with_result(ExpectError(4, "Endpoint can only be called by owner"))
        .run();
}

fn deposit() {
    let mut world = deploy();

    world
        .tx()
        .from(OWNER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .set_fee(1u16)
        .run();

    world
        .tx()
        .from(OTHER_ADDRESS1)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .deposit(RECEIVER_ADDRESS)
        .egld(3)
        .run();

    world
        .tx()
        .from(OTHER_ADDRESS2)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .deposit(RECEIVER_ADDRESS)
        .egld(4)
        .run();

    world
        .tx()
        .from(OWNER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .deposit(RECEIVER_ADDRESS)
        .egld(2)
        .run();

    let value = world
        .query()
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .collected_fees()
        .returns(ReturnsResultUnmanaged)
        .returns(ExpectValue(3u32))
        .run();

    let deposit = world
        .query()
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .reserve_for_address(RECEIVER_ADDRESS)
        .returns(ReturnsResultUnmanaged)
        .returns(ExpectValue(6u32))
        .run();

    println!(">>> {}", value.0);
    println!(">>> {}", deposit.0);
}

#[test]
fn test_deploy() {
    deploy();
    deposit_fail_required();
    fail_set_fee_required();

    deposit();
}

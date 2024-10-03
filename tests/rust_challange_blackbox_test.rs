use multiversx_sc_scenario::imports::*;

use rust_challenge::*;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const ADDRESS1: TestAddress = TestAddress::new("address1");
const ADDRESS2: TestAddress = TestAddress::new("address2");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");
const CONTRACT_ADDRESS: TestSCAddress = TestSCAddress::new("rust-challenge");
const CODE_PATH: MxscPath = MxscPath::new("output/rust_challenge.mxsc.json");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CODE_PATH, rust_challenge::ContractBuilder);

    blockchain
}

struct RustChallengeTest {
    world: ScenarioWorld,
}

impl RustChallengeTest {
    fn new() -> Self {
        let mut world = world();
        world.account(OWNER_ADDRESS).balance(4).nonce(1);
        world.account(ADDRESS1).balance(5).nonce(1);
        world.account(ADDRESS2).balance(6).nonce(1);
        world.account(RECEIVER_ADDRESS).nonce(1);

        Self { world }
    }

    fn deploy(&mut self, fee: u32) -> Address {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .init(fee)
            .code(CODE_PATH)
            .new_address(CONTRACT_ADDRESS)
            .returns(ReturnsNewAddress)
            .run()
    }

    fn deposit_fail_required(&mut self, from_address: TestAddress, wrong_deposit: u64) {
        self.world
            .tx()
            .from(from_address)
            .to(CONTRACT_ADDRESS)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .deposit(RECEIVER_ADDRESS)
            .egld(wrong_deposit)
            .with_result(ExpectError(4, "Payments must be greater than fee"))
            .run();
    }

    fn deposit(&mut self, sender: TestAddress, receiver: TestAddress, value: u64) {
        self.world
            .tx()
            .from(sender)
            .to(CONTRACT_ADDRESS)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .deposit(receiver)
            .egld(value)
            .run();
    }

    fn withdraw(&mut self, sender: TestAddress) {
        self.world
            .tx()
            .from(sender)
            .to(CONTRACT_ADDRESS)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .withdraw()
            .run();
    }

    fn query_collected_fees(&mut self) -> RustBigUint {
        self.world
            .query()
            .to(CONTRACT_ADDRESS)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .get_collected_fees()
            .returns(ReturnsResultUnmanaged)
            .run()
    }

    fn query_reserve_for_address(&mut self, address: TestAddress) -> RustBigUint {
        self.world
            .query()
            .to(CONTRACT_ADDRESS)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .get_reserve_for_address(address)
            .returns(ReturnsResultUnmanaged)
            .run()
    }

    fn check_account(&mut self, address: TestAddress, balance: u64) {
        self.world.check_account(address).balance(balance);
    }

}

#[test]
fn test_deploy() {
    let mut state = RustChallengeTest::new();
    let address = state.deploy(1u32);
    assert_eq!(address, CONTRACT_ADDRESS);
}

#[test]
fn test_deposit_fail() {
    let mut state = RustChallengeTest::new();
    state.deploy(1u32);

    state.deposit_fail_required(ADDRESS1, 0u64);
}

#[test]
fn test_deposit() {
    let mut state = RustChallengeTest::new();
    state.deploy(1u32);

    state.deposit(ADDRESS1, RECEIVER_ADDRESS, 3);
    state.deposit(ADDRESS2, RECEIVER_ADDRESS, 4);
    state.deposit(OWNER_ADDRESS, ADDRESS1, 2);

    assert_eq!(RustBigUint::from(3u32), state.query_collected_fees());
    assert_eq!(
        RustBigUint::from(5u32),
        state.query_reserve_for_address(RECEIVER_ADDRESS)
    );
    assert_eq!(
        RustBigUint::from(1u32),
        state.query_reserve_for_address(ADDRESS1)
    );
    assert_eq!(RustBigUint::ZERO, state.query_reserve_for_address(ADDRESS2));
    state.check_account(OWNER_ADDRESS, 2);
    state.check_account(ADDRESS1, 2);
    state.check_account(ADDRESS2, 2);
    state.check_account(RECEIVER_ADDRESS, 0);
}

#[test]
fn test_withdraw_address1() {
    let mut state = RustChallengeTest::new();
    state.deploy(1u32);

    state.deposit(ADDRESS1, RECEIVER_ADDRESS, 3);
    state.deposit(ADDRESS2, RECEIVER_ADDRESS, 4);
    state.deposit(OWNER_ADDRESS, ADDRESS1, 2);

    state.check_account(ADDRESS1, 2);
    assert_eq!(
        RustBigUint::from(1u32),
        state.query_reserve_for_address(ADDRESS1)
    );
    state.withdraw(ADDRESS1);
    state.check_account(ADDRESS1, 3);
    assert_eq!(RustBigUint::from(3u32), state.query_collected_fees());
    assert_eq!(RustBigUint::ZERO, state.query_reserve_for_address(ADDRESS1));
}

#[test]
fn test_withdraw_address2() {
    let mut state = RustChallengeTest::new();
    state.deploy(1u32);

    state.deposit(ADDRESS1, RECEIVER_ADDRESS, 3);
    state.deposit(ADDRESS2, RECEIVER_ADDRESS, 4);
    state.deposit(OWNER_ADDRESS, ADDRESS1, 2);

    state.check_account(ADDRESS2, 2);
    assert_eq!(
        RustBigUint::from(1u32),
        state.query_reserve_for_address(ADDRESS1)
    );
    state
        .world
        .tx()
        .from(ADDRESS2)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .withdraw()
        .with_result(ExpectError(4, "Nothing to claim"))
        .run();
    state.check_account(ADDRESS2, 2);
    assert_eq!(RustBigUint::ZERO, state.query_reserve_for_address(ADDRESS2));
    assert_eq!(RustBigUint::from(3u32), state.query_collected_fees());
}

#[test]
fn test_withdraw_receiver() {
    let mut state = RustChallengeTest::new();
    state.deploy(1u32);

    state.deposit(ADDRESS1, RECEIVER_ADDRESS, 3);
    state.deposit(ADDRESS2, RECEIVER_ADDRESS, 4);
    state.deposit(OWNER_ADDRESS, ADDRESS1, 2);

    state.check_account(RECEIVER_ADDRESS, 0);
    assert_eq!(
        RustBigUint::from(5u32),
        state.query_reserve_for_address(RECEIVER_ADDRESS)
    );
    state.withdraw(RECEIVER_ADDRESS);
    state.check_account(RECEIVER_ADDRESS, 5);
    assert_eq!(RustBigUint::ZERO, state.query_reserve_for_address(ADDRESS2));
    assert_eq!(RustBigUint::from(3u32), state.query_collected_fees());
}

#[test]
fn test_withdraw_owner() {
    let mut state = RustChallengeTest::new();
    state.deploy(1u32);

    state.deposit(ADDRESS1, RECEIVER_ADDRESS, 3);
    state.deposit(ADDRESS2, RECEIVER_ADDRESS, 4);
    state.deposit(OWNER_ADDRESS, ADDRESS1, 2);

    state.check_account(OWNER_ADDRESS, 2);
    assert_eq!(
        RustBigUint::ZERO,
        state.query_reserve_for_address(OWNER_ADDRESS)
    );
    state.withdraw(OWNER_ADDRESS);
    state.check_account(OWNER_ADDRESS, 5);
    assert_eq!(RustBigUint::ZERO, state.query_reserve_for_address(ADDRESS2));
    assert_eq!(RustBigUint::ZERO, state.query_collected_fees());
}

#[test]
fn test_fail_set_fee() {
    let mut state = RustChallengeTest::new();
    state.deploy(1u32);
    state
        .world
        .tx()
        .from(ADDRESS1)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .set_fee(2u32)
        .with_result(ExpectError(4, "Endpoint can only be called by owner"))
        .run();
}

#[test]
fn test_set_fee() {
    let mut state = RustChallengeTest::new();
    state.deploy(1u32);

    state.deposit(ADDRESS1, RECEIVER_ADDRESS, 3);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .set_fee(2u32)
        .run();

    state.deposit(ADDRESS2, RECEIVER_ADDRESS, 4);
    state.deposit(OWNER_ADDRESS, ADDRESS1, 3);

    assert_eq!(RustBigUint::from(5u32), state.query_collected_fees());
    assert_eq!(
        RustBigUint::from(4u32),
        state.query_reserve_for_address(RECEIVER_ADDRESS)
    );
    assert_eq!(
        RustBigUint::from(1u32),
        state.query_reserve_for_address(ADDRESS1)
    );
    assert_eq!(RustBigUint::ZERO, state.query_reserve_for_address(ADDRESS2));
    state.check_account(OWNER_ADDRESS, 1);
    state.check_account(ADDRESS1, 2);
    state.check_account(ADDRESS2, 2);
    state.check_account(RECEIVER_ADDRESS, 0);
}

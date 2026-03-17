use basic_interact::{Config, RustChallengeInteractor};
use multiversx_sc_snippets::{
    imports::{Bech32Address, RustBigUint},
    test_wallets,
};

struct Params {
    receiver: Bech32Address,
    address1: Bech32Address,
    address2: Bech32Address,
}

#[tokio::test]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn chain_simulator_tests() {
    let mut interact = RustChallengeInteractor::init(Config::chain_simulator_config()).await;
    let receiver = Bech32Address::from(test_wallets::alice().to_address());
    let address1 = Bech32Address::from(test_wallets::mike().to_address());
    let address2 = Bech32Address::from(test_wallets::heidi().to_address());

    interact
        .interactor
        .register_wallet(test_wallets::alice())
        .await;
    interact
        .interactor
        .register_wallet(test_wallets::mike())
        .await;
    interact
        .interactor
        .register_wallet(test_wallets::heidi())
        .await;

    interact.generate_blocks(2).await;

    let params = Params {
        receiver,
        address1,
        address2,
    };

    deposit_no_fee(&mut interact, &params).await;
    withdraw(&mut interact, &params).await;
    withdraw_with_fee_set_zero(&mut interact, &params).await;
}

async fn deposit_no_fee(interact: &mut RustChallengeInteractor, params: &Params) {
    let Params {
        receiver,
        address1,
        address2,
    } = params;

    // deploy contract with no fee
    interact.deploy(0u64).await;

    interact.deposit(&address1, &receiver, 3u64).await;
    interact.deposit(&address2, &receiver, 4u64).await;
    interact
        .deposit(&interact.owner_address.clone(), &address1, 2u64)
        .await;

    let fee = interact.collected_fees().await;
    assert_eq!(RustBigUint::ZERO, fee);

    let reserve = interact.reserve_for_address(&receiver).await;
    assert!(reserve != RustBigUint::ZERO, "Value should not be zero");
    interact.withdraw(&receiver).await;
}

async fn withdraw(interact: &mut RustChallengeInteractor, params: &Params) {
    let Params {
        receiver,
        address1,
        address2,
    } = params;

    // deploy contract with no fee
    interact.deploy(1u64).await;

    interact.deposit(&address1, &receiver, 3u64).await;
    interact.deposit(&address2, &receiver, 4u64).await;
    interact
        .deposit(&interact.owner_address.clone(), &address1, 2u64)
        .await;

    let reserve = interact.reserve_for_address(&receiver).await;
    assert!(reserve != RustBigUint::ZERO, "Value should not be zero");
    interact.withdraw(&receiver).await;
}

async fn withdraw_with_fee_set_zero(interact: &mut RustChallengeInteractor, params: &Params) {
    let Params {
        receiver,
        address1,
        address2,
    } = params;

    interact.generate_blocks(2).await;

    interact.deploy(1u64).await;
    interact.set_fee(0u64).await;

    interact.deposit(&address1, &receiver, 3u64).await;
    interact.deposit(&address2, &receiver, 4u64).await;
    interact
        .deposit(&interact.owner_address.clone(), &address1, 2u64)
        .await;

    let reserve = interact
        .reserve_for_address(&interact.owner_address.clone())
        .await;
    assert!(reserve != RustBigUint::ZERO, "Value should not be zero");
    interact.withdraw(&interact.owner_address.clone()).await;
}

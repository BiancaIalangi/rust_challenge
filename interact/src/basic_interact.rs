mod basic_interact_cli;
mod basic_interact_config;
mod basic_interact_state;

use basic_interact_config::Config;
use basic_interact_state::State;
use clap::Parser;
use rust_challenge::rust_challenge_proxy;

use multiversx_sc_snippets::imports::*;
use test_wallets::{alice, carol, heidi, mike};

const INTERACTOR_SCENARIO_TRACE_PATH: &str = "interactor_trace.scen.json";

const CODE_PATH: MxscPath = MxscPath::new("../output/rust-challenge.mxsc.json");

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut interact = RustChallengeInteractor::init().await;

    let cli = basic_interact_cli::InteractCli::parse();
    match &cli.command {
        Some(basic_interact_cli::InteractCliCommand::Deploy(args)) => {
            interact.deploy(args.fee.into()).await;
        }
        Some(basic_interact_cli::InteractCliCommand::Deposit(args)) => {
            let sender: Address =
                Bech32Address::from_bech32_string(args.sender.clone()).to_address();
            let receiver: Address =
                Bech32Address::from_bech32_string(args.receiver.clone()).to_address();
            interact.deposit(sender, receiver, args.value.into()).await;
        }
        Some(basic_interact_cli::InteractCliCommand::Withdraw(args)) => {
            let sender: Address =
                Bech32Address::from_bech32_string(args.sender.clone()).to_address();
            interact.withdraw(sender).await;
        }
        Some(basic_interact_cli::InteractCliCommand::SetFee(args)) => {
            interact.set_fee(args.fee.into()).await;
        }
        Some(basic_interact_cli::InteractCliCommand::CollectedFees) => {
            interact.collected_fees().await;
        }
        Some(basic_interact_cli::InteractCliCommand::ReserveForAddress(args)) => {
            let sender: Address =
                Bech32Address::from_bech32_string(args.sender.clone()).to_address();
            interact.reserve_for_address(sender).await;
        }
        None => {}
    }
}

#[allow(unused)]
struct RustChallengeInteractor {
    interactor: Interactor,
    owner_address: Bech32Address,
    state: State,
}

impl RustChallengeInteractor {
    async fn init() -> Self {
        let config = Config::load_config();
        let mut interactor = Interactor::new(config.gateway())
            .await
            .with_tracer(INTERACTOR_SCENARIO_TRACE_PATH)
            .await;

        let owner_address = interactor.register_wallet(
            Wallet::from_pem_file("/home/bibi/Desktop/wallet_test/test_wallet.pem").unwrap(),
        );

        Self {
            interactor,
            owner_address: owner_address.into(),
            state: State::load_state(),
        }
    }

    async fn set_state(&mut self) {
        println!("wallet address for owner: {}", self.owner_address);
        self.interactor.retrieve_account(&self.owner_address).await;
    }

    async fn deploy(&mut self, fee: BigUint<StaticApi>) {
        // warning: multi deploy not yet fully supported
        // only works with last deployed address

        self.set_state().await;

        let new_address = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .gas(8_000_000)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .init(fee)
            .code(CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewBech32Address)
            .prepare_async()
            .run()
            .await;

        println!("new address: {new_address}");
        self.state.set_rust_challenge_address(new_address);
    }

    async fn set_fee(&mut self, fee: BigUint<StaticApi>) {
        self.interactor
            .tx()
            .from(self.owner_address.clone())
            .to(self.state.current_rust_challenge_address())
            .gas(8_000_000)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .set_fee(fee)
            .prepare_async()
            .run()
            .await;
    }

    async fn deposit(&mut self, sender: Address, receiver: Address, value: BigUint<StaticApi>) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.state.current_rust_challenge_address())
            .gas(8_000_000)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .deposit(receiver)
            .egld(value)
            .prepare_async()
            .run()
            .await;
    }

    async fn withdraw(&mut self, sender: Address) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.state.current_rust_challenge_address())
            .gas(8_000_000)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .withdraw()
            .prepare_async()
            .run()
            .await;
    }

    async fn collected_fees(&mut self) -> RustBigUint {
        let fees = self
            .interactor
            .query()
            .to(self.state.current_rust_challenge_address())
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .get_collected_fees()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("fees: {fees}");

        fees
    }

    async fn reserve_for_address(&mut self, receiver: Address) -> RustBigUint {
        let reserve = self
            .interactor
            .query()
            .to(self.state.current_rust_challenge_address())
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .get_reserve_for_address(receiver)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("reserve: {reserve}");

        reserve
    }
}

#[tokio::test]
#[ignore = "run on demand"]
async fn deposit_with_errors() {
    let mut interact = RustChallengeInteractor::init().await;
    let receiver = interact.interactor.register_wallet(alice());
    let address1 = interact.interactor.register_wallet(mike());
    let address2 = interact.interactor.register_wallet(carol());

    // deploy contract with fee 1EGLD
    // interact.deploy(BigUint::from(2u32)).await;

    // should throw an error when the payment is lower or equal to the fee
    // interact
    // .deposit(address1.clone(), receiver.clone(), BigUint::from(1u32))
    // .await;

    // should throw an error when there is no payment.
    interact
        .interactor
        .tx()
        .from(address1.clone())
        .to(interact.state.current_rust_challenge_address())
        .gas(8_000_000)
        .typed(rust_challenge_proxy::RustChallengeProxy)
        .deposit(receiver)
        .prepare_async()
        .run()
        .await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn deposit_no_fee() {
    let mut interact = RustChallengeInteractor::init().await;
    let receiver = interact.interactor.register_wallet(alice());
    let address1 = interact.interactor.register_wallet(mike());
    let address2 = interact.interactor.register_wallet(heidi());

    // deploy contract with no fee
    interact.deploy(BigUint::from(0u32)).await;

    interact
        .deposit(address1.clone(), receiver.clone(), BigUint::from(3u32))
        .await;
    interact
        .deposit(address2.clone(), receiver.clone(), BigUint::from(4u32))
        .await;
    interact
        .deposit(
            interact.owner_address.clone().to_address(),
            address1.clone(),
            BigUint::from(2u32),
        )
        .await;

    let fee = interact.collected_fees().await;
    assert_eq!(RustBigUint::ZERO, fee);

    let reserve = interact.reserve_for_address(receiver.clone()).await;
    assert!(reserve != RustBigUint::ZERO, "Value should not be zero");
    interact.withdraw(receiver).await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn deposit() {
    let mut interact = RustChallengeInteractor::init().await;
    let receiver = interact.interactor.register_wallet(alice());
    let address1 = interact.interactor.register_wallet(mike());
    let address2 = interact.interactor.register_wallet(heidi());

    // interact.deploy(BigUint::from(1u32)).await;

    // interact
    //     .deposit(address1.clone(), receiver.clone(), BigUint::from(3u32))
    //     .await;
    // interact
    //     .deposit(address2.clone(), receiver.clone(), BigUint::from(4u32))
    //     .await;
    interact
        .deposit(
            interact.owner_address.clone().to_address(),
            address1.clone(),
            BigUint::from(2u32),
        )
        .await;

    interact.withdraw(address1).await;

    // should fail when there is nothing to claim for the user
    interact.withdraw(address2).await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn withdraw() {
    let mut interact = RustChallengeInteractor::init().await;
    let receiver = interact.interactor.register_wallet(alice());
    let address1 = interact.interactor.register_wallet(mike());
    let address2 = interact.interactor.register_wallet(heidi());

    interact.deploy(BigUint::from(1u32)).await;

    interact
        .deposit(address1.clone(), receiver.clone(), BigUint::from(3u32))
        .await;
    interact
        .deposit(address2.clone(), receiver.clone(), BigUint::from(4u32))
        .await;
    interact
        .deposit(
            interact.owner_address.clone().to_address(),
            address1.clone(),
            BigUint::from(2u32),
        )
        .await;

    let reserve = interact.reserve_for_address(receiver.clone()).await;
    assert!(reserve != RustBigUint::ZERO, "Value should not be zero");
    interact.withdraw(receiver).await;
}

#[tokio::test]
#[ignore = "run on demand"]
async fn withdraw_with_fee_set_zero() {
    let mut interact = RustChallengeInteractor::init().await;
    let receiver = interact.interactor.register_wallet(alice());
    let address1 = interact.interactor.register_wallet(mike());
    let address2 = interact.interactor.register_wallet(heidi());

    interact.deploy(BigUint::from(1u32)).await;
    interact.set_fee(BigUint::zero()).await;

    interact
        .deposit(address1.clone(), receiver.clone(), BigUint::from(3u32))
        .await;
    interact
        .deposit(address2.clone(), receiver.clone(), BigUint::from(4u32))
        .await;
    interact
        .deposit(
            interact.owner_address.clone().to_address(),
            address1.clone(),
            BigUint::from(2u32),
        )
        .await;

    let reserve = interact
        .reserve_for_address(interact.owner_address.to_address())
        .await;
    assert!(reserve != RustBigUint::ZERO, "Value should not be zero");
    interact.withdraw(interact.owner_address.to_address()).await;
}

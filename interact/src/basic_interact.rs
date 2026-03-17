mod basic_interact_cli;
mod basic_interact_config;
mod basic_interact_state;

pub use basic_interact_config::Config;
use basic_interact_state::State;
use clap::Parser;
use rust_challenge::rust_challenge_proxy;

use multiversx_sc_snippets::imports::*;

const INTERACTOR_SCENARIO_TRACE_PATH: &str = "interactor_trace.scen.json";

const CODE_PATH: MxscPath = MxscPath::new("../output/rust-challenge.mxsc.json");

pub async fn rust_challenge_cli() {
    env_logger::init();

    let config = Config::load_config();

    let mut interact = RustChallengeInteractor::init(config).await;

    let cli = basic_interact_cli::InteractCli::parse();
    match &cli.command {
        Some(basic_interact_cli::InteractCliCommand::Deploy(args)) => {
            interact.deploy(args.fee.into()).await;
        }
        Some(basic_interact_cli::InteractCliCommand::Deposit(args)) => {
            let sender: Bech32Address = Bech32Address::from_bech32_string(args.sender.clone());
            let receiver: Bech32Address = Bech32Address::from_bech32_string(args.receiver.clone());
            interact.deposit(&sender, &receiver, args.value).await;
        }
        Some(basic_interact_cli::InteractCliCommand::Withdraw(args)) => {
            let sender: Bech32Address = Bech32Address::from_bech32_string(args.sender.clone());
            interact.withdraw(&sender).await;
        }
        Some(basic_interact_cli::InteractCliCommand::SetFee(args)) => {
            interact.set_fee(args.fee.into()).await;
        }
        Some(basic_interact_cli::InteractCliCommand::CollectedFees) => {
            interact.collected_fees().await;
        }
        Some(basic_interact_cli::InteractCliCommand::ReserveForAddress(args)) => {
            let sender: Bech32Address = Bech32Address::from_bech32_string(args.sender.clone());
            interact.reserve_for_address(&sender).await;
        }
        None => {}
    }
}

pub struct RustChallengeInteractor {
    pub interactor: Interactor,
    pub owner_address: Bech32Address,
    pub state: State,
}

impl RustChallengeInteractor {
    pub async fn init(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator())
            .with_tracer(INTERACTOR_SCENARIO_TRACE_PATH)
            .await;

        let owner_address = interactor.register_wallet(test_wallets::frank()).await;

        Self {
            interactor,
            owner_address: owner_address.into(),
            state: State::load_state(),
        }
    }

    pub async fn generate_blocks(&self, num_blocks: i32) {
        self.interactor
            .generate_blocks(num_blocks as u64)
            .await
            .unwrap();
    }

    pub async fn set_state(&mut self) {
        println!("wallet address for owner: {}", self.owner_address);
        self.interactor.retrieve_account(&self.owner_address).await;
    }

    pub async fn deploy(&mut self, fee: u64) {
        self.set_state().await;

        let new_address = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .gas(20_000_000)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .init(fee)
            .code(CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewBech32Address)
            .run()
            .await;

        println!("new address: {new_address}");
        self.state.set_rust_challenge_address(new_address);
    }

    pub async fn set_fee(&mut self, fee: u64) {
        self.interactor
            .tx()
            .from(self.owner_address.clone())
            .to(self.state.current_rust_challenge_address())
            .gas(100_000_000)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .set_fee(fee)
            .run()
            .await;
    }

    pub async fn deposit(&mut self, sender: &Bech32Address, receiver: &Bech32Address, value: u64) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.state.current_rust_challenge_address())
            .gas(20_000_000)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .deposit(receiver)
            .egld(value)
            .run()
            .await;
    }

    pub async fn withdraw(&mut self, sender: &Bech32Address) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.state.current_rust_challenge_address())
            .gas(100_000_000)
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .withdraw()
            .run()
            .await;
    }

    pub async fn collected_fees(&mut self) -> RustBigUint {
        let fees = self
            .interactor
            .query()
            .to(self.state.current_rust_challenge_address())
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .get_collected_fees()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("fees: {fees}");

        fees
    }

    pub async fn reserve_for_address(&mut self, receiver: &Bech32Address) -> RustBigUint {
        let reserve = self
            .interactor
            .query()
            .to(self.state.current_rust_challenge_address())
            .typed(rust_challenge_proxy::RustChallengeProxy)
            .get_reserve_for_address(receiver)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("reserve: {reserve}");

        reserve
    }
}

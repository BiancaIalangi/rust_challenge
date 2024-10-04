use clap::{Args, Parser, Subcommand};

/// Adder Interact CLI
#[derive(Default, PartialEq, Eq, Debug, Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct InteractCli {
    #[command(subcommand)]
    pub command: Option<InteractCliCommand>,
}

/// Adder Interact CLI Commands
#[derive(Clone, PartialEq, Eq, Debug, Subcommand)]
pub enum InteractCliCommand {
    #[command(name = "deploy", about = "Deploy contract")]
    Deploy(DeployArgs),
    #[command(name = "deposit", about = "Deposit EGLD")]
    Deposit(DepositArgs),
    #[command(name = "withdraw", about = "Withdraw")]
    Withdraw(WithdrawArgs),
    #[command(name = "set-fee", about = "Set new fee, only owner allowed")]
    SetFee(DeployArgs),
    #[command(name = "collected-fees", about = "See the fees collected")]
    CollectedFees,
    #[command(
        name = "reserve-for-address",
        about = "See the sum reserved for a specific address"
    )]
    ReserveForAddress(WithdrawArgs),
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct WithdrawArgs {
    /// The sender of the withdraw request
    #[arg(short = 's', long = "sender")]
    pub sender: String,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct DeployArgs {
    /// The value of the contract fee
    #[arg(short = 'f', long = "value")]
    pub fee: u32,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct DepositArgs {
    /// The sender of the deposit sum
    #[arg(short = 's', long = "sender")]
    pub sender: String,

    /// The receiver of the deposit sum
    #[arg(short = 'r', long = "receiver")]
    pub receiver: String,

    /// The value of the deposit sum
    #[arg(short = 'v', long = "value")]
    pub value: u64,
}

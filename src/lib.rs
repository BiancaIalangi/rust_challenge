#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait RustChallenge {
    #[init]
    fn init(&self) {}

    #[only_owner]
    #[endpoint(setFee)]
    fn set_fee(&self, fee: BigUint) {
        self.fee().set(fee);
    }

    #[payable("EGLD")]
    #[endpoint]
    fn deposit(&self, receiver: ManagedAddress) {
        let payment_amount = self.call_value().egld_value();
        require!(
            *payment_amount > self.fee().get(),
            "Payments must be greater tha the fee"
        );
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[view]
    #[storage_mapper]
    fn fee(&self) -> SingleValueMapper<BigUint>;
}

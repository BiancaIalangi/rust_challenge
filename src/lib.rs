#![no_std]

use multiversx_sc::imports::*;
pub mod rust_challenge_proxy;

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
            "Payments must be greater than fee"
        );
        if self.collected_fees().is_empty() {
            self.collected_fees().set(self.fee().get());
        } else {
            self.collected_fees().update(|fee| *fee += self.fee().get());
        }

        let reserve = payment_amount.clone_value().sub(self.fee().get());

        if self.reserve_for_address(&receiver).is_empty() {
            self.reserve_for_address(&receiver).set(reserve);
        } else {
            self.reserve_for_address(&receiver)
                .update(|current_reserve| *current_reserve += reserve);
        }
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[view]
    #[storage_mapper]
    fn fee(&self) -> SingleValueMapper<BigUint>;

    #[view(getCollectedFees)]
    #[storage_mapper("collectedFees")]
    fn collected_fees(&self) -> SingleValueMapper<BigUint>;

    #[view(getReserveForAddress)]
    #[storage_mapper("reserveForAddress")]
    fn reserve_for_address(&self, receiver: &ManagedAddress) -> SingleValueMapper<BigUint>;
}

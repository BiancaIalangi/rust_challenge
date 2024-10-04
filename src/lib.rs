#![no_std]

use multiversx_sc::imports::*;
pub mod rust_challenge_proxy;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait RustChallenge {
    #[init]
    fn init(&self, fee: BigUint) {
        require!(fee >= BigUint::zero(), "Fee should be positive");
        self.set_fee(fee);
    }

    #[payable("EGLD")]
    #[endpoint]
    fn deposit(&self, receiver: ManagedAddress) {
        let payment_amount = self.call_value().egld_value();
        require!(
            *payment_amount > self.get_fee(),
            "Payments must be greater than fee"
        );
        if self.collected_fees().is_empty() {
            self.collected_fees().set(self.get_fee());
        } else {
            self.collected_fees().update(|fee| *fee += self.get_fee());
        }

        let reserve = payment_amount.clone_value().sub(self.get_fee());

        if self.reserve_for_address(&receiver).is_empty() {
            self.reserve_for_address(&receiver).set(reserve);
        } else {
            self.reserve_for_address(&receiver)
                .update(|current_reserve| *current_reserve += reserve);
        }
    }

    #[endpoint]
    fn withdraw(&self) {
        let caller = self.blockchain().get_caller();
        let owner = self.blockchain().get_owner_address();
        if !owner.eq(&caller) {
            require!(
                !self.reserve_for_address(&caller).is_empty(),
                "Nothing to claim"
            );
        }

        let transfer_amount = self.get_reserve_for_address(&caller);
        self.reserve_for_address(&caller).clear();
        self.tx().to(&caller).egld(transfer_amount).transfer();

        if owner.eq(&caller) {
            self.tx()
                .to(&caller)
                .egld(self.get_collected_fees())
                .transfer();
            self.collected_fees().clear();
        }
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[storage_mapper]
    fn fee(&self) -> SingleValueMapper<BigUint>;

    #[only_owner]
    #[endpoint(setFee)]
    fn set_fee(&self, fee: BigUint) {
        self.fee().set(fee);
    }

    #[view]
    fn get_fee(&self) -> BigUint {
        self.fee().get()
    }

    #[storage_mapper("collectedFees")]
    fn collected_fees(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("reserveForAddress")]
    fn reserve_for_address(&self, receiver: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(getReserveForAddress)]
    fn get_reserve_for_address(&self, receiver: &ManagedAddress) -> BigUint {
        self.reserve_for_address(receiver).get()
    }

    #[view(getCollectedFees)]
    fn get_collected_fees(&self) -> BigUint {
        self.collected_fees().get()
    }
}

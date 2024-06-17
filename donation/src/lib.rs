#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;

use odra::casper_types::U512;
use odra::prelude::*;
use odra::{Address, Var};

#[odra::event]
pub struct DonationReceived {
    pub donor: Address,
    pub amount: U512,
}

#[odra::event]
pub struct Withdrawal {
    pub amount: U512,
}

#[odra::odra_error]
pub enum Error {
    UnauthorizedToWithdraw = 0,
    CouldntGetBalance = 1,
}

#[odra::module(
    events = [DonationReceived, Withdrawal],
    errors = Error
)]
pub struct Donation {
    balance: Var<U512>,
    owner: Var<Address>,
}

#[odra::module]
impl Donation {
    pub fn init(&mut self) {
        self.owner.set(self.env().caller());
        self.balance.set(U512::from(0));
    }

    #[odra(payable)]
    pub fn donate(&mut self) {
        let amount: U512 = self.env().attached_value();

        self.balance.add(amount);

        self.env().emit_event(DonationReceived {
            donor: self.env().caller(),
            amount,
        });
    }

    pub fn withdraw(&mut self) {
        let caller = self.env().caller();
        if self.owner.get().unwrap() != caller {
            self.env().revert(Error::UnauthorizedToWithdraw);
        }
        let current_balance: U512 = self.balance.get_or_default();
        self.balance.set(U512::from(0));
        self.env().transfer_tokens(&caller, &current_balance);

        self.env().emit_event(Withdrawal {
            amount: current_balance,
        });
    }

    pub fn get_balance(self) -> U512 {
        self.balance.get_or_revert_with(Error::CouldntGetBalance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef, NoArgs};

    #[test]
    fn donate() {
        let env = odra_test::env();
        let contract = DonationHostRef::deploy(&env, NoArgs);
        let donation_amount = U512::from(1_000_000_000);
        let caller_initial_balance = env.balance_of(&env.get_account(0));
        contract
            .with_tokens(donation_amount)
            .try_donate()
            .expect("Donation should be successful");
        assert_eq!(
            env.balance_of(&env.get_account(0)),
            caller_initial_balance - donation_amount
        );
        env.emitted_event(
            contract.address(),
            &DonationReceived {
                donor: env.get_account(0),
                amount: donation_amount,
            },
        );
    }

    #[test]
    fn withdraw() {
        let env = odra_test::env();
        let mut contract = DonationHostRef::deploy(&env, NoArgs);
        let donation_amount = U512::from(1_000_000_000);
        contract
            .with_tokens(donation_amount)
            .try_donate()
            .expect("Donation should be successful");
        let caller_initial_balance = env.balance_of(&env.get_account(0));
        let initial_contract_balance = contract
            .try_get_balance()
            .expect("Balance should be obtainable");
        env.set_caller(env.get_account(1));
        contract
            .try_withdraw()
            .expect_err("Withdrawal from non-owner should fail");
        env.set_caller(env.get_account(0));
        contract
            .try_withdraw()
            .expect("Withdrawal from owner should succeed");
        assert_eq!(
            env.balance_of(&env.get_account(0)),
            caller_initial_balance + initial_contract_balance
        );
        let new_contract_balance = contract
            .try_get_balance()
            .expect("Balance should be obtainable");
        assert_eq!(new_contract_balance, U512::from(0));
        env.emitted_event(
            contract.address(),
            &Withdrawal {
                amount: initial_contract_balance,
            },
        );
    }
}

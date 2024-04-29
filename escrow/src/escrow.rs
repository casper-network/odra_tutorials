extern crate std;
use std::collections::HashSet;

use odra::casper_types::U512;
use odra::prelude::*;
use odra::{Address, Event, Var};

#[odra::odra_error]
pub enum Error {
    NotDepositor = 0,
    NotArbiter = 1,
    GoodNotProvided = 2,
    FundsNotDeposited = 3,
    IllegalAccounts = 4,
    FundsAlreadyDeposited = 5,
    IncorrectDepositAmount = 6,
}
#[odra::odra_type]
pub enum Account {
    Depositor,
    Beneficiary,
    Arbiter,
}

#[derive(Event)]
pub struct DepositMade {
    pub depositor: Address,
    pub amount: U512,
}

#[derive(Event)]
pub struct GoodProvided {
    beneficiary: Address,
}

#[derive(Event)]
pub struct EscrowSettled {
    pub depositor: Address,
    pub beneficiary: Address,
    pub amount_paid: U512,
}

#[derive(Event)]
pub struct EscrowRejected {
    pub depositor: Address,
    pub beneficiary: Address,
    pub amount_returned: U512,
}

#[odra::module]
pub struct Escrow {
    arbiter: Var<Address>,
    depositor: Var<Address>,
    beneficiary: Var<Address>,
    balance: Var<U512>,
    good_provided: Var<bool>,
    deposit_amount: Var<U512>,
}

#[odra::module]
impl Escrow {
    pub fn init(
        &mut self,
        arbiter: Address,
        depositor: Address,
        beneficiary: Address,
        deposit_amount: U512,
    ) {
        let all_accounts = vec![self.env().caller(), arbiter, depositor, beneficiary];
        let mut accounts_set = HashSet::new();
        for account in all_accounts {
            if !accounts_set.insert(account) {
                self.env().revert(Error::IllegalAccounts);
            }
        }
        self.arbiter.set(arbiter);
        self.depositor.set(depositor);
        self.beneficiary.set(beneficiary);
        self.good_provided.set(false);
        self.deposit_amount.set(deposit_amount);
        self.balance.set(0.into());
    }

    #[odra(payable)]
    pub fn deposit(&mut self) {
        self.assert_caller(Account::Depositor);
        if self.balance.get().unwrap() != U512::from(0) {
            self.env().revert(Error::FundsAlreadyDeposited);
        }
        if self.env().attached_value() != self.deposit_amount.get().unwrap() {
            self.env().revert(Error::IncorrectDepositAmount);
        }
        self.balance.add(self.env().attached_value());
        self.env().emit_event(DepositMade {
            depositor: self.env().caller(),
            amount: self.env().attached_value(),
        });
    }

    pub fn provided_good(&mut self) {
        self.assert_caller(Account::Beneficiary);
        self.good_provided.set(true);
        self.env().emit_event(GoodProvided {
            beneficiary: self.env().caller(),
        });
    }

    pub fn settle(&mut self) {
        self.assert_caller(Account::Arbiter);
        if !self.good_provided.get().unwrap() {
            self.env().revert(Error::GoodNotProvided);
        }
        if self.balance.get().unwrap() != self.deposit_amount.get().unwrap() {
            self.env().revert(Error::FundsNotDeposited);
        }
        let contract_balance = self.balance.get_or_default();
        self.balance.set(0.into());
        self.good_provided.set(false);
        self.env()
            .transfer_tokens(&self.beneficiary.get().unwrap(), &contract_balance);
        self.env().emit_event(EscrowSettled {
            depositor: self.depositor.get().unwrap(),
            beneficiary: self.beneficiary.get().unwrap(),
            amount_paid: contract_balance,
        });
    }

    pub fn reject(&mut self) {
        self.assert_caller(Account::Arbiter);
        let contract_balance = self.balance.get_or_default();
        self.balance.set(0.into());
        self.good_provided.set(false);
        self.env()
            .transfer_tokens(&self.depositor.get().unwrap(), &contract_balance);
        self.env().emit_event(EscrowRejected {
            depositor: self.depositor.get().unwrap(),
            beneficiary: self.beneficiary.get().unwrap(),
            amount_returned: contract_balance,
        });
    }

    fn assert_caller(&self, account: Account) {
        let target_account = match account {
            Account::Depositor => self.depositor.get().unwrap(),
            Account::Arbiter => self.arbiter.get().unwrap(),
            Account::Beneficiary => self.beneficiary.get().unwrap(),
        };
        if target_account != self.env().caller() {
            self.env().revert(Error::NotDepositor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef};

    #[test]
    fn successful_escrow() {
        let env = odra_test::env();
        let arbiter = env.get_account(1);
        let depositor = env.get_account(2);
        let beneficiary = env.get_account(3);
        let deposit_amount = U512::from(10_000_000_000u64);
        let init_args = EscrowInitArgs {
            arbiter: arbiter,
            depositor: depositor,
            beneficiary: beneficiary,
            deposit_amount: deposit_amount,
        };
        // Account 0 Deploys Contract
        let mut contract = EscrowHostRef::deploy(&env, init_args);

        // Get initial balances
        let depositor_initial_balance = env.balance_of(&depositor);
        let beneficiary_initial_balance = env.balance_of(&beneficiary);

        // Depositor deposits 10 CSPR and expects success
        env.set_caller(depositor);
        contract
            .with_tokens(deposit_amount)
            .try_deposit()
            .expect("Deposit should be successful");
        env.emitted_event(
            contract.address(),
            &DepositMade {
                depositor: depositor,
                amount: deposit_amount,
            },
        );

        // Beneficiary provides good
        env.set_caller(beneficiary);
        contract
            .try_provided_good()
            .expect("Beneficiary should be able to provide good");
        env.emitted_event(
            contract.address(),
            &GoodProvided {
                beneficiary: beneficiary,
            },
        );

        // Arbiter settles escrow
        env.set_caller(arbiter);
        contract
            .try_settle()
            .expect("Arbiter should be able to settle escrow");
        env.emitted_event(
            contract.address(),
            &EscrowSettled {
                depositor: depositor,
                beneficiary: beneficiary,
                amount_paid: deposit_amount,
            },
        );

        // Assert proper balances
        assert_eq!(
            env.balance_of(&beneficiary),
            beneficiary_initial_balance + deposit_amount
        );

        assert_eq!(
            env.balance_of(&depositor),
            depositor_initial_balance - deposit_amount
        );
    }
}

use odra::casper_types::U512;
use odra::prelude::*;
use odra::Address;
use odra::Mapping;
use odra::Var;

#[odra::odra_error]
/// Errors that may occur during the contract execution.
pub enum Error {
    /// Insufficient balance for the requested transfer
    InsufficientBalance = 1,
    /// Caller is not the owner of the wallet
    NotAnOwner = 2,
    /// Guardian has already participated in a recovery attempt
    GuardianAlreadyRecovered = 3,
    /// Caller is not a registered recovery guardian
    NotAGuradian = 4,
    /// Provided recovery address doesn't match the previously set one
    RecoveryAddressMismatch = 5,
    /// Recovery threshold percentage is outside the valid range (50-100)
    InvalidThreshold = 6,
}

#[odra::module(errors = Error)]
pub struct Wallet {
    /// Address of the account's owner
    owner: Var<Address>,
    /// Mapping of recovery guardian addresses to their participation status (voted/not voted)
    recovery_guardians: Mapping<Address, bool>,
    /// Number of recovery votes received
    recover_votes: Var<u8>,
    /// Minimum number of votes required to recover
    recovery_threshold: Var<u8>,
    /// Address to which funds will be transferred upon successful recovery
    recovery_address: Var<Address>,
}

#[odra::module]
impl Wallet {
    /// Initializes the contract with a list of recovery guardians and an optional recovery threshold.
    /// Sets the threshold to 70% if not provided. Ensures the threshold is within the valid range (50-100%).
    pub fn init(&mut self, recovery_guardians: Vec<Address>, recovery_threshold: Option<u8>) {
        self.owner.set(self.env().caller());
        match recovery_threshold {
            None => self
                .recovery_threshold
                .set(recovery_guardians.len() as u8 * 70 / 100),
            Some(threshold) => {
                self.assert_valid_threshold(threshold);
                self.recovery_threshold
                    .set(recovery_guardians.len() as u8 * threshold / 100);
            }
        }
        self.recover_votes.set(0);
        for guardian in recovery_guardians {
            self.recovery_guardians.set(&guardian, false);
        }
    }

    /**********
     * TRANSACTIONS
     **********/

    #[odra(payable)]
    pub fn deposit(&mut self) {}

    /// Transfers funds to the specified address.
    /// Reverts if the caller is not the owner or the balance is insufficient.
    #[odra(payable)]
    pub fn transfer_to(&mut self, to: Address, amount: U512) {
        self.assert_owner();
        if amount > self.balance() {
            self.env().revert(Error::InsufficientBalance)
        }
        self.env().transfer_tokens(&to, &amount);
    }

    /// Initiates a recovery process by a guardian.
    /// Reverts if the caller is not a registered guardian, has already participated in a recovery attempt,
    /// or the provided recovery address doesn't match the previously set one (if any).
    /// Increments the vote count. If the threshold is reached, transfers funds to the recovery address.
    pub fn recover_to(&mut self, recovery_address: Address) {
        self.assert_recovery_guardian();
        self.assert_or_set_recovery_address(recovery_address);
        self.recover_votes.add(1);
        if self.recover_votes.get_or_default() >= self.recovery_threshold.get_or_default() {
            self.env()
                .transfer_tokens(&self.recovery_address.get().unwrap(), &self.balance());
        }
    }

    /**********
     * QUERIES
     **********/

    /// Returns the current contract balance (including potentially direct CSPR deposits).
    pub fn balance(&self) -> U512 {
        self.env().self_balance()
    }

    /**********
     * INTERNAL
     **********/

    /// Ensures the caller of the function is the current owner of the wallet.
    /// Reverts with `NotAnOwner` error if the caller is not the owner.
    fn assert_owner(&self) {
        if self.env().caller() != self.owner.get().unwrap() {
            self.env().revert(Error::NotAnOwner)
        }
    }

    /// Checks if the provided recovery address matches the existing one.
    /// If no recovery address is set, it sets the provided address.
    /// Reverts with `RecoveryAddressMismatch` error if the addresses don't match (and one is already set).
    fn assert_or_set_recovery_address(&mut self, recovery_address: Address) {
        match self.recovery_address.get() {
            Some(r_address) => {
                if r_address != recovery_address {
                    self.env().revert(Error::RecoveryAddressMismatch)
                }
            }
            None => self.recovery_address.set(recovery_address),
        }
    }

    /// Verifies if the caller is a registered recovery guardian for the wallet.
    /// Also checks if the guardian has already participated in a recovery attempt (voted).
    /// Reverts with appropriate errors (`NotAGuradian` or `GuardianAlreadyRecovered`) based on the check results.
    fn assert_recovery_guardian(&mut self) {
        let caller = &self.env().caller();
        match self.recovery_guardians.get(caller) {
            Some(vote) => {
                if vote {
                    self.env().revert(Error::GuardianAlreadyRecovered);
                } else {
                    self.recovery_guardians.set(caller, true);
                }
            }
            None => self.env().revert(Error::NotAGuradian),
        }
    }

    /// Ensures the provided recovery threshold value is within the valid range (50-100%).
    /// Reverts with `InvalidThreshold` error if the threshold is outside the allowed range.
    fn assert_valid_threshold(&self, threshold: u8) {
        if threshold < 50 || threshold > 100 {
            self.env().revert(Error::InvalidThreshold)
        }
    }
}

#[cfg(test)]
mod tests {

    use odra::prelude::*;
    use odra::host::{HostEnv, HostRef, Deployer};
	use super::{Error, WalletHostRef, WalletInitArgs};
    use odra::Address;
	use odra::casper_types::U512;

    struct Accounts {
        alice: Address,
        bob: Address,
        carol: Address,
        dan: Address,
        elon: Address,
    }

    fn get_accounts(env: &HostEnv) -> Accounts {
        Accounts {
            alice: env.get_account(0),
            bob: env.get_account(1),
            carol: env.get_account(2),
            dan: env.get_account(3),
            elon: env.get_account(4),
        }
    }

    fn setup(env: &HostEnv) -> (WalletHostRef, Accounts) {
        let acc = get_accounts(env);
        env.set_caller(env.get_account(0));
        (
            WalletHostRef::deploy(
                &env,
                WalletInitArgs {
                    recovery_guardians: vec![acc.bob, acc.carol, acc.dan],
                    recovery_threshold: None, // 70% by default
                },
            ),
            acc,
        )
    }

    #[test]
    fn transfer_not_an_owner() {
        let test_env: HostEnv = odra_test::env();
        let (mut wallet, acc) = setup(&test_env);

        test_env.set_caller(acc.bob);
        assert_eq!(
            wallet.try_transfer_to(acc.bob, U512::one()),
            Err(Error::NotAnOwner.into())
        );
    }

    #[test]
    fn transfer_owner_insuficient_balance() {
        let test_env: HostEnv = odra_test::env();
        let (mut wallet, acc) = setup(&test_env);

        assert_eq!(
            wallet.try_transfer_to(acc.bob, U512::one()),
            Err(Error::InsufficientBalance.into())
        );
    }

    #[test]
    fn transfer_owner() {
        let test_env: HostEnv = odra_test::env();
        let (mut wallet, acc) = setup(&test_env);
        let inital_bob_balance = test_env.balance_of(&acc.bob);
        assert_eq!(wallet.balance(), U512::zero());

        wallet.with_tokens(U512::from(100)).deposit();
        assert_eq!(wallet.balance(), U512::from(100));

        wallet.transfer_to(acc.bob, U512::one());
        assert_eq!(wallet.balance(), U512::from(99));
        assert_eq!(inital_bob_balance + 1, test_env.balance_of(&acc.bob));
    }

    #[test]
    fn recover_by_not_guardian() {
        let test_env: HostEnv = odra_test::env();
        let (mut wallet, acc) = setup(&test_env);

        assert_eq!(
            wallet.try_recover_to(acc.elon),
            Err(Error::NotAGuradian.into())
        );
    }

    #[test]
    fn recover_by_guardian() {
        let test_env: HostEnv = odra_test::env();
        let (mut wallet, acc) = setup(&test_env);

        test_env.set_caller(acc.bob);
        wallet.recover_to(acc.elon);
    }

    #[test]
    fn recovery_already_attempted_recover() {
        let test_env: HostEnv = odra_test::env();
        let (mut wallet, acc) = setup(&test_env);

        // bob wants to recover to elon
        test_env.set_caller(acc.bob);
        wallet.recover_to(acc.elon);

        // bob tires to submit the recovery request agains
        test_env.set_caller(acc.bob);
        assert_eq!(
            wallet.try_recover_to(acc.elon),
            Err(Error::GuardianAlreadyRecovered.into())
        );
    }

    #[test]
    fn recovery_address_missmatch() {
        let test_env: HostEnv = odra_test::env();
        let (mut wallet, acc) = setup(&test_env);

        // bob wants to recover to elon
        test_env.set_caller(acc.bob);
        wallet.recover_to(acc.elon);

        // carol wants to recover to alice
        test_env.set_caller(acc.carol);
        assert_eq!(
            wallet.try_recover_to(acc.alice),
            Err(Error::RecoveryAddressMismatch.into())
        );
    }

    #[test]
    fn recover_to() {
        let test_env: HostEnv = odra_test::env();
        let (mut wallet, acc) = setup(&test_env);

        let elon_initial_balance = test_env.balance_of(&acc.elon);
        wallet.with_tokens(U512::from(100)).deposit();

        // bob submits the recovery request
        test_env.set_caller(acc.bob);
        wallet.recover_to(acc.elon);

        // after the first requeset the funds should still be in the wallet
        assert_eq!(test_env.balance_of(&acc.elon), elon_initial_balance);
        assert_eq!(wallet.balance(), U512::from(100));

        // carol submits the same recovery request
        test_env.set_caller(acc.carol);
        wallet.recover_to(acc.elon);

        // after the second request (threshold has been reached) the wallet should be empty
        // and the recovery address should have the funds
        assert_eq!(test_env.balance_of(&acc.elon), elon_initial_balance + 100);
        assert_eq!(wallet.balance(), U512::from(0));
    }
}

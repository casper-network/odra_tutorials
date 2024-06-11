# Recoverable Wallet contract with Odra

## Introduction

This tutorial shows how to create a smart contract that acts as a personal wallet with some additional fetaures on top. This concept is known as account abstraction. The idea is to delegate all the funds to a smart contract and interact with the smart contract as a user would normally intercat with a wallet, but the smart contract introduces some extra features. It might be social recoery using trusted addresses to recover the account in case you lost it, daily transaction limits, allowist for transfers exceding a given amount of tokens etc. To read more about it check this link: (here paste a link to some articale explaining it) (you can even add more link here to some etherum standatds etc.)

In this example we implement the social recovery feature. Where user can set a list of trusted addresses (`recovery_guardians`) that in case of a lost key to this wallet, can recover the funds and transfer them to a new account.

**1. Core Functionalities:**

- **Secure Token Storage:** The contract acts as a digital vault for your CSPR tokens. Deposit and manage your balance safely.
- **Fund Transfers:** Transfer your tokens to other accounts, ensuring sufficient balance for successful transactions.
- **Multi-Guardian Recovery:** Protect your assets from permanent loss! Appoint trusted guardians who can collaboratively recover funds if you lose access.

## Building Your Secure CSPR Wallet: A Step-by-Step Guide

This comprehensive guide walks you through creating a secure CSPR wallet smart contract using the Odra development framework. By following these steps, you'll gain a solid understanding of the contract's functionalities and how to leverage them for safekeeping your digital assets.

### 1. Project Setup and Dependencies

Let's begin by setting up the project environment.
Initalize new blank template of Odra project:

```bash
cargo odra new --name recoverable_wallet -t blank
```

Open _recoverable/src/lib.rs_ in your editor

Now include the necessary dependencies for your `Wallet` contract

```rust
use odra::casper_types::U512;
use odra::prelude::*;
use odra::Address;
use odra::Mapping;
use odra::OdraError;
use odra::Var;
```

**Explanation of Dependencies:**

- `odra::casper_types::U512`: This dependency provides the `U512` data type, allowing us to represent large unsigned integers, perfect for handling CSPR token amounts.
- `odra::prelude::*`: This imports essential functionalities frequently used in Odra development, including functions for interacting with the environment and basic data structures.
- `odra::Address`: This defines the `Address` type, representing account addresses on the Casper blockchain where CSPR tokens can be stored and transferred.
- `odra::Mapping`: This provides the `Mapping` data structure, enabling us to efficiently store key-value pairs within the contract. In our case, it will be used to manage recovery guardian information.
- `odra::OdraError`: This dependency allows defining custom error types for the contract.
- `odra::Var`: This provides the `Var` type, representing variables that can be persistently stored within the contract's memory.

### 2. Defining Contract Errors

The `Error` enum in the code snippet outlines potential errors that might occur during contract interactions. These errors provide informative messages to users, helping them understand issues like insufficient balance or unauthorized actions.

```rust
#[derive(OdraError)]
// Errors that may occur during the contract execution.
pub enum Error {
    // Insufficient balance for the requested transfer
    InsufficientBalance = 1,
    // Caller is not the owner of the wallet
    NotAnOwner = 2,
    // Guardian has already participated in a recovery attempt
    GuardianAlreadyRecovered = 3,
    // Caller is not a registered recovery guardian
    NotAGuradian = 4,
    // Provided recovery address doesn't match the previously set one
    RecoveryAddressMismatch = 5,
    // Recovery threshold percentage is outside the valid range (50-100)
    InvalidThreshold = 6,
}
```

### 3. The `Wallet` Smart Contract

Now, let's delve into the core structure of the `Wallet` contract:

```rust
#[odra::module]
pub struct Wallet {
    // Address of the account's owner
    owner: Var<Address>,
    // Mapping of recovery guardian addresses to their participation status (voted/not voted)
    recovery_guardians: Mapping<Address, bool>,
    // Number of recovery votes received
    recover_votes: Var<u8>,
    // Minimum number of votes required to recover
    recovery_threshold: Var<u8>,
    // Address to which funds will be transferred upon successful recovery
    recovery_address: Var<Address>,
}
```

This section defines the contract's state variables using the `Var` type from `odra::Var`. These variables hold crucial information for the contract's operation:

- `owner`: Stores the address of the account that owns the wallet.
- `recovery_guardians`: A mapping that associates recovery guardian addresses with a boolean value indicating their voting participation (voted or not voted) during a recovery attempt.
- `recover_votes`: Tracks the current number of votes received towards recovery.
- `recovery_threshold`: Defines the minimum number of guardian votes required for a successful recovery.
- `recovery_address`: Holds the address where funds will be transferred upon a successful recovery.

### 4. Contract Initialization (`init` Function)

The `init` function acts as a contructor for our smart contract. We call it only once upon initialization.

```rust
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
```

**Explanation:**

- This function takes two arguments:
  - `recovery_guardians`: A vector containing the addresses of the chosen recovery guardians.
  - `recovery_threshold` (optional): An optional value specifying the minimum number of guardian votes required for recovery (as a percentage). If not provided, it defaults to 70%.
- The function first sets the `owner` variable to the address of the account calling the `init` function, which becomes the initial owner of the wallet.
- It then checks the provided `recovery_threshold`:
  - If no threshold is provided, it calculates a default value as 70% of the total number of recovery guardians.
  - If a threshold is provided, it ensures it falls within the valid range (50-100%) using the `assert_valid_threshold` function (explained later). Then, it calculates the threshold based on the provided percentage of the total guardians.
- The initial `recover_votes` are set to 0, indicating no recovery votes received yet.
- Finally, the function iterates over the `recovery_guardians` vector and sets their corresponding entries in the `recovery_guardians` mapping to `false`, signifying they haven't participated in any recovery attempts yet.

### **5. Transactions:**

The following functions represent functionalities users can interact with:

```rust
#[odra(payable)]
pub fn deposit(&mut self) {}

// Transfers funds to the specified address.
// Reverts if the caller is not the owner or the balance is insufficient.
#[odra(payable)]
pub fn transfer_to(&mut self, to: Address, amount: U512) {
	self.assert_owner();
	if amount > self.balance() {
		self.env().revert(Error::InsufficientBalance)
	}
	self.env().transfer_tokens(&to, &amount);
}

// Initiates a recovery process by a guardian.
// Reverts if the caller is not a registered guardian, has already participated in a recovery attempt,
// or the provided recovery address doesn't match the previously set one (if any).
// Increments the vote count. If the threshold is reached, transfers funds to the recovery address.
pub fn recover_to(&mut self, recovery_address: Address) {
	self.assert_recovery_guardian();
	self.assert_or_set_recovery_address(recovery_address);
	self.recover_votes.add(1);
	if self.recover_votes.get_or_default() >= self.recovery_threshold.get_or_default() {
		self.env()
			.transfer_tokens(&self.recovery_address.get().unwrap(), &self.balance());
		}
}
```

**Explanation:**

- `deposit`: This function allows users to deposit CSPR tokens into their wallet. You'll need to fill in the implementation details based on your specific needs (e.g., handling received tokens).

- `transfer_to`: This function enables users to transfer CSPR tokens from their wallet to another specified address. However, it ensures that only the wallet owner can initiate this action and verifies sufficient funds are available to avoid failures. Any unsuccessful transfers due to insufficient balance will result in an `Error::InsufficientBalance` being reverted back to the user.

- `recover_to`: This function allows authorized recovery guardians to initiate a recovery process. It performs several checks:
  - Ensures the caller is a registered recovery guardian using `assert_recovery_guardian`.
  
  - Verifies if the provided recovery address matches the previously set one using `assert_or_set_recovery_address`. This ensures consistency in the recovery destination.
  
  - Increments the `recover_votes` counter to track the number of guardians who have participated in the current recovery attempt.
  
  - If the number of received votes reaches or exceeds the defined recovery threshold (checked using `get_or_default` for both variables), it transfers the entire wallet balance to the recovery address stored in `recovery_address`.
  

### **6. Queries:**

The following function `balance` allows users to retrieve information from the contract:

```rust
/// Returns the current contract balance (including potentially direct CSPR deposits).
pub fn balance(&self) -> U512 {
	self.env().self_balance()
}
```

**Explanation:**

- `balance`: This function provides users with the current CSPR balance held within the wallet contract. It utilizes the `self.env().self_balance()` Odra-provided function to access the contract's own balance information.

### **7. Internal Functions:**

These functions are internal helper functions used by other parts of the contract and not directly accessible by users:

```rust
// Ensures the caller of the function is the current owner of the wallet.
// Reverts with `NotAnOwner` error if the caller is not the owner.
fn assert_owner(&self) {
	if self.env().caller() != self.owner.get().unwrap() {
		self.env().revert(Error::NotAnOwner)
	}
}

// Checks if the provided recovery address matches the existing one.
// If no recovery address is set, it sets the provided address.
// Reverts with `RecoveryAddressMismatch` error if the addresses don't match (and one is already set).
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

// Verifies if the caller is a registered recovery guardian for the wallet.
// Also checks if the guardian has already participated in a recovery attempt (voted).
// Reverts with appropriate errors (`NotAGuradian` or `GuardianAlreadyRecovered`) based on the check results.
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

// Ensures the provided recovery threshold value is within the valid range (50-100%).
// Reverts with `InvalidThreshold` error if the threshold is outside the allowed range.
fn assert_valid_threshold(&self, threshold: u8) {
	if threshold < 50 || threshold > 100 {
		self.env().revert(Error::InvalidThreshold)
	}
}
```

**Explanation:**

- `assert_owner`: This function ensures that the caller of any function requiring owner privileges is indeed the current owner of the wallet. If the caller is not the owner, the function reverts with a `NotAnOwner` error. This check is critical to maintaining the security and integrity of the wallet by ensuring that only the designated owner can perform certain sensitive operations, such as transferring funds.
- `assert_or_set_recovery_address`: This function checks if the provided recovery address matches the one already set in the contract. If no recovery address is set, it sets the provided address as the recovery address. If the addresses do not match, and a recovery address is already set, it reverts with a `RecoveryAddressMismatch` error. This function is essential to prevent unauthorized changes to the recovery address, ensuring that the recovery process remains consistent and secure.
- `assert_recovery_guardian`: This function verifies if the caller is a registered recovery guardian for the wallet and checks if they have already participated in the recovery process. If the caller is not a registered guardian, it reverts with a `NotAGuradian` error. If the guardian has already participated (voted), it reverts with a `GuardianAlreadyRecovered` error. If the caller passes these checks, their participation in the recovery process is recorded. This mechanism ensures that only authorized guardians can initiate or participate in the recovery process, and it prevents duplicate voting, maintaining the integrity of the recovery process.
- `assert_valid_threshold`: This function ensures that the provided recovery threshold value is within the valid range of 50-100%. If the threshold is outside this range, it reverts with an `InvalidThreshold` error. This check is crucial to ensure that the recovery threshold is set to a sensible value, maintaining the balance between security and recoverability. The threshold determines the percentage of recovery guardians required to approve the recovery process, ensuring a fair and secure recovery mechanism.

### **8. Unit Tests:**

Now that the smart contract itself is complete you can write tests to ensure the functions work as they are expected to.

First, open a new module `tests`, annotated with Rust's `#[cfg(test)]` attribute, and import the required dependencies:

```rust
#[cfg(test)]
mod tests {
  use odra::prelude::*;
  use odra::host::{HostEnv, HostRef, Deployer};
	use super::{Error, WalletHostRef, WalletInitArgs};
  use odra::Address;
	use odra::casper_types::U512;
}
```

**Explanation:**

* `use odra::prelude::*` Imports essential functionalities frequently used in Odra development, including functions for interacting with the environment and basic data structures.
* ` use odra::host::{HostEnv, HostRef, Deployer}`
  * `HostEnv` is a struct that provides methods for interacting with the underlying host context and managing the execution of contracts.
  * `HostRef` is a trait implemented by `WalletHostRef` which references a deployed contract instance within the host environment, used to call methods on the contract during tests.
  * `Deployer`: Used to deploy instances of contracts within the host environment.
* `use super::{Error, WalletHostRef, WalletInitArgs}`
  * `Error` is the enum of user errors created in the smart contract.
  * `WalletHostRef` is a reference to the contract that we can use to interact with it and implements the `HostRef` trait.
  * `WalletInitArgs` is a struct that consists of the initialization arguments for the contract, in this case  `pub recovery_guardians: Vec<Address>` and `pub recovery_threshold: Option<u8>`.
* `use odra::Address` is a data structure used to store Casper account addresses.
* `use odra::casper_types::U512` imports the `U512` type from the Casper-specific types provided by the Odra framework. `U512` is a large unsigned integer type used for handling large values, such as token balances, within the contract.

Create the functions:

```rust
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
```

**Explanation:**

- `fn get_accounts(env: &HostEnv) -> Accounts`:
  - This function retrieves a set of predefined accounts from the test environment.
  - It returns an `Accounts` struct containing five accounts: `alice`, `bob`, `carol`, `dan`, and `elon`, which are used throughout the tests.
- `fn setup(env: &HostEnv) -> (WalletHostRef, Accounts)`:
  - This function initializes the test environment and deploys the `Wallet` contract.
  - It sets the caller to the first account (typically the owner of the wallet).
  - The function deploys the wallet with `bob`, `carol`, and `dan` as recovery guardians and uses the default recovery threshold (70%).
  - It returns a reference to the deployed wallet and the accounts struct for use in tests.

Now the tests can be written. These functions are used to test the functionality of the smart contract by simulating different scenarios and verifying the expected outcomes.

Create the functions:

* `#[test] fn transfer_not_an_owner()`:

  - This test ensures that a non-owner cannot transfer funds from the wallet.

  - `bob` attempts to transfer funds but receives a `NotAnOwner` error, confirming that only the owner can initiate transfers.

```rust
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
```

* `#[test] fn transfer_owner_insuficient_balance()`:
  - This test checks that a transfer fails if the owner does not have enough balance.
  - The owner tries to transfer `U512::one()` (1 token) without any balance, resulting in an `InsufficientBalance` error.

```rust
#[test]
fn transfer_owner_insufficient_balance() {
	let test_env: HostEnv = odra_test::env();
	let (mut wallet, acc) = setup(&test_env);
	assert_eq!(
		wallet.try_transfer_to(acc.bob, U512::one()),
		Err(Error::InsufficientBalance.into())
	);
}
```

* `#[test] fn transfer_owner()`:
  - This test verifies that the owner can successfully transfer funds if there is sufficient balance.
  - The owner deposits 100 tokens, then transfers 1 token to `bob`. The balances are checked to ensure the transfer occurred correctly.

```rust
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
```

* `#[test] fn recover_by_not_guardian()`:
  - This test ensures that only registered recovery guardians can initiate the recovery process.
  - `elon`, who is not a guardian, tries to initiate recovery and receives a `NotAGuradian` error.

```rust
#[test]
fn recover_by_not_guardian() {
	let test_env: HostEnv = odra_test::env();
	let (mut wallet, acc) = setup(&test_env);

	assert_eq!(
		wallet.try_recover_to(acc.elon),
		Err(Error::NotAGuradian.into())
	);
}
```

* `#[test] fn recover_by_guardian()`:
  - This test confirms that a registered recovery guardian can successfully initiate the recovery process.
  - `bob`, a registered guardian, initiates recovery to `elon` without any errors.

```rust
#[test]
fn recover_by_guardian() {
	let test_env: HostEnv = odra_test::env();
	let (mut wallet, acc) = setup(&test_env);
	test_env.set_caller(acc.bob);
	wallet.recover_to(acc.elon);
}
```

* `#[test] fn recovery_already_attempted_recover()`:
  - This test checks that a guardian cannot participate in the recovery process more than once.
  - `bob` initiates recovery to `elon`, then tries again and receives a `GuardianAlreadyRecovered` error.

```rust
#[test]
fn recovery_already_attempted_recover() {
	let test_env: HostEnv = odra_test::env();
	let (mut wallet, acc) = setup(&test_env);

	// bob wants to recover to elon
	test_env.set_caller(acc.bob);
	wallet.recover_to(acc.elon);

	// bob tries to submit the recovery request again
	test_env.set_caller(acc.bob);
	assert_eq!(
		wallet.try_recover_to(acc.elon),
		Err(Error::GuardianAlreadyRecovered.into())
	);
}
```

* `#[test] fn recovery_address_missmatch()`:
  - This test ensures that the recovery address must remain consistent throughout the process.
  - `bob` initiates recovery to `elon`, and when `carol` tries to recover to a different address (`alice`), a `RecoveryAddressMismatch` error is returned.

```rust
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
```

* `#[test] fn recover_to()`:
  - This test verifies the complete recovery process.
  - The owner deposits 100 tokens, and `bob` initiates recovery to `elon`. The balance remains until `carol` also initiates recovery to `elon`, meeting the threshold and transferring the funds to `elon`. The balances are checked to ensure the process completes as expected.

```rust
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
```

## Conclusion

In this tutorial, we walked through the development of a recoverable wallet smart contract using Rust and the Odra Framework. We covered various aspects of the contract, including the initialization, transaction functionalities, and the recovery mechanism. Here’s a summary of what was accomplished:

1. **Contract Initialization**:
   - We defined the contract’s state variables, including the owner’s address, recovery guardians, vote count, recovery threshold, and the recovery address.
   - The `init` function was implemented to set up the contract with a list of recovery guardians and an optional recovery threshold, ensuring all necessary checks and defaults are applied.
2. **Transaction Functions**:
   - We created the `deposit` function to allow users to deposit CSPR tokens into the wallet.
   - The `transfer_to` function was implemented to enable the owner to transfer tokens to a specified address, with checks for ownership and sufficient balance.
   - The `recover_to` function was designed to facilitate a recovery process initiated by guardians, involving checks for guardian status, address consistency, and vote counting, ultimately transferring the balance to a designated recovery address if the threshold is met.
3. **Internal Functions**:
   - Several internal helper functions were developed to ensure contract security and proper operation, including `assert_owner`, `assert_or_set_recovery_address`, `assert_recovery_guardian`, and `assert_valid_threshold`.
4. **Testing**:
   - We wrote comprehensive tests to validate the functionality of our smart contract, covering various scenarios such as non-owner transfers, insufficient balances, recovery attempts by non-guardians, and ensuring the integrity of the recovery process.

By following this tutorial, you have gained insight into building a secure and recoverable wallet on the Casper Network, leveraging the Odra Framework. This approach ensures that wallet owners can recover their assets in case of lost access while maintaining security through a guardian-based recovery mechanism. The skills and knowledge acquired here can be applied to develop more complex and robust smart contracts.

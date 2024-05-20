 # Recoverable Wallet contract with Odra

 ## Introduction

 This tutorial shows how to create a smart contract that acts as a personal wallet with some additional fetaures on top. This concept is known as account abstraction. The idea is to delegate all the funds to a smart contract and interact with the smart contract as a user would normally intercat with a wallet, but the smart contract introduces some extra features. It might be social recoery using trusted addresses to recover the account in case you lost it, daily transaction limits, allowist for transfers exceding a given amount of tokens etc. To read more about it check this link: (here paste a link to some articale explaining it) (you can even add more link here to some etherum standatds etc.) 

 In this example we implement the social recovery feature. Where user can set a list of trusted addresses (`recovery_guardians`) that in case of a lost key to this wallet can recover the funds and transfer them to a new account. 

 **1. Core Functionalities:**

* **Secure Token Storage:** The contract acts as a digital vault for your CSPR tokens. Deposit and manage your balance safely.
* **Fund Transfers:** Transfer your tokens to other accounts, ensuring sufficient balance for successful transactions.
* **Multi-Guardian Recovery:** Protect your assets from permanent loss! Appoint trusted guardians who can collaboratively recover funds if you lose access.

## Building Your Secure CSPR Wallet: A Step-by-Step Guide

This comprehensive guide walks you through creating a secure CSPR wallet smart contract using the Odra development framework. By following these steps, you'll gain a solid understanding of the contract's functionalities and how to leverage them for safekeeping your digital assets.

### 1. Project Setup and Dependencies

Let's begin by setting up the project environment.
Initalize new blank template of Odra project:
 
 ```bash
cargo odra new --name recoverable_wallet -t blank
```

Open *recoverable/src/lib.rs* in your editor

Now include the necessary dependencies for our `Wallet` contract

```rust
use odra::casper_types::U512;
use odra::prelude::*;
use odra::Address;
use odra::Mapping;
use odra::OdraError;
use odra::Var;
```

**Explanation of Dependencies:**

* `odra::casper_types::U512`: This dependency provides the `U512` data type, allowing us to represent large unsigned integers, perfect for handling CSPR token amounts.
* `odra::prelude::*`: This imports essential functionalities frequently used in Odra development, including functions for interacting with the environment and basic data structures.
* `odra::Address`: This defines the `Address` type, representing account addresses on the Casper blockchain where CSPR tokens can be stored and transferred.
* `odra::Mapping`: This provides the `Mapping` data structure, enabling us to efficiently store key-value pairs within the contract. In our case, it will be used to manage recovery guardian information.
* `odra::OdraError`: This dependency allows defining custom error types for the contract.
* `odra::Var`: This provides the `Var` type, representing variables that can be persistently stored within the contract's memory.

### 2. Defining Contract Errors

The `Error` enum in the code snippet outlines potential errors that might occur during contract interactions. These errors provide informative messages to users, helping them understand issues like insufficient balance or unauthorized actions.

```rust
#[derive(OdraError)]
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
```

### 3. The `Wallet` Smart Contract

Now, let's delve into the core structure of the `Wallet` contract:

```rust
#[odra::module]
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
```

This section defines the contract's state variables using the `Var` type from `odra::Var`. These variables hold crucial information for the contract's operation:

* `owner`: Stores the address of the account that owns the wallet.
* `recovery_guardians`: A mapping that associates recovery guardian addresses with a boolean value indicating their voting participation (voted or not voted) during a recovery attempt.
* `recover_votes`: Tracks the current number of votes received towards recovery.
* `recovery_threshold`: Defines the minimum number of guardian votes required for a successful recovery.
* `recovery_address`: Holds the address where funds will be transferred upon a successful recovery.

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

* This function takes two arguments:
    * `recovery_guardians`: A vector containing the addresses of the chosen recovery guardians.
    * `recovery_threshold` (optional): An optional value specifying the minimum number of guardian votes required for recovery (as a percentage). If not provided, it defaults to 70%.
* The function first sets the `owner` variable to the address of the account calling the `init` function, which becomes the initial owner of the wallet.
* It then checks the provided `recovery_threshold`:
    * If no threshold is provided, it calculates a default value as 70% of the total number of recovery guardians.
    * If a threshold is provided, it ensures it falls within the valid range (50-100%) using the `assert_valid_threshold` function (explained later). Then, it calculates the threshold based on the provided percentage of the total guardians.
* The initial `recover_votes` are set to 0, indicating no recovery votes received yet.
* Finally, the function iterates over the `recovery_guardians` vector and sets their corresponding entries in the `recovery_guardians` mapping to `false`, signifying they haven't participated in any recovery attempts yet.

**### 5. Transactions:**

The following functions represent functionalities users can interact with:

```rust
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
```

**Explanation:**

* `deposit`: This function allows users to deposit CSPR tokens into their wallet. You'll need to fill in the implementation details based on your specific needs (e.g., handling received tokens).
* `transfer_to`: This function enables users to transfer CSPR tokens from their wallet to another specified address. However, it ensures that only the wallet owner can initiate this action and verifies sufficient funds are available to avoid failures. Any unsuccessful transfers due to insufficient balance will result in an `Error::InsufficientBalance` being reverted back to the user.
* `recover_to`: This function allows authorized recovery guardians to initiate a recovery process. It performs several checks:
    * Ensures the caller is a registered recovery guardian using `assert_recovery_guardian`.
    * Verifies if the provided recovery address matches
    Absolutely! Let's complete the step-by-step guide for the `Wallet` contract:

**### 5. Transactions (continued):**

* Verifies if the provided recovery address matches the previously set one using `assert_or_set_recovery_address`. This ensures consistency in the recovery destination.
* Increments the `recover_votes` counter to track the number of guardians who have participated in the current recovery attempt.
* If the number of received votes reaches or exceeds the defined recovery threshold (checked using `get_or_default` for both variables), it transfers the entire wallet balance to the recovery address stored in `recovery_address`.

**### 6. Queries:**

The following function allows users to retrieve information from the contract:

```odra
    /// Returns the current contract balance (including potentially direct CSPR deposits).
    pub fn balance(&self) -> U512 {
        self.env().self_balance()
    }
```

**Explanation:**

* `balance`: This function provides users with the current CSPR balance held within the wallet contract. It utilizes the `self.env().self_balance()` function to access the contract's own balance information.

**### 7. Internal Functions:**

These functions are internal helper functions used by other parts of the contract and not directly accessible by users:

```odra
    /// Ensures the caller of the function is the current owner of the wallet.
    /// Reverts with `NotAnOwner` error if the caller is not the owner.
    fn assert_owner(&self) {
        if self.env().caller() != self.owner.get().unwrap() {
            self.env().revert(Error::NotAnOwner)
        }
    }

    // ... other internal functions explained below
```

**Explanation:**

* `assert_owner`: This internal function verifies if the caller of a function is the authorized owner of the wallet. It compares the caller's address with the address stored in the `owner` variable and reverts with an `Error::NotAnOwner` error if there's a mismatch.

**Here's the breakdown of the remaining internal functions:**

* `assert_recovery_guardian`: This function checks if the caller is a registered recovery guardian for the wallet and if they haven't already participated in the ongoing recovery attempt. It reverts with appropriate errors (`NotAGuradian` or `GuardianAlreadyRecovered`).
* `assert_or_set_recovery_address`: This function ensures the provided recovery address aligns with the existing one (if any). If no address is set, it allows setting the provided address. It reverts with an `Error::RecoveryAddressMismatch` if there's a mismatch and an address is already set.
* `assert_valid_threshold`: This function validates the provided recovery threshold, ensuring it falls within the allowed range (50-100%). It reverts with an `Error::InvalidThreshold` if the threshold is outside the valid range.

**### 8. Unit Tests:**

The provided test suite (shown as `#[cfg(test)] mod tests {...}`) plays a crucial role in verifying the contract's functionalities and ensuring it behaves as intended under different scenarios. These tests typically involve setting up a test environment, deploying the contract, and simulating various user interactions to validate expected outcomes. By incorporating a robust test suite, you can enhance the contract's reliability and catch potential issues before deployment on the blockchain.

This comprehensive walkthrough has equipped you with an in-depth understanding of the `Wallet` smart contract, its functionalities, and how it facilitates secure CSPR management. Remember to tailor the deposit function implementation and testing scenarios based on your specific requirements for a complete and secure solution.


# Escrow Contract with Odra

## Introduction

Escrow contracts are common and useful agreements for arbitrating arrangements between two or more parties. This tutorial will teach you how to create a basic escrow smart contract between two accounts with a dedicated arbiter.

## Terms

There are many ways to prepare escrow agreements. This contract will assume the following terms:

* There is one depositor, one beneficiary, and one arbiter, which must all be unique from one another.
* The deploying account may not be the depositor, beneficiary, or arbiter.
* The depositor must deposit funds, and the beneficiary must provide an off-chain good before the escrow can be settled by the arbiter.
* The arbiter can reject the agreement, returning funds to the depositor.
* Once the escrow agreement is settled, the deposited funds are sent to the beneficiary.

This contract could be altered or extended to change the terms of the agreement in order to support multiple accounts, specific conditions, two on-chain provisions, etc.

## Preparation

Initialize a new Odra project:

```bash
cargo odra new --name escrow -t blank
```

*Note: The `-t blank` flag will create the contract as a blank template.*

Open *escrow/src/lib.rs* in an editor.

Begin by importing the required Odra types:

```rust
use odra::casper_types::U512;
use odra::prelude::*;
use odra::{Address, Event, Var};
```

To efficiently detect intent to use unallowed accounts, you can use `HashSet` from the `std` library:

```rust
extern crate std;
use std::collections::HashSet;
```

## User Errors

You can prepare custom user errors which are useful for debugging the contract upon reverts. For this example, the following errors will be defined:

```rust
#[odra::odra_error]
pub enum Error {
    NotDepositor = 0,
    NotArbiter = 1,
    GoodNotProvided = 2,
    FundsNotDeposited = 3,
    IllegalAccounts = 4,
  	FundsAlreadyDeposited = 5,
  	IncorrectDepositAmount = 6
}
```

## Custom Types

To efficiently assert that the correct account is calling a given entrypoint, prepare a custom Odra type `Account` that will differentiate the depositor, beneficiary, and arbiter:

```rust
#[odra::odra_type]
pub enum Account {
    Depositor,
    Beneficiary,
    Arbiter,
}
```

## Events

By creating custom events and emitting them in specific situations, listeners of the contract's event stream can pick up on and react to actions taken on the contract. Add the following four events to the contract:

```rust
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
```

## Contract Interface

Before implementing the contract, a module definition must first be created, including all of the objects stored by the contract:

```rust
#[odra::module]
pub struct Escrow {
    arbiter: Var<Address>,
    depositor: Var<Address>,
    beneficiary: Var<Address>,
  	balance: Var<U512>,
    good_provided: Var<bool>,
    deposit_amount: Var<U512>,
}
```

In this implementation, the objects reference the following:

* `arbiter`:

  The account arbitrating the escrow agreement.

* `depositor`:

  The account responsible for depositing the funds; the "buying" account.

* `beneficiary`:

  The account receiving the funds; the "selling" account.

* `balance`:

  The balance of the contract. Tracks the amount owned by the contract at any given time.

* `good_provided`:

  Tracks whether the beneficiary has provided the off-chain good to the arbiter.

* `deposit_amount`:

  The amount the depositor must deposit.

## Contract Implementation

To begin writing the smart contract's functionality, implement the `Escrow` module, marking the implementation with the `#[odra::module]` attribute:

```rust
#[odra::module]
impl Escrow {
	 
}
```

### Constructor

Define the constructor, which is called upon contract deployment and must be named `init`:

```rust
pub fn init(
	&mut self,
	arbiter: Address,
	depositor: Address,
	beneficiary: Address,
	deposit_amount: U512,
) {}
```

The `&mut self` parameter allows us to access and mutate `self` and its objects. The `arbiter`, `depositor`, and `beneficiary` are three unique Casper accounts defined by the contract deployer. The `deposit_amount` is also defined by the contract deployer, and will be the amount the depositor must deposit to initiate the escrow.

Within the constructor, start by ensuring that the contract deployer, `arbiter`, `depositor`, and `beneficiary` are all unique accounts. This can be done efficiently by inserting each into a `HashSet`, if an `Address` is not able to be inserted, it is not unique, and the contract will revert with `IllegalAccounts`:

```rust
let all_accounts = vec![self.env().caller(), arbiter, depositor, beneficiary];
let mut accounts_set = HashSet::new();
for account in all_accounts {
	if !accounts_set.insert(account) {
		self.env().revert(Error::IllegalAccounts);
	}
}
```

So long as the accounts are unique, they can then be set to their respective contract objects:

```rust
self.arbiter.set(arbiter);
self.depositor.set(depositor);
self.beneficiary.set(beneficiary);
```

Now set the initial escrow contract values:

```rust
self.good_provided.set(false);
self.deposit_amount.set(deposit_amount);
self.balance.set(0.into());
```

### Caller Assertion

An efficient way to ensure that the proper account is calling a given entrypoint is to create a function that reverts execution if an unallowed user attempts to invoke it. For this, you can create a function `assert_caller` that makes use of the custom Odra type `Account` defined in [Custom Types](#Custom-Types):

```rust
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
```

Note the lack of the `pub` keyword in the function declaration, which prevents Odra from assigning this an entrypoint.

### Deposit Entrypoint

The `deposit` entrypoint is to be called by the depositor and initiates the escrow agreement. It should be a `payable` entrypoint so as to accept funds:

```rust
#[odra(payable)]
pub fn deposit(&mut self) {
  
}
```

A mutable reference to `self` is necessary to mutate contract values, in this case `balance`.

Within the entrypoint, first assert that the correct caller is invoking the entrypoint:

```rust
self.assert_caller(Account::Depositor);
```

Then check that the balance is equal to `0`. If it isn't, funds have already been deposited:

```rust
if self.balance.get().unwrap() != U512::from(0) {
	self.env().revert(Error::FundsAlreadyDeposited);
}
```

Next, ensure that the correct CSPR value is provided. If the value is incorrect, revert with `IncorrectDepositAmount`:

```rust
if self.env().attached_value() != self.deposit_amount.get().unwrap() {
	self.env().revert(Error::IncorrectDepositAmount);
}
```

Now you can add the value of the attached CSPR to the contract's `balance` object:

```rust
self.balance.add(self.env().attached_value());
```

Finally, emit the `DepositMade` event:

```rust
self.env().emit_event(DepositMade {
	depositor: self.env().caller(),
	amount: self.env().attached_value(),
});
```

### Provided Good Entrypoint

This entrypoint is called by the beneficiary and affirms that the beneficiary has provided the off-chain good to the arbiter:

```rust
pub fn provided_good(&mut self) {
	self.assert_caller(Account::Beneficiary);
	self.good_provided.set(true);
	self.env().emit_event(GoodProvided {
		beneficiary: self.env().caller(),
	});
}
```

This function simply asserts that the beneficiary is the caller, sets `good_provided` to `true`, and emits the `GoodProvided` event.

### Settle Entrypoint

The `settle` entrypoint is open to the arbiter and settles the escrow, sending the funds to the beneficiary, setting the contract balance to `0`, and reverting `good_provided` to `false`.

Create the `settle` entrypoint:

```rust
pub fn settle(&mut self) {
        
}
```

Within the function, first assert that the arbiter is the account invoking the entrypoint:

```rust
self.assert_caller(Account::Arbiter);
```

Then make sure that the good has been provided:

```rust
if !self.good_provided.get().unwrap() {
	self.env().revert(Error::GoodNotProvided);
}
```

Next, check that the balance is equal to the deposit amount. If it isn't, the funds have yet to be deposited:

```rust
if self.balance.get().unwrap() != self.deposit_amount.get().unwrap() {
	self.env().revert(Error::FundsNotDeposited);
}
```

Now get the current balance of the contract, as in good practice, you'll want to set the contract balance to `0` before sending funds out:

```rust
let contract_balance = self.balance.get_or_default();
```

Set the contract balance to `0`, and set `good_provided` to `false`:

```rust
self.balance.set(0.into());
self.good_provided.set(false);
```

Now you can set up a token transfer to the beneficiary:

```rust
self.env().transfer_tokens(&self.beneficiary.get().unwrap(), &contract_balance);
```

Finally, emit the `EscrowSettled` event:

```rust
self.env().emit_event(EscrowSettled {
	depositor: self.depositor.get().unwrap(),
	beneficiary: self.beneficiary.get().unwrap(),
	amount_paid: contract_balance,
});
```

### Reject Entrypoint

In some cases, the symmetric escrow parties may disagree on terms, or wish to cancel the agreement. For this case, the `reject` entrypoint is necessary. It may be invoked by the arbiter and returns the funds to the depositor, as well as sets the `good_provided` boolean to `false`.

Create the entrypoint:

```rust
pub fn reject(&mut self) {

}
```

First, ensure that the arbiter is the account invoking the entrypoint:

```rust
self.assert_caller(Account::Arbiter);
```

Then, like in `settle`, save the current contract balance:

```rust
let contract_balance = self.balance.get_or_default();
```

Set the contract balance to `0` and `good_provided` to false:

```rust
self.balance.set(0.into());
self.good_provided.set(false);
```

Send the funds back to the depositor:

```rust
self.env().transfer_tokens(&self.depositor.get().unwrap(), &contract_balance);
```

Finally, emit the `EscrowRejected` event:

```rust
self.env().emit_event(EscrowRejected {
	depositor: self.depositor.get().unwrap(),
	beneficiary: self.beneficiary.get().unwrap(),
	amount_returned: contract_balance,
});
```

## Testing

To test the escrow contract's functionality before deploying it to a production environment, you can write standard Rust integration tests and test them against a variety of backends, the simplest being Odra's mock VM.

### Preparation

To get started writing tests, create a new module `tests` and annotate it with the `[cfg(test)]` attribute:

```rust
#[cfg(test)]
mod tests {
  
}
```

At the top of the `tests` module, import the objects from the Escrow module:

```rust
use super::*;
```

Also import the `HostRef` and `Deployer` trait to expose the contract functions and the `deploy` function on the contract, respectively:

```rust
use odra::host::{Deployer, HostRef};
```

### Successful Escrow Arbitration Test

This tutorial will contain a single test for brevity's sake, but additional tests may be written to ensure proper functionality given various circumstances.

The following test will test a successful escrow arbitration. It will deploy the contract, assigning a depositor, beneficiary, and arbiter, then fulfill the requirements of each party, asserting the success of each aspect.

Create a new test function `successful_escrow`:

```rust
#[test]
fn successful_escrow() {
  
}
```

Create a new instance of `HostEnv`, which provides access to the contracts environment context:

```rust
let env = odra_test::env();
```

Define each escrow party as a different test account:

```rust
let arbiter = env.get_account(1);
let depositor = env.get_account(2);
let beneficiary = env.get_account(3);
```

Define the deposit amount. In this example it is 10 CSPR, or 10 billion motes. The number will default to type `i32`, so you must specify a larger integer type as 10 billion is larger than `i32::MAX`. This example uses `u64`, which is then converted to an Odra `U512`:

```rust
let deposit_amount = U512::from(10_000_000_000u64);
```

Now instantiate `EscrowInitArgs`, providing the arbiter, depositor, and beneficiary addresses, as well as the deposit amount:

```rust
let init_args = EscrowInitArgs {
	arbiter: arbiter,
	depositor: depositor,
	beneficiary: beneficiary,
	deposit_amount: deposit_amount,
};
```

Now the contract can be deployed. The `EscrowHostRef::deploy` function accepts a reference to the environment and the initialization arguments:

```rust
let mut contract = EscrowHostRef::deploy(&env, init_args);
```

A mutable instance is required as entrypoints we call will make adjustments to values within `contract`.

*Note: The deployer in this case is test account 0, as `env.set_caller()` was never called with another account. Remember that test accounts 1, 2, and 3 are reserved for the arbiter, depositor, and beneficiary respectively*

Get the initial balances of the depositor and beneficiary, so we can test their expected balances against their originals later in the test:

```rust
let depositor_initial_balance = env.balance_of(&depositor);
let beneficiary_initial_balance = env.balance_of(&beneficiary);
```

Set the caller to `depositor`:

```rust
env.set_caller(depositor);
```

And attempt to call the `deposit` entrypoint, expecting success:

```rust
contract
	.with_tokens(deposit_amount)
	.try_deposit()
	.expect("Deposit should be successful");
```

*Note: `try_{{entrypoint}}` is created by `EscrowHostRef` for every entrypoint, which returns a `Result<(), OdraError>` allowing you to test proper and improper execution.*

Check that the `DepositMade` event was emitted with the correct values:

```rust
env.emitted_event(
	contract.address(),
	&DepositMade {
		depositor: depositor,
		amount: deposit_amount,
	},
);
```

Set the caller to `beneficiary`, and try calling `provided_good`, expecting success:

```rust
env.set_caller(beneficiary);
contract
	.try_provided_good()
	.expect("Beneficiary should be able to provide good");
```

Ensure that the `GoodProvided` event was emitted with the proper value:

```rust
env.emitted_event(
	contract.address(),
	&GoodProvided {
		beneficiary: beneficiary,
	},
);
```

Set the caller to the arbiter and attempt to call the `settle` entrypoint, expecting success:

```rust
env.set_caller(arbiter);
contract
	.try_settle()
	.expect("Arbiter should be able to settle escrow");
```

Ensure that the `EscrowSettled` event was emitted with the proper values:

```rust
env.emitted_event(
	contract.address(),
	&EscrowSettled {
		depositor: depositor,
		beneficiary: beneficiary,
		amount_paid: deposit_amount,
	},
);
```

Finally, assert that the balances of the depositor and beneficiary are as they should be. The beneficiary should have its initial balance plus `deposit_amount`, and the depositor its initial balance minus `deposit_amount`:

```rust
assert_eq!(
	env.balance_of(&beneficiary),
	beneficiary_initial_balance + deposit_amount
);

assert_eq!(
	env.balance_of(&depositor),
	depositor_initial_balance - deposit_amount
);
```

## Conclusion

In this tutorial you learned how to write a basic escrow smart contract with two parties and an arbiter. This contract can be expanded to support more specific situations and circumstances.
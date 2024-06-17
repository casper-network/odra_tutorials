# Donation Contract with Odra

## Introduction

In this tutorial, you will learn how to create a donation contract using Odra. This smart contract can accept funds from anyone, and can be withdrawn from by the original deployer. The donation contract will introduce two new concepts in Odra development, not covered in the previous tutorials: payable entrypoints and event emission.

## Preparation

Initialize a new Odra project:

```bash
cargo odra new --name donation -t blank
```

Open _donation/src/lib.rs_ in an editor.

Begin the contract by importing necessary dependencies. Start with the Odra [prelude](https://docs.rs/odra/latest/odra/prelude) which contains a set of modules, macros, structs, enums and traits that are commonly used in smart contract development with Odra:

```rust
use odra::prelude::*;
```

Since the contract will interact with CSPR, which is represented as a 512 bit unsigned integer, import `U512`:

```rust
use odra::casper_types::U512;
```

Lastly, import the following Odra datatypes:

```rust
use odra::{Address, Var};
```

## Event Declaration

As mentioned in the introduction, this smart contract will emit events. It will emit an event upon the reception of a donation, and at the request of a withdrawal.

Before emitting these events, they need to be defined. Begin with the `DonationReceived` event:

```rust
#[odra::event]
pub struct DonationReceived {
    pub donor: Address,
    pub amount: U512,
}
```

Notice that the event is represented as a public struct, annotated with the `#[odra::event]` attribute. The event could also derive `PartialEq`, `Eq`, and `Debug`, which is in many cases useful for writing tests, but unnecessary in this case.

The event contains two parameters, `donor` and `amount`, which specify the donor's public key and the amount they donated, respectively.

Next, define the `Withdrawal` event, which only consists of one parameter, `amount`, as the contract deployer is always the withdrawer:

```rust
#[odra::event]
pub struct Withdrawal {
    pub amount: U512,
}
```

## Errors

It is also useful to define user errors that can be thrown if unexpected behavior is encountered. Do this by defining a new public `enum` `Error`, and annotate it with the `#[odra::error]` attribute:

```rust
#[odra::error]
pub enum Error {
    UnauthorizedToWithdraw = 0,
    CouldntGetBalance = 1,
}
```

In this case, two errors are defined, `UnauthorizedToWithdraw`, which the contract will throw if a non-owner attempts to withdraw the funds, and `CouldntGetBalance` if the balance of the contract is unobtainable.

## Interface

You can now create an Odra module that will expose the variables used in the smart contract:
To register errors and events , we need to add their inner attributes to the struct's #[odra::module] attribute and pass the error and event types as the values. The registered errors and events will be present in the contract schema.

```rust
#[odra::module(
    events = [DonationReceived, Withdrawal],
    errors = Error
)]
pub struct Donation {
    balance: Var<U512>,
    owner: Var<Address>,
}
```

For the donation contract, only `balance` and `owner` (the owner of the contract) are needed.

## Contract Implementation

The smart contract can now be implemented. Start by using `impl` to implement the `Donation` module defined above:

```rust
#[odra::module]
impl Donation {

}
```

### Constructor

Begin the implementation with the contract's constructor, which must be named `init`. It accepts one argument, `&mut self`, a mutable reference to `self`, allowing access to Odra methods and the two variables `balance` and `owner` defined previously. A specifically mutable reference is needed to set values on the two variables:

```rust
pub fn init(&mut self) {
	self.owner.set(self.env().caller());
	self.balance.set(U512::from(0));
}
```

In this case, `owner` is set to the contract caller, which is the deployer, and `balance` is set to a `U512` representation of `0`.

### Donate Entrypoint

Next, create the `donate` entrypoint, which is expected to be payable, so should be annotated with the `#[odra(payable)]` attribute:

```rust
#[odra(payable)]
pub fn donate(&mut self) {

}
```

To get the payment sent by the caller, use the following:

```rust
let amount: U512 = self.env().attached_value();
```

Now add this to the contract's balance:

```rust
self.balance.add(amount);
```

At this point, the contract has accepted the funds, and updated its balance accordingly, so this entrypoint can conclude by emitting the `DonationReceived` event:

```rust
self.env().emit_event(DonationReceived {
	donor: self.env().caller(),
	amount,
});
```

### Withdraw Entrypoint

Create a new entrypoint, this time non-payable, `withdraw`:

```rust
pub fn withdraw(&mut self) {

}
```

The entrypoint should start with a verification that the caller is the owner. Do this by obtaining the caller, and comparing it to the stored `owner` in the contract:

```rust
let caller = self.env().caller();
if self.owner.get().unwrap() != caller {
	self.env().revert(Error::UnauthorizedToWithdraw);
}
```

If the caller is not the owner, revert with `UnauthorizedToWithdraw`.

To keep things simple, the `withdraw` entrypoint will remove all funds from the contract. For this reason, its balance can simply be set back to `0`, but before doing so, a note needs to be made of the current balance, so it can be used to specify how much to be sent to the caller, and used for event emission:

```rust
let current_balance: U512 = self.balance.get_or_default();
self.balance.set(U512::from(0));
```

Now, transfer the tokens:

```rust
self.env().transfer_tokens(&caller, &current_balance);
```

And emit the `Withdrawal` event:

```rust
self.env().emit_event(Withdrawal {
	amount: current_balance,
});
```

### Get Balance Entrypoint

The contract is effectively complete, but for purposes of testing, and external contract inquiries, it is important to implement another simple entrypoint that returns the current balance of the contract:

```rust
pub fn get_balance(self) -> U512 {
	self.balance.get_or_revert_with(Error::CouldntGetBalance)
}
```

This entrypoint simply obtains the balance and returns it to the caller, and if fails, reverts with `CouldntGetBalance`. Omitting a semicolon at the end of the statement returns the value produced, removing the need for a `return` statement.

## Calling Payable Entrypoint from Client

When invoking a payable entrypoint from a client, such as an SDK, you'll need to deploy a piece of session code that acquires the contract's purse and fills it with the amount you're providing. This proxy WASM can be found [here](https://github.com/odradev/odra/tree/release/1.0.0/odra-casper/proxy-caller).

You'll also need to provide the following runtime arguments:

```javascript
const contractPackageHashBytes = new CLByteArray(decodeBase16(TARGET_CONTRACT_PKG_HASH_STR));

const args_bytes: Uint8Array = RuntimeArgs.fromMap({
   // Arguments forwarded to the target contract
}).toBytes().unwrap();
const serialized_args = new CLList(Array.from(args_bytes).map(value => new CLU8(value)));

const args = RuntimeArgs.fromMap({
  amount: CLValueBuilder.u512(csprToMotes(50)), // Amount paid to target contract entrypoint
	attached_value: CLValueBuilder.u512(csprToMotes(50)), // Should be the same as amount
	entry_point: CLValueBuilder.string('entrypoint'), // Target contract entrypoint
	contract_package_hash: contractPackageHashBytes, // Target contract package hash
	args: serialized_args // Runtime Arguments passed to the contract
});

// Install the Proxy WASM
```

## Testing

With the contract now complete, tests can be written. Start by opening a new module `tests` annotated with the Rust attribute `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {

}
```

Begin the module by importing the required dependencies:

```rust
use super::*;
use odra::host::{Deployer, HostRef, NoArgs};
```

`use super::*;` is used to import the `Donation` contract, and the imports from the `host` module consist of:

- `Deployer`: A trait that exposes the `deploy` function for deploying the contract to the mock VM.
- `HostRef`: A trait that exposes references to the host, allowing for the invocation of `Donation` entrypoints.
- `NoArgs`: A struct that can be used in place of initialization arguments for deployment of the contract. Used because the `Donation` contract doesn't require constructor arguments.

### Donation Test

Create the first test, `donate`, annotated with the `#[test]` attribute:

```rust
#[test]
fn donate() {

}
```

Begin the test with an instantiation of `HostEnv`, which provides access to a variety of objects necessary for interaction with the backend:

```rust
let env = odra_test::env();
```

Deploy the contract, returning to a new variable the `DonationHostRef`:

```rust
let contract = DonationHostRef::deploy(&env, NoArgs);
```

Specify the donation amount, in this case 1 CSPR or 1 billion motes, of type `U512`:

```rust
let donation_amount = U512::from(1_000_000_000);
```

Get the initial balance of the default testing account:

```rust
let caller_initial_balance = env.balance_of(&env.get_account(0));
```

Now call the `donate` entrypoint, but do so using the `try_donate` function that is created by the `DonationHostRef`. Before calling the function, use the `with_tokens(amount)` function to assign a payment value to the call. Laslty, use `expect` to expect success, otherwise throwing an error with the message provided:

```rust
contract
	.with_tokens(donation_amount)
	.try_donate()
	.expect("Donation should be successful");
```

_Note: The `HostRef` creates `try_`functions for each entrypoint, which return a`Result<(T), OdraError>`.\_

Assert that the new value of the calling account is `donation_amount` less than its original balance:

```rust
assert_eq!(
	env.balance_of(&env.get_account(0)),
	caller_initial_balance - donation_amount
);
```

And finally, listen for the `DonationReceived` event emission:

```rust
env.emitted_event(
	contract.address(),
	&DonationReceived {
		donor: env.get_account(0),
		amount: donation_amount,
	},
);
```

### Withdrawal Test

For the next and final test of this tutorial, create the test `withdraw`:

```rust
#[test]
fn withdraw() {

}
```

Like in the donation test, instantiate a `HostEnv` instance:

```rust
let env = odra_test::env();
```

And deploy the contract, which returns a `DonationHostRef`. This time create the `contract` as mutable, which is required as it will call `get_balance` which returns a value:

```rust
let mut contract = DonationHostRef::deploy(&env, NoArgs);
```

Since at the beginning of each test, the contract is deployed anew, it will always start with a balance of `0`, so a donation must be made before a (meaningful) withdrawal can be done.

_Note: Technically a withdrawal of 0 tokens could be tested._

Specify the donation amount:

```rust
let donation_amount = U512::from(1_000_000_000);
```

Perform the donation:

```rust
contract
	.with_tokens(donation_amount)
	.try_donate()
	.expect("Donation should be successful");
```

Get the balance of the account after the donation, but before the withdrawal:

```rust
let caller_initial_balance = env.balance_of(&env.get_account(0));
```

And get the balance of the contract, which should be `donation_amount`:

```rust
let initial_contract_balance = contract
	.try_get_balance()
	.expect("Balance should be obtainable");
```

Change the caller of the contract to test withdrawing from a non-owner account, which should fail:

```rust
contract
	.try_withdraw()
	.expect_err("Withdrawal from non-owner should fail");
```

As shown, you can use `expect_err` to expect that an entrypoint invocation will fail.

Set the caller back to the owner:

```rust
env.set_caller(env.get_account(0));
```

And attempt the withdrawal, this time expecting success:

```rust
contract
	.try_withdraw()
	.expect("Withdrawal from owner should succeed");
```

The balance of the default account should now be the balance of the account after donation, plus the contract balance after donation. Assert that these values are equal:

```rust
assert_eq!(
	env.balance_of(&env.get_account(0)),
	caller_initial_balance + initial_contract_balance
);
```

The balance of the contract itself should now be `0`. Get the balance and assert that it is `0`:

```rust
let new_contract_balance = contract
	.try_get_balance()
	.expect("Balance should be obtainable");
assert_eq!(new_contract_balance, U512::from(0));
```

Lastly, check that the event was emitted with the proper amount attached:

```rust
env.emitted_event(
	contract.address(),
	&Withdrawal {
		amount: initial_contract_balance,
	},
);
```

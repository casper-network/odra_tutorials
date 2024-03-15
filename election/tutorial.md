# Election Contract with Odra

## Introduction

This tutorial will guide you through the creation of a voting smart contract using Odra. The contract will be built assuming the following principles:

* The deployer can specify candidates and a final voting time in the constructor.
* The final voting time is denominated in block height.
* The deployer cannot modify the candidates or end time after deployment.
* Any account, besides the deployer, may make one vote for any candidate they please.

The contract could be extended to allow for modifications to candidates, end time, and voting capabilities, but this tutorial avoids these functionalities in the interest of simplicity.

## Preparation

Initialize a new Odra project:

```bash
cargo odra new --name election -t blank
```

*Note: The `-t blank` flag will create the contract as a blank template.*

Open *election/src/lib.rs* in an editor.

Begin the contract by importing the Odra datatypes `Var`, `Mapping`, and `Address`, as well as the `Vec` class from `alloc::vec`:

```rust
use odra::{Var, Mapping, Address};
use alloc::vec::Vec;
```

`Var` is Odra's most basic data structure that supports reading and writing to global state using built-in methods. `Mapping` has similar features, but is for key-value pairs. An `Address`, on the other hand, is an enum representing either an `AccountHash` or `ContractPackageHash` from [casper_types](https://docs.rs/casper-types/latest/casper_types/).
`Vec` is a well-known Rust struct representing a contiguous growable array type; we can wrap `Vec` with `Var`, to be able to save and read `Vec`s to and from global state.

## Interfacing

The module definition for the Election contract can be defined next, which needs to contain all of the objects to be stored by the contract:

```rust
#[odra::module]
pub struct Election {
    end_block: Var<u64>,
    candidates: Var<Vec<String>>,
    candidate_votes: Mapping<String, u32>,
    voters: Mapping<Address, bool>
}
```

In this implementation, the objects reference the following:

* `end_block`

  The final block in which voting is permitted

* `candidates`

  A vector of candidate names. For this example it will be assumed that there are no duplicates, however this should be validated in a production environment.

* `candidate_votes`

  A dictionary of candidate to vote count; a running tally of votes accrued by each candidate.

* `voters`

  A dictionary of accounts that have voted. If the boolean `true` is present under an `Address`, that account has already voted and may not vote again.

## User Errors

Preparing custom user errors is recommended and can provide insight when debugging or experiencing issues. Odra has an enum `OdraError` that can be derived to create custom errors. For this example, define the following errors:

```rust
#[derive(OdraError)]
pub enum Error {
    VotingEnded = 0,
    VoterAlreadyVoted = 1,
    ProbablyBecauseTheValueIsEmpty = 2
}
```

## Contract Implementation

To begin writing the smart contract's functionality, implement the `Election` module, marking the implementation with the `#[odra::module]` attribute:

```rust
#[odra::module]
impl Election {
	 
}
```

### Constructor

Define the constructor, which must be named `init`:

```rust
pub fn init(&mut self, end_block: u64, candidates: Vec<String>) {
        
}
```

The `&mut self` parameter allows us to access and mutate `self`, while the `end_block` and `candidates` parameters allow us to pass in values upon deployment.

Within the constructor, start by setting the end block to that passed in as an argument:

```rust
self.end_block.set(end_block);
```

Then iterate over `candidates` and use each candidate string to prepare the `candidate_votes` dictionary:

```rust
for candidate in candidates.iter() {
	candidate_votes.set(&candidate, 0u32);
}
```

### Vote Entrypoint

Create a new function `vote` to expose an entrypoint of the same name:

```rust
pub fn vote(&mut self, candidate: String) {

}
```

The only parameter (besides a mutable reference to `self`) is `candidate`, which denotes which candidate the caller would like to vote for.

Start off the function by checking if the current block time is greater than `end_block`. If it is, revert with the error `VotingEnded`:

```rust
if self.env().get_block_time() > self.end_block.get_or_default() {
	self.env().revert(Error::VotingEnded);
}
```

Next, obtain the caller's `Address` as it will be used to determine if the calling account has already voted:

```rust
let caller: Address = self.env().caller();
```

Now prepare a `match` expression to get the dictionary value at the key `caller`:

```rust
match self.voters.get(&caller) {
	Some(_) => self.env().revert(Error::VoterAlreadyVoted),
	None => {}
}
```

In this case, if a value exists, then it is known that the account has already voted, and execution can be reverted with `VoterAlreadyVoted`.
If no value exists, execution may continue.

Assuming execution isn't reverted due to the result of the `match` expression, the only thing left to do is record the user's vote and register them in `voters` as having voted.

Start by getting the selected candidate's current vote count, reverting if the candidate doesn't exist:

```rust
let candidate_vote_count: u32 = self
	.candidate_votes
	.get(&candidate)
	.unwrap_or_revert_with(&self.env(), Error::CandidateDoesntExist);
```

Then write this value, plus 1, to `candidate_votes`:

```rust
self.candidate_votes.set(&candidate, candidate_vote_count + 1);
```

Finally, add the calling account's `Address` to `voters` to restrict the user from voting again:

```rust
self.voters.set(&caller, true)
```

### `get_candidate_votes` Entrypoint

In order to obtain the number of votes a candidate has received, open a new public entrypoint `get_candidate_votes` that accepts a candidate and returns its vote-count:

```rust
pub fn get_candidate_votes(&self, candidate: &String) -> u32 {
	self.candidate_votes.get_or_default(candidate)
}
```

## Testing

Writing tests in Odra is familiar given it's compliance with standard Rust unit and integration tests.
To get started, create a new module, annotated with the `#[cfg(test)]` attribute:

```rust
#[cfg(test)]
mod tests {
  
}
```

Begin the module by importing the host reference and initial arguments types, which are generated by Odra. These types follow the naming structure `{{ModuleName}}HostRef` and `{{ModuleName}}InitArgs`. For this example, the types are `ElectionHostRef` and `ElectionInitArgs`:

```rust
use super::{ElectionHostRef, ElectionInitArgs};
```

The `ElectionHostRef` type is a reference to the smart contract used to interact with its entrypoints, and implements the [`HostRef`](https://docs.rs/odra/0.8.0/odra/host/trait.HostRef.html) trait.

`ElectionInitArgs` is a struct that can be used to initialize the contract with the proper runtime arguments, and implements the [`InitArgs`](https://docs.rs/odra/0.8.0/odra/host/trait.InitArgs.html) trait.

The next import required is the `Deployer` trait which is implemented by `ElectionHostRef` and provides the `deploy` method that will be used to deploy the contract for testing:

```rust
use odra::host::Deployer;
```

Finally, import the Odra prelude which contains a variety of functions that can be used in development:

```rust
use odra::prelude::*
```

Now you can create a new test by defining a new function annotated with the `#[test]` attribute:

```rust
#[test]
fn vote() {
  
}
```

As the function will need access to environment information during testing, you can define a new `HostEnv` instance that will provide access to objects such as the caller, purse balance, and system variables such as block time:

```rust
let test_env: HostEnv = odra_test::env();
```

In order to deploy the contract to the test environment, initial runtime arguments are needed, so create a new instance of `ElectionInitArgs` and specify the arguments:

```rust
let init_args = ElectionInitArgs {
	end_block: 1,
	candidates: vec!["Alice".to_string(), "Bob".to_string()],
};
```

In this example, the final block for which voting is valid is `1`, and since the testing environment starts at block `0`, this will provide ample time to vote.

`candidates` consists of two strings in this case, `"Alice"` and `"Bob"`, providing two candidates to vote for.

Now the contract can be deployed. Create a new mutable instance of `ElectionHostRef` by executing the `deploy` method:

```rust
let mut contract = ElectionHostRef::deploy(&test_env, init_args);
```

You can now invoke entrypoints on the `contract` object. Call the `vote` entrypoint, providing one of the two candidates:

```rust
contract.vote("Alice".to_string());
```

As part of the test, validate that the candidate now has a single vote:

```rust
assert_eq!(contract.get_candidate_votes("Alice".to_string()), 1);
```


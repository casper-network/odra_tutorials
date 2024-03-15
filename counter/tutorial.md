# Odra Counter Tutorial

This guide will demonstrate the process of creating a counter smart contract using Odra. Initially, it will cover the conventional approach of utilizing [casper_contract](https://docs.rs/casper-contract/latest/casper_contract/), followed by an explanation of how the Odra framework simplifies this process.

Start by opening the [Counter v1 example](https://github.com/casper-ecosystem/counter/blob/master/contract-v1/src/main.rs) on the casper-ecosystem github profile. This smart contract has three entrypoints:  `call`, `counter_inc`, and `counter_get`. Let's look at the first two of these and what they do. As you may know, the `call` entrypoint is called once when the contract is deployed, and can never be invoked again. In this case, `call` starts by initializing the count, which can be incremented by `counter_inc`:

```rust
// Initialize the count to 0, locally.
let count_start = storage::new_uref(0_i32);
```

It then creates a [Named Keys](https://docs.casper.network/concepts/glossary/N/#named-keys) object, and adds the count to it:

```rust
// In the named keys of the contract, add a key for the count.
let mut counter_named_keys = NamedKeys::new();
let key_name = String::from(COUNT_KEY);
counter_named_keys.insert(key_name, count_start.into());
```

As mentioned above, the Counter contract has another two entrypoints, `counter_inc` and `counter_get`, but for these to be callable, they need to be exposed, which is done next in the `call` function:

```rust
let mut counter_entry_points = EntryPoints::new();

counter_entry_points.add_entry_point(EntryPoint::new(
	ENTRY_POINT_COUNTER_GET,
	Vec::new(),
	CLType::I32,
	EntryPointAccess::Public,
	EntryPointType::Contract,
));

counter_entry_points.add_entry_point(EntryPoint::new(
	ENTRY_POINT_COUNTER_INC,
	Vec::new(),
	CLType::Unit,
	EntryPointAccess::Public,
	EntryPointType::Contract,
));
```

Finally, a new contract is officially created in global state, and the contract hash is stored in the deploying account's named keys:

```rust
// Create a new contract package that can be upgraded.
let (stored_contract_hash, contract_version) = storage::new_contract(
	counter_entry_points,
	Some(counter_named_keys),
	Some(CONTRACT_PACKAGE_NAME.to_string()),
	Some(CONTRACT_ACCESS_UREF.to_string()),
);

// Store the contract version in the context's named keys.
let version_uref = storage::new_uref(contract_version);
runtime::put_key(CONTRACT_VERSION_KEY, version_uref.into());

// Create a named key for the contract hash.
runtime::put_key(CONTRACT_KEY, stored_contract_hash.into());
```

Now let's look at `counter_inc`, which is responsible for incrementing the counter. All it does is get the current count from the `COUNT_KEY` named key, converts it to a URef, and adds one to it.

```rust
#[no_mangle]
pub extern "C" fn counter_inc() {
    let uref: URef = runtime::get_key(COUNT_KEY)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    storage::add(uref, 1); // Increment the count by 1.
}
```

This smart contract has a very limited functionality of incrementing a single arbitrary integer, yet it still requires close to one hundred lines of code to implement. Let's look at how the Odra framework can be used to make the Counter implementation much simpler.

Create a new Odra project by opening a terminal and running:

```bash
cargo odra new --name counter -t blank
```

*Note: The `-t blank` flag will create the contract as a blank template.*

Opening *counter/src/lib.rs* will display a blank file. [// n otherwise blank file with the `#![no_std]` macro on the first line.]

You can use the Odra `Variable` module to easily store and read global state values. This will be the only import of the file:

```rust
use odra::Variable;
```

All smart contracts are modules in Odra, so we need to define a new module `Counter`, listing all the fields in our contract:

```rust
#[odra::module]
pub struct Counter {
    count: Variable<u32>,
}
```

This acts like an interface, and allows our Counter contract to be implemented in other contracts.

Now you can implement the `Counter` module. Do so by using the Rust `impl` keyword and denoting the implementation as a `#[odra::module]` to ensure entrypoints are automatically generated for your functions:

```rust
#[odra::module]
impl Counter {
  
}
```

The first function to be implemented is the constructor, which like the `call` function in a traditional Casper contract, is only ever called once upon deployment. Constructors in Odra are marked by the `#[odra(init)]` macro. In this function, set the initial value of the  `count` variable to 0:

```rust
#[odra(init)]
pub fn init(&mut self) {
	self.count.set(0);
}
```

All that's left is the increment function, which will be named `counter_inc` for consistency. This function will contain a single line, that will get the current count, and set it to that number, plus one:

```rust 
pub fn counter_inc(&mut self) {
	self.count.set(self.count.get_or_default() + 1);
}
```

The `get_or_default` method is used here to avoid having to unwrap an `Option` value. You could instead use the `get` method if you prefer to perform the `Option` unwrapping.

Congratulations! You've just written your first smart contract with the Odra framework.
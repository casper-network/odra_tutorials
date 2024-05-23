# Zero to Hero with NFTs: Part 2 - Batch Minting with Nested CEP-78 Module

In this second part of our NFT series, we'll expand our existing NFT contract to include batch minting. This feature allows the creation of multiple NFTs within a single transaction, offering efficiency and cost-effectiveness for large NFT collections.

## Why Batch Minting?

Minting NFTs individually can be costly and time-consuming, especially for large collections. Batch minting offers a more efficient solution, reducing gas fees and simplyfing the process.

## Prerequisites

- Make sure you've completed Part 1 of the tutorial, where we created an Odra project and deployed a basic CEP-78 NFT contract.

## Extending the Contract

We'll work within the existing `lib.rs` file to add the batch minting capability.

**1. Imports and Type Definitions**

```rust
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;
use odra::{args::Maybe, module::SubModule, prelude::*, Address};
use odra_modules::cep78::{
    modalities::{MetadataMutability, NFTIdentifierMode, NFTKind, NFTMetadataKind, OwnershipMode},
    token::Cep78,
};

pub type MintReceipt = (String, Address, String);
pub type TransferReceipt = (String, Address);
```

* **Functionality:** This section imports necessary modules and defines types used by the CEP-78 standard. 
* **Key Points:**
    - `odra` module provides core functionalities.
    - `cep78` module includes the CEP-78 NFT standard implementation.
    - `MintReceipt` and `TransferReceipt` represent the results of minting and transferring NFTs.

**2. Contract Structure**

```rust
#[odra::module]
pub struct ExtendedCEP78 {
    cep78: SubModule<Cep78>,  
}
```

* **Functionality:** Defines the main contract structure `ExtendedCEP78`, which embeds the `Cep78` contract as a submodule.
* **Key Points:**
    - `#[odra::module]` marks the structure as an Odra contract.
    - `SubModule<Cep78>`  the only field is the cep78 contract.

**3. Initialization (`init`)**

```rust
#[odra::module]
impl ExtendedCEP78 {
    pub fn init(
        &mut self,
        collection_name: String,
        collection_symbol: String,
        total_token_supply: u64,
        nft_kind: NFTKind,
        receipt_name: String,
    ) {
        self.cep78.init(
            collection_name,
            collection_symbol,
            total_token_supply,
            OwnershipMode::Transferable,
            nft_kind,
            NFTIdentifierMode::Ordinal,
            NFTMetadataKind::CEP78,
            MetadataMutability::Immutable,
            receipt_name,
            Maybe::Some(true),
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
        );
    }
    // ... rest of the implementation
}
```

* **Functionality:**  Acts as the contract's constructor, initializing the embedded `cep78` module with provided parameters.
* **Key Points:**
    - The `init` function sets up the CEP-78 NFT collection with properties like name, symbol, and total supply.
    - Some arguments are hardcoded for simplicity in this tutorial.


**4. Delegating Functions**

```rust
// ... (inside the impl ExtendedCEP78 block)

delegate! {
        to self.cep78 {
            fn mint(
                &mut self,
                token_owner: Address,
                token_meta_data: String,
                token_hash: Maybe<String>
            ) -> MintReceipt;
            fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);
            fn transfer(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>,
                source_key: Address,
                target_key: Address
            ) -> TransferReceipt;
            fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>);
            fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);
            fn set_approval_for_all(&mut self, approve_all: bool, operator: Address);
            fn is_approved_for_all(&mut self, token_owner: Address, operator: Address) -> bool;
            fn owner_of(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address;
            fn get_approved(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>
            ) -> Option<Address>;
            fn metadata(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> String;
            fn set_token_metadata(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>,
                token_meta_data: String
            );
            fn balance_of(&mut self, token_owner: Address) -> u64;
            fn register_owner(&mut self, token_owner: Maybe<Address>) -> String;
        }
    }
}
```

* **Functionality:** Exposes specific CEP-78 functions (like `mint`, `burn`, `transfer`) as entrypoints in our extended contract.
* **Key Points:**
    - Uses Odra's `delegate!` macro to add CEP-78 functions.
    - Allows our contract to use the standard NFT behaviors defined in CEP-78.

**5. Implementing Batch Minting (`batch_mint`)**
````rust
// ... (inside the impl ExtendedCEP78 block)

#[odra::entrypoint]
pub fn batch_mint(
    &mut self,
    token_owner: Address,
    token_meta_data: Vec<String>,
) -> Vec<MintReceipt> {
    let mut mint_receipts: Vec<MintReceipt> = Vec::new();
    for t in token_meta_data.iter() {
        let receipt = self.cep78.mint(token_owner, t.clone(), Maybe::None);
        mint_receipts.push(receipt);
    }
    mint_receipts
}
````


* **Functionality:**  The core function for batch minting multiple NFTs at once.
* **Key Points:**
   - Takes an owner's address and a list of metadata strings as input.
   - Iterates through the metadata, minting each NFT using the delegated `mint` function from CEP-78.
   - Returns a vector of `MintReceipt`s, confirming successful minting for each NFT.


**6. Testing `batch_mint`**

```rust
// ... (inside lib.rs, after the contract implementation)

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;
    #[test]
    fn batch_mint() {
        // Deploy the contract
        let env = odra_test::env();
        let init_args = ExtendedCEP78InitArgs {
            collection_name: "Batch Collection".to_string(),
            collection_symbol: "BC".to_string(),
            total_token_supply: 20,
            nft_kind: NFTKind::Digital,
            receipt_name: "receipt".to_string(),
        };

        let mut contract = ExtendedCEP78HostRef::deploy(&env, init_args);
        let alice = env.get_account(1);

        // Prepare metadata for batch minting
        let cep78_metadata: String = r#"{
            "name": "Batch collection",
            "token_uri": "https://www.batch-collection.io",
            "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
        }"#
        .to_string();
        let mut metadata: Vec<String> = Vec::new();
        for _ in 0..20 {
            metadata.push(cep78_metadata.clone()); // Clone to add copies
        }

        assert_eq!(contract.balance_of(alice), 0);

        // Mint 20 nfts using new entry point `batch_mint`
        contract.batch_mint(alice, metadata);
        assert_eq!(contract.balance_of(alice), 20);
    }
}
```
* **Functionality:** Unit test to verify the `batch_mint` function's correctness.
* **Key Points:**
    - Deploys a test instance of the `ExtendedCEP78` contract.
    - Creates metadata for 20 sample NFTs.
    - Checks that Alice initially has no NFTs.
    - Calls `batch_mint` to mint 20 NFTs to Alice's address.
    - Asserts that Alice now owns 20 NFTs.
    
To run the tests you need to execute the command:
```bash
cargo odra test
```

You should see output similar to this, indicating that the tests have passed:

```
running 1 test
test tests::batch_mint ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


**Summary:**

- **One Contract:** Only a single `ExtendedCEP78` contract is deployed, which now includes the `batch_mint` functionality.
- **Nested Module:** The CEP-78 standard is not a separate contract but is included within `ExtendedCEP78` as a `SubModule`.
- The `delegate!` macro exposes functions from the `cep78` submodule as entry points to your `ExtendedCEP78` contract.


**Additional Resources:**

* **Odra Documentation - Delegate Calls:** [https://odra.dev/docs/next/advanced/delegate](https://odra.dev/docs/next/advanced/delegate)
* **CEP-78 Standard:** [in-depth-guide](https://support.casperlabs.io/hc/en-gb/articles/14102164670363--In-depth-Guide-CEP-78-Understanding-Installing-and-Minting-your-own-Enhanced-NFTs)
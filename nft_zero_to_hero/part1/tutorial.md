# Zero to Hero with NFTs: Part 1

Welcome to the first part of our NFT series! In this tutorial, we'll cover the basics of NFTs, dive into the CEP-78 standard, and deploy a simple NFT contract on the Casper testnet using Odra.

## What Are NFTs?

NFTs, or Non-Fungible Tokens, are unique digital assets. Unlike cryptocurrencies (which are fungible), NFTs cannot be directly exchanged for other assets of equal value. NFTs can represent various items, including:

* **Real-world assets:**  Tickets to events, proof of ownership, etc.
* **Digital art:** Unique JPEGs, digital collectibles, etc.
* **In-game assets:** Unique items, characters, etc.

The CEP-78 standard (Casper Enhancement Proposal 78) defines how NFTs behave on the Casper Network. It introduces different modalities that dictate an NFT's functionality.

## CEP-78 Modalities

When deploying an NFT contract using CEP-78, you need to specify certain required arguments:

1. **Ownership:**
   * `Transferable:`  The NFT can be transferred between accounts.
   * `Assigned:` The initial owner is set at minting and cannot be changed.
   * `Minter:` The NFT's owner is always the minter (creator).

2. **NFT Kind:** Indicates the type of asset the NFT represents:
   * `Physical`
   * `Digital`
   * `Virtual`

3. **NFT Metadata Kind:** Dictates the metadata schema for minted NFTs:
   * `CEP78`
   * `NFT721`
   * `Raw`
   * `CustomValidated`

4. **Identifier Mode:** Governs how NFTs are identified:
   * `Ordinal:` By a sequential number
   * `Hash:` By the hash of the NFT's metadata

5. **Metadata Mutability:**
   * `Immutable:` Metadata cannot be changed after minting.
   * `Mutable:` Metadata can be updated.

For a more in-depth look at these and other modalities, refer to the CEP-78 [in-depth-guide](https://support.casperlabs.io/hc/en-gb/articles/14102164670363--In-depth-Guide-CEP-78-Understanding-Installing-and-Minting-your-own-Enhanced-NFTs)

## Deploying a CEP-78 Contract with Odra

Odra is a framework for building dApps on the Casper Network. Let's use it to deploy our NFT contract to the Casper testnet.

1. **Initialize an Odra project:**

   ```bash
   cargo odra new --name cep78 -t blank
   ```

2. **Set up LiveNet:** Create a `cep78_livenet.rs` file and add the following to your `Cargo.toml`:

   ```rust
   [dependencies]
   odra-casper-livenet-env = { version = "1.0.0", optional = true }

   [features]
   default = []
   livenet = ["odra-casper-livenet-env"]

   [[bin]]
   name = "cep78_livenet"
   path = "src/bin/cep78_livenet.rs"
   required-features = ["livenet"]
   test = false
   ```

3. **Create a `.env` file:**  Provide your network and private key information (replace placeholders with your actual values):

   ```bash
   # Path to the secret key of the account that will be used
   # to deploy the contracts.
   ODRA_CASPER_LIVENET_SECRET_KEY_PATH=.keys/secret_key.pem

   # RPC address of the node that will be used to deploy the contracts.
   ODRA_CASPER_LIVENET_NODE_ADDRESS=http://95.216.37.50:7777 # Or your node's address

   # Chain name of the network. Known values:
   # - integration-test
   # - casper-test
   ODRA_CASPER_LIVENET_CHAIN_NAME=casper-test
   ```

4. **Update `odra.toml`:** Specify the name of the contract you will be deploying:

   ```rust
   [[contracts]]
   fqn = "Cep78"
   ```
5. **Implement Contract Deployment and Interactions:** 
Add the following Rust code to your `cep78_livenet.rs` file:

```rust
//! Deploys a CEP-78 contract, mints an nft token and transfers it to another address.
use std::str::FromStr;

use odra::args::Maybe;
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader};
use odra::Address;
use odra_modules::cep78::modalities::{
    EventsMode, MetadataMutability, NFTIdentifierMode, NFTKind, NFTMetadataKind, OwnershipMode,
};
use odra_modules::cep78::token::{Cep78HostRef, Cep78InitArgs};
use odra_modules::cep78::utils::InitArgsBuilder;

const CEP78_METADATA: &str = r#"{
    "name": "John Doe",
    "token_uri": "https://www.barfoo.com",
    "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
}"#;
const CASPER_CONTRACT_ADDRESS: &str = "hash-"; // change to a deployed contract
const RECIPIENT_ADDRESS: &str = "hash-"; // change to a desired recipient address

fn main() {
    let env = odra_casper_livenet_env::env();

    // Deploy new contract.
    let mut token = deploy_contract(&env);
    println!("Token address: {}", token.address().to_string());

    // Uncomment to load existing contract.
    // let mut token = load_contract(&env, CASPER_CONTRACT_ADDRESS);
    // println!("Token name: {}", token.get_collection_name());

    env.set_gas(3_000_000_000u64);
    let owner = env.caller();
    let recipient =
        Address::from_str(RECIPIENT_ADDRESS).expect("Should be a valid recipient address");
    // casper contract may return a result or not, so deserialization may fail and it's better to use `try_transfer`/`try_mint`/`try_burn` methods
    let _ = token.try_mint(owner, CEP78_METADATA.to_string(), Maybe::None);
    println!("Owner's balance: {:?}", token.balance_of(owner));
    println!("Recipient's balance: {:?}", token.balance_of(recipient));
    let token_id = token.get_number_of_minted_tokens() - 1;
    let _ = token.try_transfer(Maybe::Some(token_id), Maybe::None, owner, recipient);

    println!("Owner's balance: {:?}", token.balance_of(owner));
    println!("Recipient's balance: {:?}", token.balance_of(recipient));
}

/// Loads a Cep78 contract.
pub fn load_contract(env: &HostEnv, address: &str) -> Cep78HostRef {
    let address = Address::from_str(address).expect("Should be a valid contract address");
    Cep78HostRef::load(env, address)
}

/// Deploys a Cep78 contract.
pub fn deploy_contract(env: &HostEnv) -> Cep78HostRef {
    let name: String = String::from("CEP-78 Example Deployment with CES");
    let symbol = String::from("CEP78-EXAMPLE-CES");
    let receipt_name = String::from("Example_NFT_Receipt");

    let init_args = InitArgsBuilder::default()
        .collection_name(name)
        .collection_symbol(symbol)
        .total_token_supply(100)
        .ownership_mode(OwnershipMode::Transferable)
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_kind(NFTKind::Digital)
        .metadata_mutability(MetadataMutability::Mutable)
        .receipt_name(receipt_name)
        .events_mode(EventsMode::CES)
        .build();

    env.set_gas(400_000_000_000u64);
    Cep78HostRef::deploy(env, init_args)
}
```

**Code Explanation:**

* **Imports:** Includes necessary modules from Odra and the CEP-78 module.
* **Constants:** Defines metadata for the NFT and placeholders for contract and recipient addresses (which you'll need to fill in).
* **`main` function:**
    - Gets the host environment (`env`).
    - Deploys the contract (`deploy_contract`) or loads an existing one (`load_contract`).
    - Sets the gas limit.
    - Gets the owner's address (`env.caller()`).
    - Mints an NFT (`token.try_mint`).
    - Transfers the NFT to another address (`token.try_transfer`).
    - Prints balances to the console.
* **`load_contract` function:** Loads a CEP-78 contract from a given address.
* **`deploy_contract` function:** Deploys a new CEP-78 contract with specified initial parameters.

6. **Build the Contract:**
   ```bash
   cargo odra build 
   ```
   This will compile your contract into Wasm bytecode.

7. **Run LiveNet and Deploy:**
   ```bash
   cargo run --bin cep78_livenet --features=livenet
   ```
   This command will execute your `cep78_livenet.rs` script, deploying the contract and interacting with it as you've defined.



Now you've successfully deployed your first CEP-78 NFT contract!


**Additional Resources:**

* **CEP-78 Standard:** [in-depth-guide](https://support.casperlabs.io/hc/en-gb/articles/14102164670363--In-depth-Guide-CEP-78-Understanding-Installing-and-Minting-your-own-Enhanced-NFTs)
* **Odra Framework:** [odra-docs](https://odra.dev/)

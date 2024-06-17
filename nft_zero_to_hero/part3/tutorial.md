## Zero to Hero NFT: Part 3 - Building an NFT Auction Contract

In this part of our series, we'll dive into creating a smart contract that enables NFT auctions on the Casper Network. Our auction contract will allow users to list their CEP-78 NFTs for sale, place bids, and transfer the NFT to the highest bidder and funds to the seller.

**How It Works**

Our auction contract leverages the CEP-78 standard for NFTs on Casper. When creating an auction, the seller first needs to "approve" the auction contract to transfer their NFT. This is done by calling the `approve` function on the NFT contract, specifying the auction contract's address. Once approved, our contract's `create_auction` function can transfer the NFT to itself, holding it in escrow until the auction ends.

After the auction duration expires, someone needs to call the `end_auction` function. This function automatically transfers the NFT to the highest bidder and the corresponding funds (in CSPR) to the seller. Losing bidders are refunded their bids immediately after being outbid, a design choice made for simplicity in this tutorial. In a production this approach is not ideal due to gas costs. Alternative apporach is to use a batch refund mechanis for gas optimisation.

### Designing the Auction Contract

#### Core Functionality

Our auction contract will implement the following core features:

1.  **Create Auction:** Sellers initiate an auction by specifying the NFT contract address, token ID, starting price, and auction duration.
2.  **Place Bid:** Bidders participate by placing bids higher than the current highest bid.
3.  **End Auction:** The auction ends after the specified duration. Calling the `end_auction` entry point is required to transfer the NFT to the highest bidder and funds to the seller.
4.  **Refunds:** Losing bidders are immediately refunded upon being outbid. This is a simplified approach for this tutorial, as frequent transactions can lead to higher gas costs.

#### Smart Contract Structure

We'll use ODRA's `Mapping` to store auction details efficiently. Each auction will be represented by a struct:

```rust
#[odra::odra_type]
struct Auction {
    seller: Address,
    nft_contract: Address,
    token_id: u64,
    starting_price: U512,
    ends_at: u64,
    highest_bidder: Option<Address>,
    highest_bid: U512,
}
```

### Implementing the Smart Contract

Let's start by defining our contract, including error types, imports, and initialization.

```rust
use odra::{
    args::Maybe,
    casper_types::{U256, U512},
    module::Module,
    Address, ContractRef, Mapping, SubModule, Var,
};
use odra_modules::cep78::token::Cep78ContractRef;
use odra_modules::{access::Ownable, security::Pauseable};

#[odra::module]
pub struct Auctions {
    // ... (rest of the contract code)
}
```

Now, let's add the functions for creating, bidding on, and ending auctions, along with the necessary error handling and logic.

```rust
// ... (contract struct and error enum)

#[odra::module]
impl Auctions {
    // ... (init)

    /// Creates a new auction for a CEP-78 NFT.
    pub fn create_auction(
        &mut self,
        nft_contract: Address,
        nft_id: u64,
        starting_price: U512,
        duration: u64,
    ) {
        self.pausable.require_not_paused(); // Ensure contract is not paused

        if duration < self.min_auction_duration.get_or_default() {
            self.env().revert(Error::InvalidAuctionDuration) // Revert if duration is too short
        }

        let seller = self.env().caller();

        // Transfer the NFT to the auction contract
        Cep78ContractRef::new(self.env(), nft_contract).transfer(
            Maybe::Some(nft_id),
            Maybe::None,
            seller,
            self.env().self_address(),
        );

        // Create and store the auction details
        let auction = Auction {
            nft_contract,
            nft_id,
            seller,
            starting_price,
            highest_bid: U512::zero(),
            highest_bidder: None,
            ends_at: self.env().get_block_time() + duration,
        };
        self.auctions
            .set(&self.auction_counter.get_or_default(), auction);
        self.auction_counter.add(U256::one()); // Increment auction counter
    }

    /// Places a bid on an active auction.
    #[odra(payable)] // Indicates this function accepts CSPR payments
    pub fn bid(&mut self, auction_id: U256) {
        self.pausable.require_not_paused();

        let bidder = self.env().caller();
        let amount = self.env().attached_value(); // Get the attached CSPR amount
        let mut auction = self.auctions.get(&auction_id).unwrap();

        // Validate bid amount
        if amount < auction.starting_price || amount < auction.highest_bid {
            self.env().revert(Error::InvalidBid);
        }

        // Check if auction is still ongoing
        if self.env().get_block_time() > auction.ends_at {
            self.env().revert(Error::AuctionHasEnded);
        }

        // Refund the previous highest bidder (if any)
        if let Some(highest_bidder) = auction.highest_bidder {
            self.env()
                .transfer_tokens(&highest_bidder, &auction.highest_bid);
        }

        // Update the auction with the new highest bid and bidder
        auction.highest_bid = amount;
        auction.highest_bidder = Some(bidder);
        self.auctions.set(&auction_id, auction);
    }

    /// Ends an auction and distributes the NFT and funds accordingly.
    pub fn end_auction(&mut self, auction_id: U256) {
        self.pausable.require_not_paused();
        let auction = self.auctions.get(&auction_id).unwrap();

        // Check if auction has ended
        if self.env().get_block_time() < auction.ends_at {
            self.env().revert(Error::AuctionStillInProgress);
        }

        // Transfer the NFT and funds
        if let Some(winner) = auction.highest_bidder {
            Cep78ContractRef::new(self.env(), auction.nft_contract).transfer(
                Maybe::Some(auction.nft_id),
                Maybe::None,
                self.env().self_address(),
                winner,
            );
            self.env()
                .transfer_tokens(&auction.seller, &auction.highest_bid);
        } else {
            // No bids were placed, return the NFT to the seller
            Cep78ContractRef::new(self.env(), auction.nft_contract).transfer(
                Maybe::Some(auction.nft_id),
                Maybe::None,
                self.env().self_address(),
                auction.seller,
            );
        }
    }

    // ... (admin functions: pause and unpause)
}
```

### Deploying and Interacting with the Contract

1.  **Compilation:** Compile your contract:

    ```bash
    cargo odra build
    ```

2.  **Deployment:** Deploy the compiled Wasm file to the Casper Network using the `casper-client` tool or `odra-livenet`.

3.  **Interaction:**
    *   Use the contract's entry points (`create_auction`, `bid`, `end_auction`) to interact with it.
    *   Remember to approve the contract to transfer your NFT before creating an auction.

### Conclusion

Congratulations! You've successfully built a simple NFT auction contract on the Casper Network using the ODRA framework. This contract provides a foundation for creating more sophisticated auction mechanisms and exploring advanced NFT features.

**Summary:**

*   **CEP-78 Integration:** We utilized the `Cep78ContractRef` to seamlessly interact with CEP-78 NFT contracts.
*   **Auction Logic:** We implemented core auction functionalities like creating auctions, placing bids, and resolving auctions.
*   **Error Handling:** We integrated error handling mechanisms using `odra::odra_error`.

**Additional Resources:**

* **Odra Documentation - Cross-Contract Calls:** [https://odra.dev/docs/next/basics/cross-calls](https://odra.dev/docs/next/basics/cross-calls)
* **CEP-78 Standard:** [in-depth-guide](https://support.casperlabs.io/hc/en-gb/articles/14102164670363--In-depth-Guide-CEP-78-Understanding-Installing-and-Minting-your-own-Enhanced-NFTs)

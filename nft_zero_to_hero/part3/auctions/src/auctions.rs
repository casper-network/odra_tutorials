use odra::{
    args::Maybe,
    casper_types::{U256, U512},
    module::Module,
    Address, ContractRef, Mapping, SubModule, Var,
};
use odra_modules::cep78::token::Cep78ContractRef;
use odra_modules::{access::Ownable, security::Pauseable};

#[odra::module]
/// This contract facilitates NFT auctions, allowing users to create and participate in auctions for CEP-78 NFTs.
pub struct Auctions {
    /// Ownable submodule for managing contract ownership and permissions.
    ownable: SubModule<Ownable>,
    /// Pauseable submodule for pausing/unpausing contract functionality.
    pausable: SubModule<Pauseable>,
    /// Storage for active auctions, indexed by a unique auction ID.
    auctions: Mapping<U256, Auction>,
    /// Counter to track the total number of auctions created.
    auction_counter: Var<U256>,
    /// Minimum allowed duration for an auction, set by the contract owner.
    min_auction_duration: Var<u64>,
}

#[odra::odra_error]
/// Errors that may occur during the contract execution.
pub enum Error {
    /// Invalid auction duration, shorter than the minimum allowed.
    InvalidAuctionDuration = 1,
    /// Invalid bid amount, lower than the starting price or the current highest bid.
    InvalidBid = 2,
    /// Attempted to end an auction that has already ended.
    AuctionHasEnded = 3,
    /// Attempted to end an auction that is still in progress.
    AuctionStillInProgress = 4,
}

#[odra::odra_type]
/// Represents an active auction for an NFT.
pub struct Auction {
    /// Address of the seller who initiated the auction.
    seller: Address,
    /// Address of the CEP-78 NFT contract.
    nft_contract: Address,
    /// ID of the NFT being auctioned.
    nft_id: u64,
    /// Starting price of the auction in CSPR.
    starting_price: U512,
    /// Timestamp of when the auction ends.
    ends_at: u64,
    /// Optional address of the highest bidder (None if no bids yet).
    highest_bidder: Option<Address>,
    /// Amount of the highest bid in CSPR.
    highest_bid: U512,
}

#[odra::module]
impl Auctions {
    /// Initializes the contract, setting the owner (optional) and minimum auction duration.
    pub fn init(&mut self, admin: Option<Address>, min_auction_duration: u64) {
        self.ownable.init();
        if let Some(a) = admin {
            self.ownable.transfer_ownership(&a); // Transfer ownership to the provided admin
        }
        self.auction_counter.set(U256::one()); // Start auction counter from 1
        self.min_auction_duration.set(min_auction_duration);
    }

    /**********
     * TRANSACTIONS
     **********/

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

    /**********
     * ADMIN
     **********/

    /// Pauses the contract, preventing further interactions.
    pub fn pause(&mut self) {
        self.ownable.assert_owner(&self.env().caller());
        self.pausable.pause();
    }

    /// Unpauses the contract, resuming normal operation.
    pub fn unpause(&mut self) {
        self.ownable.assert_owner(&self.env().caller());
        self.pausable.unpause();
    }
}

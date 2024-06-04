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

#[odra::module]
pub struct ExtendedCEP78 {
    cep78: SubModule<Cep78>,
}

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
}

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

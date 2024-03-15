// Copyright (c) 2024 Algorealm, Inc.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod sam_os {
    use ink::prelude::vec::Vec;

    use ink::storage::Mapping;

    /// The struct describing an account on the network
    #[derive(scale::Decode, scale::Encode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct AccountInfo {
        /// Type of account (user | app)
        r#type: AccountType,
        /// The address of the DID document decribing the account
        did_doc_ipfs_addr: IpfsAddress,
    }

    /// SamaritanOS error type.
    #[derive(scale::Decode, scale::Encode, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Error {
        /// Returned when an account exists in storage already
        AccountExistsAlready,
        /// Returned when an account does not exist in storage
        AccountUnknown,
    }

    //// Event to announce the creation of an account
    #[ink(event)]
    pub struct AccountCreated {
        #[ink(topic)]
        account_id: AccountId,
    }

    //// Event to announce an account removal
    #[ink(event)]
    pub struct AccountRemoved {
        #[ink(topic)]
        account_id: AccountId,
    }

    /// The contracts result type.
    pub type Result<T> = core::result::Result<T, Error>;
    /// The type of an account (user | app)
    type AccountType = Vec<u8>;
    /// This type represents an IPFS address (CID)
    type IpfsAddress = Vec<u8>;

    /// The SamaritanOS contract storage
    #[ink(storage)]
    pub struct SamOs {
        accounts: Mapping<AccountId, AccountInfo>,
    }

    impl SamOs {
        /// Constructor that initializes the SamaritanOS contract storage
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                accounts: Default::default(),
            }
        }

        /// Create a new account on the network
        #[ink(message, payable)]
        pub fn new_account(&mut self, r#type: bool, did_doc_ipfs_addr: IpfsAddress) -> Result<()> {
            // Get the contract caller
            let caller = Self::env().caller();

            if !self.accounts.contains(&caller) {
                // Create new account
                self.accounts.insert(
                    &caller,
                    &AccountInfo {
                        r#type: if r#type {
                            "user".as_bytes().to_vec()
                        } else {
                            "application".as_bytes().to_vec()
                        },
                        did_doc_ipfs_addr,
                    },
                );

                // Emit event
                self.env().emit_event(AccountCreated { account_id: caller });
            } else {
                // Throw error: Account exists already!
                return Err(Error::AccountExistsAlready);
            }
            Ok(())
        }

        /// Delete a Samaritan from the network
        #[ink(message, payable)]
        pub fn delete_account(&mut self) -> Result<()> {
            // Get the contract caller
            let caller = Self::env().caller();

            if self.accounts.contains(&caller) {
                // remove from storage if it exists
                self.accounts.remove(&caller);

                // Emit event
                self.env().emit_event(AccountRemoved { account_id: caller });
            } else {
                // account doesn't exist
                return Err(Error::AccountUnknown);
            }

            Ok(())
        }

        /// Authenticate an account and return its DID document CID
        #[ink(message, payable)]
        pub fn auth_account(&mut self) -> Vec<u8> {
            // Get the contract caller
            let caller = Self::env().caller();

            if let Some(account_info) = self.accounts.get(&caller) {
                let mut return_bytes = account_info.did_doc_ipfs_addr;
                return_bytes.push(b'#');
                return_bytes.extend(account_info.r#type);
                return return_bytes;
            }

            Default::default()
        }

        /// Verify if a DID exists in contract storage
        #[ink(message, payable)]
        pub fn did_exists(&mut self, bytes: Vec<u8>) -> Vec<u8> {
            // try to convert a Vec<u8> into an AccountId
            if let Ok(address) = AccountId::try_from(&bytes[..]) {
                // try to read from strorage
                if let Some(account_info) = self.accounts.get(&address) {
                    let mut return_bytes = account_info.did_doc_ipfs_addr;
                    return_bytes.push(b'#');
                    return_bytes.extend(account_info.r#type);
                    return return_bytes;
                }
            }

            Default::default()
        }
    }
}

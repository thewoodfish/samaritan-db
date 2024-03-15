/// Copyright (c) Algorealm, Inc.
use crate::{cli, prelude::*};

/// Query the contract and authenticate the auth payload
pub async fn authenticate(cfg: &DbConfig, auth_payload: &AuthPayload) -> bool {
    let _ipfs_cid = cli::auth_account(&cfg, &auth_payload.secret).await;
    if _ipfs_cid.is_empty() {
        false
    } else {
        true
    }
}

/// Check the contract if a particular DID is registered.
pub async fn did_exists(cfg: &DbConfig, did: &Did) -> bool {
    // we need the SS58 suffix of the DID only
    if let Some(ss58_address) = did.0.split(":").last() {
        let _ipfs_cid = cli::did_exists(&cfg, ss58_address).await;
        if !_ipfs_cid.is_empty() {
            return true;
        }
    }
    false
}

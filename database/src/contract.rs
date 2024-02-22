/// Copyright (c) Algorealm, Inc.

use crate::prelude::*;

/// query the contract and authenticate the auth payload
pub fn authenticate(_auth_payload: &AuthPayload) -> bool {
    // TODO: write code to authenticate the contract

    true
}

/// check the contract if a particular DID is registered
pub fn did_exists(_did: &Did) -> bool {
    // TODO: write code that checks contract for DID validity

    true
}

/// Copyright (c) Algorealm, Inc.

use crate::{rpc, prelude::*};
use rocket::serde::json::Value;

/// Query the contract and authenticate the account 
pub async fn authenticate(auth_payload: &AuthPayload) -> bool {
    if let Ok(response) = rpc::auth_account(&auth_payload.secret).await {
        if response["error"] == Value::Bool(false) {
            return if response["data"]["exists"] == Value::Bool(true) { true } else { false }
        }
    }
    false
}

/// Check the contract if a particular DID is registered
pub async fn did_exists(cfg: &DbConfig, did: &Did) -> bool {
    // we're sending the ;ast
    if let Ok(response) = rpc::did_exists(&did.0.split(":").last().unwrap_or_default(), &cfg.mnemonic).await {
        if response["error"] == Value::Bool(false) {
            return if response["data"]["exists"] == Value::Bool(true) { true } else { false }
        }
    }
    false
}
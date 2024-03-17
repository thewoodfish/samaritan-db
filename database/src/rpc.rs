/// Copyright (c) Algorealm, Inc.
use crate::prelude::*;
use rocket::serde::json::Value;

/// Send an RPC to the contract to verify the validity of an account
pub async fn auth_account(mnemonic: &str) -> Result<Value, GenericError> {
    let url = format!("http://localhost:5000/authenticate?mnemonic={}", mnemonic);
    let response = reqwest::get(&url).await?.json::<Value>().await?;

    Ok(response)
}

/// Send an RPC to check if a DID exists on the network
pub async fn did_exists(did: &str, mnemonic: &str) -> Result<Value, GenericError> {
    let url = format!(
        "http://localhost:5000/didExists?address={}&mnemonic={}",
        did, mnemonic
    );
    let response = reqwest::get(&url).await?.json::<Value>().await?;

    Ok(response)
}

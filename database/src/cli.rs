/// Copyright (c) Algorealm, Inc.
use crate::{prelude::*, util};
use std::process::Command;

/// Authenticate an account and return its DID Document IPFS address if it exists
pub async fn auth_account(cfg: &DbConfig, mnemonic: &str) -> String {
    let mut count = 0;
    loop {
        match Command::new("cargo")
            .args([
                "contract",
                "call",
                "--contract",
                &cfg.contract_address,
                "--message",
                "auth_account",
                "--suri",
                mnemonic,
                "--url",
                &cfg.chain_address,
                "--dry-run",
            ])
            .current_dir(CONTRACT_DIRECTORY) // directory that contains the contract code
            .output()
        {
            Ok(output) => {
                let binding: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stdout);
                return util::parse_contract_return_data(&binding);
            }
            Err(_) => {
                if count == MAX_RETRY_COUNT {
                    break;
                } else {
                    count += 1
                }

                println!(
                    "contract invocation attempt returned an error: fn -> `auth_address()`. Trying again...",
                );
                // sleep for 5 seconds
                async_std::task::sleep(CLI_RETRY_DURATION).await;
            }
        }
    }

    Default::default()
}

/// Check if the DID is recognized onchain
pub async fn did_exists(cfg: &DbConfig, did: &str) -> String {
    let mut count = 0;
    loop {
        match Command::new("cargo")
            .args([
                "contract",
                "call",
                "--contract",
                &cfg.contract_address,
                "--message",
                "did_exists",
                "--args",
                &util::str_to_hex(did),
                "--suri",
                &cfg.mnemonic,
                "--url",
                &cfg.chain_address,
                "--dry-run",
            ])
            .current_dir(CONTRACT_DIRECTORY) // directory that contains the contract code
            .output()
        {
            Ok(output) => {
                let binding: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stdout);
                return util::parse_contract_return_data(&binding);
            }
            Err(_) => {
                if count == MAX_RETRY_COUNT {
                    break;
                } else {
                    count += 1
                }

                println!(
                    "contract invocation attempt returned an error: fn -> `did_exists()`. Trying again...",
                );
                // sleep for 5 seconds
                async_std::task::sleep(CLI_RETRY_DURATION).await;
            }
        }
    }

    Default::default()
}

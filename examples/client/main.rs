//! Wallet (client) example for the openlv library.
//!
//! Connects to a dApp session from an `openlv://` URL.
//!
//! Usage:
//!   cargo run --example client -- "openlv://..."

use std::{env, process};

use alloy_primitives::hex;
use alloy_signer::SignerSync;
use alloy_signer_local::{MnemonicBuilder, PrivateKeySigner, coins_bip39::English};
use openlv::prelude::*;
use serde_json::Value;

const ANVIL_MNEMONIC: &str = "test test test test test test test test test test test junk";

fn test_signer() -> Result<PrivateKeySigner, Box<dyn std::error::Error>> {
    Ok(MnemonicBuilder::<English>::default()
        .phrase(ANVIL_MNEMONIC)
        .index(0)?
        .build()?)
}

fn handle_rpc_request(request: Value, signer: &PrivateKeySigner) -> Value {
    match request.get("method").and_then(Value::as_str) {
        Some("eth_accounts") => json!([signer.address().to_string()]),
        Some("eth_chainId") => json!("0x1"),
        Some("personal_sign") => personal_sign(request, signer),
        Some(method) => json!({
            "error": {
                "code": -32601,
                "message": format!("Method {method} not found"),
            }
        }),
        None => json!({
            "error": {
                "code": -32600,
                "message": "Invalid request",
            }
        }),
    }
}

fn personal_sign(request: Value, signer: &PrivateKeySigner) -> Value {
    let Some(params) = request.get("params").and_then(Value::as_array) else {
        return invalid_params("personal_sign requires message and account parameters");
    };
    let [message, account] = params.as_slice() else {
        return invalid_params("personal_sign requires exactly two parameters");
    };
    let (Some(message), Some(account)) = (message.as_str(), account.as_str()) else {
        return invalid_params("personal_sign parameters must be strings");
    };
    if account != signer.address().to_string() {
        return json!({
            "error": {
                "code": 4100,
                "message": "Requested account is not available",
            }
        });
    }

    let message = match message.strip_prefix("0x").map(hex::decode).transpose() {
        Ok(Some(message)) => message,
        _ => return invalid_params("personal_sign message must be 0x-prefixed hex"),
    };
    match signer.sign_message_sync(&message) {
        Ok(signature) => json!(signature.to_string()),
        Err(error) => json!({
            "error": {
                "code": -32603,
                "message": error.to_string(),
            }
        }),
    }
}

fn invalid_params(message: &str) -> Value {
    json!({
        "error": {
            "code": -32602,
            "message": message,
        }
    })
}

#[tokio::main]
async fn main() {
    let url = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: client <openlv://...>");
        process::exit(1);
    });

    println!("Connecting to: {url}");
    let signer = test_signer().unwrap_or_else(|error| {
        eprintln!("Failed to derive test account: {error}");
        process::exit(1);
    });
    println!("Wallet account: {}", signer.address());

    let wallet = openlv::wallet(&url)
        .on_request(move |msg| {
            let signer = signer.clone();
            async move {
                println!("Received EIP-1193 request: {msg}");
                Ok(handle_rpc_request(msg, &signer))
            }
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to create session: {e}");
            process::exit(1);
        });

    wallet.connect().await.unwrap_or_else(|e| {
        eprintln!("Connection failed: {e}");
        process::exit(1);
    });

    wallet.wait_for_link().await.unwrap_or_else(|e| {
        eprintln!("Link failed: {e}");
        process::exit(1);
    });

    println!("Connected!");
    println!("Listening for EIP-1193 requests. Press Ctrl+C to exit.");

    std::future::pending::<()>().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responds_to_standard_wallet_requests() {
        let signer = test_signer().unwrap();
        let address = signer.address().to_string();
        assert_eq!(address, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        assert_eq!(
            handle_rpc_request(json!({"method": "eth_accounts", "params": []}), &signer),
            json!([address]),
        );
        assert_eq!(
            handle_rpc_request(json!({"method": "eth_chainId", "params": []}), &signer),
            json!("0x1"),
        );
    }

    #[test]
    fn signs_personal_messages_for_the_configured_account() {
        let signer = test_signer().unwrap();
        let response = handle_rpc_request(
            json!({
                "method": "personal_sign",
                "params": [
                    "0x48656c6c6f2c20776f726c6421",
                    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
                ]
            }),
            &signer,
        );
        assert!(
            response
                .as_str()
                .is_some_and(|signature| signature.starts_with("0x"))
        );
        assert_eq!(response.as_str().unwrap().len(), 132);
    }
}

# openlv (Rust)

Native Rust implementation of the [Open Lavatory](https://openlv.sh) protocol for interoperability with the [typescript implementation](https://github.com/v3xlabs/open-lavatory)

## Parity status

| Layer | Status |
|-------|--------|
| URI encode/decode + validation | Done |
| Handshake crypto (AES-128-GCM) | Done |
| Peer crypto (X25519 + XSalsa20-Poly1305) | Done |
| Wire frames (`h`/`x` + `h`/`c`) | Done |
| Signaling state machine | Done |
| NTFY signaling channel | Done |
| MQTT signaling channel | Done |
| WebRTC transport (`openlv-data`) | Done |
| Session API (`create_session` / `connect_session`) | Done |

## dApp

```rust
use openlv::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), openlv::OpenLvError> {
    let provider = dapp()
        .protocol(Protocol::Ntfy)
        .server("https://ntfy.sh/")
        .provider()
        .await?;

    provider.connect().await?;
    println!("{}", provider.uri());

    provider.wait_for_link().await?;
    let chain_id = provider.request("eth_chainId", json!([])).await?;
    println!("Connected to chain {chain_id}");
    Ok(())
}
```

## Wallet

```rust
use openlv::prelude::*;
use serde_json::{Value, json};

async fn connect_wallet(connection_url: &str) -> Result<(), OpenLvError> {
    let wallet = wallet(connection_url)
        .on_request(|request: Value| async move {
            match request.get("method").and_then(Value::as_str) {
                Some("eth_chainId") => Ok(json!("0x1")),
                Some("eth_accounts") => Ok(json!(["0x0000000000000000000000000000000000000000"])),
                Some(method) => Ok(json!({
                    "error": {
                        "code": -32601,
                        "message": format!("Method {method} not found"),
                    }
                })),
                None => Ok(json!({
                    "error": {
                        "code": -32600,
                        "message": "Invalid request",
                    }
                })),
            }
        })
        .await?;

    wallet.connect().await?;
    wallet.wait_for_link().await?;
    Ok(())
}
```

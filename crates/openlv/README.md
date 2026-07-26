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

## Usage

```rust
use openlv::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dapp = openlv::dapp()
        .protocol(Protocol::Ntfy)
        .server("https://ntfy.sh/")
        .on_request(|msg| async move {
            println!("received: {msg}");
            Ok(json!({"result": "ok"}))
        })
        .await?;

    dapp.connect().await?;
    println!("Connection URL: {}", dapp.uri());
    dapp.wait_for_link().await?;

    dapp.close().await?;
    Ok(())
}
```

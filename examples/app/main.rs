//! dApp (host) example for the openlv library.
//!
//! Creates a session, waits for a wallet to connect, sends an EIP-1193
//! request, and prints the response.
//!
//! Usage:
//!   cargo run --example dapp

use openlv::prelude::*;
use qrcode::QrCode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = openlv::dapp()
        .protocol(Protocol::Ntfy)
        .server("https://ntfy.sh/")
        .provider()
        .await?;

    provider.connect().await?;

    let uri = provider.uri().to_string();

    println!("Connection URL: {}", uri);

    let qr = QrCode::new(uri)?;
    let qr_data = qr
        .render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    println!("QR Code:\n{}", qr_data);

    println!("Waiting for wallet to connect...");
    provider.wait_for_link().await?;
    println!("Connected!");

    let chain_id = provider.request("eth_chainId", json!([])).await?;
    println!("Chain ID: {chain_id}");

    provider.close().await?;
    Ok(())
}

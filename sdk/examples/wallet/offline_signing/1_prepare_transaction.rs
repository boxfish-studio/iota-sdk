// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will get inputs and prepare a transaction.
//!
//! Rename `.env.example` to `.env` first, then run the command:
//! ```sh
//! cargo run --release --all-features --example 1_prepare_transaction
//! ```

use iota_sdk::{
    client::{api::PreparedTransactionDataDto, constants::SHIMMER_COIN_TYPE, secret::SecretManager},
    crypto::keys::bip44::Bip44,
    types::block::address::Bech32Address,
    wallet::{ClientOptions, SendParams, Wallet},
};

const ONLINE_WALLET_DB_PATH: &str = "./examples/wallet/offline_signing/example-online-walletdb";
const ADDRESS_FILE_PATH: &str = "./examples/wallet/offline_signing/example.address.json";
const PREPARED_TRANSACTION_FILE_PATH: &str = "./examples/wallet/offline_signing/example.prepared_transaction.json";
const PROTOCOL_PARAMETERS_FILE_PATH: &str = "./examples/wallet/offline_signing/example.protocol_parameters.json";
// Address to which we want to send the amount.
const RECV_ADDRESS: &str = "rms1qpszqzadsym6wpppd6z037dvlejmjuke7s24hm95s9fg9vpua7vluaw60xu";
// The amount to send.
const SEND_AMOUNT: u64 = 1_000_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    for var in ["NODE_URL"] {
        std::env::var(var).expect(&format!(".env variable '{var}' is undefined, see .env.example"));
    }

    let params = [SendParams::new(SEND_AMOUNT, RECV_ADDRESS)?];

    // Recovers addresses from example `0_address_generation`.
    let address = read_address_from_file().await?;

    let client_options = ClientOptions::new().with_node(&std::env::var("NODE_URL").unwrap())?;

    // Create the wallet with the secret_manager and client options
    let wallet = Wallet::builder()
        .with_secret_manager(SecretManager::Placeholder)
        .with_storage_path(ONLINE_WALLET_DB_PATH)
        .with_client_options(client_options.clone())
        .with_address(address)
        .with_bip_path(Bip44::new(SHIMMER_COIN_TYPE))
        .finish()
        .await?;

    // Sync the wallet to get the outputs for the addresses
    wallet.sync(None).await?;

    let prepared_transaction = wallet.prepare_send(params.clone(), None).await?;

    println!("Prepared transaction sending {params:?}");

    write_data_to_file(
        PreparedTransactionDataDto::from(&prepared_transaction),
        PREPARED_TRANSACTION_FILE_PATH,
    )
    .await?;

    write_data_to_file(
        wallet.client().get_protocol_parameters().await?,
        PROTOCOL_PARAMETERS_FILE_PATH,
    )
    .await?;

    Ok(())
}

async fn read_address_from_file() -> Result<Bech32Address, Box<dyn std::error::Error>> {
    use tokio::io::AsyncReadExt;

    let mut file = tokio::io::BufReader::new(tokio::fs::File::open(ADDRESS_FILE_PATH).await?);
    let mut json = String::new();
    file.read_to_string(&mut json).await?;

    Ok(serde_json::from_str(&json)?)
}

async fn write_data_to_file(data: impl serde::Serialize, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io::AsyncWriteExt;

    let json = serde_json::to_string_pretty(&data)?;
    let mut file = tokio::io::BufWriter::new(tokio::fs::File::create(path).await?);
    println!("{path}:\n{json}");
    file.write_all(json.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

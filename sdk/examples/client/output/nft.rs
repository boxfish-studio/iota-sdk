// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will create an NFT output.
//!
//! Rename `.env.example` to `.env` first, then run the command:
//! ```sh
//! cargo run --release --example nft
//! ```

use iota_sdk::{
    client::{
        api::GetAddressesOptions, node_api::indexer::query_parameters::QueryParameter, request_funds_from_faucet,
        secret::SecretManager, Client, Result,
    },
    types::block::{
        address::{Bech32Address, NftAddress},
        output::{
            unlock_condition::AddressUnlockCondition, BasicOutputBuilder, NftId, NftOutputBuilder, Output, OutputId,
        },
        payload::Payload,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    // Configure your own mnemonic in the ".env" file. Since the output amount cannot be zero, the seed must contain
    // non-zero balance.
    dotenvy::dotenv().ok();

    for var in ["NODE_URL", "MNEMONIC", "FAUCET_URL", "EXPLORER_URL"] {
        std::env::var(var).unwrap_or_else(|_| panic!(".env variable '{var}' is undefined, see .env.example"));
    }

    let node_url = std::env::var("NODE_URL").unwrap();
    let explorer_url = std::env::var("EXPLORER_URL").unwrap();
    let faucet_url = std::env::var("FAUCET_URL").unwrap();

    // Create a node client.
    let client = Client::builder().with_node(&node_url)?.finish().await?;

    let secret_manager = SecretManager::try_from_mnemonic(std::env::var("MNEMONIC").unwrap())?;

    let token_supply = client.get_token_supply().await?;

    let address = secret_manager
        .generate_ed25519_addresses(GetAddressesOptions::from_client(&client).await?.with_range(0..1))
        .await?[0];

    println!(
        "Requesting funds (waiting 15s): {}",
        request_funds_from_faucet(&faucet_url, &address).await?,
    );
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    //////////////////////////////////
    // create new nft output
    //////////////////////////////////

    let outputs = [
        // address of the owner of the NFT
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())
            .add_unlock_condition(AddressUnlockCondition::new(address))
            // address of the minter of the NFT
            // .add_feature(IssuerFeature::new(address))
            .finish_output(token_supply)?,
    ];

    let block = client
        .build_block()
        .with_secret_manager(&secret_manager)
        .with_outputs(outputs)?
        .finish()
        .await?;

    println!("Block with new NFT output sent: {}/block/{}", explorer_url, block.id());

    let _ = client.retry_until_included(&block.id(), None, None).await?;

    //////////////////////////////////
    // move funds from an NFT address
    //////////////////////////////////

    let nft_output_id = get_nft_output_id(block.payload().unwrap())?;
    let nft_id = NftId::from(&nft_output_id);

    let nft_address = NftAddress::new(nft_id);
    let bech32_nft_address = Bech32Address::new(client.get_bech32_hrp().await?, nft_address);
    println!("bech32_nft_address {bech32_nft_address}");

    println!(
        "Requesting funds (waiting 15s): {}",
        request_funds_from_faucet(&faucet_url, &bech32_nft_address).await?,
    );
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let output_ids_response = client
        .basic_output_ids([QueryParameter::Address(bech32_nft_address)])
        .await?;
    let output_with_meta = client.get_output(&output_ids_response.items[0]).await?;

    let block = client
        .build_block()
        .with_secret_manager(&secret_manager)
        .with_input(nft_output_id.into())?
        .with_input(output_ids_response.items[0].into())?
        .with_outputs([
            NftOutputBuilder::new_with_amount(1_000_000 + output_with_meta.output().amount(), nft_id)
                .add_unlock_condition(AddressUnlockCondition::new(bech32_nft_address))
                .finish_output(token_supply)?,
        ])?
        .finish()
        .await?;

    println!(
        "Block with input (basic output) to NFT output sent: {}/block/{}",
        explorer_url,
        block.id()
    );

    let _ = client.retry_until_included(&block.id(), None, None).await?;

    //////////////////////////////////
    // burn NFT
    //////////////////////////////////

    let nft_output_id = get_nft_output_id(block.payload().unwrap())?;
    let output_with_meta = client.get_output(&nft_output_id).await?;
    let outputs = [BasicOutputBuilder::new_with_amount(output_with_meta.output().amount())
        .add_unlock_condition(AddressUnlockCondition::new(bech32_nft_address))
        .finish_output(token_supply)?];

    let block = client
        .build_block()
        .with_secret_manager(&secret_manager)
        .with_input(nft_output_id.into())?
        .with_outputs(outputs)?
        .finish()
        .await?;

    println!(
        "Block with burn transaction sent: {}/block/{}",
        explorer_url,
        block.id()
    );

    let _ = client.retry_until_included(&block.id(), None, None).await?;

    Ok(())
}

// helper function to get the output id for the first NFT output
fn get_nft_output_id(payload: &Payload) -> Result<OutputId> {
    match payload {
        Payload::Transaction(tx_payload) => {
            for (index, output) in tx_payload.essence().as_regular().outputs().iter().enumerate() {
                if let Output::Nft(_nft_output) = output {
                    return Ok(OutputId::new(tx_payload.id(), index.try_into().unwrap())?);
                }
            }
            panic!("No nft output in transaction essence")
        }
        _ => panic!("No tx payload"),
    }
}

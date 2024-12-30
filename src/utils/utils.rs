// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::Read;
use anyhow::Context;
use std::path::PathBuf;
use std::process::Command;

use identity_iota::iota::block::output::AliasOutput;
use identity_iota::iota::IotaClientExt;
use identity_iota::iota::IotaDocument;
use identity_iota::iota::IotaIdentityClientExt;
use identity_iota::iota::NetworkName;
use identity_iota::storage::JwkDocumentExt;
use identity_iota::storage::JwkMemStore;
use identity_iota::storage::KeyIdMemstore;
use identity_iota::storage::Storage;
use identity_iota::verification::MethodScope;

use identity_iota::verification::jws::JwsAlgorithm;
use iota_sdk::client::api::GetAddressesOptions;
use iota_sdk::client::node_api::indexer::query_parameters::QueryParameter;
use iota_sdk::client::secret::SecretManager;
use iota_sdk::client::Client;
use iota_sdk::crypto::keys::bip39;
use iota_sdk::types::block::address::Address;
use iota_sdk::types::block::address::Bech32Address;
use iota_sdk::types::block::address::Hrp;
use rand::distributions::DistString;
use serde_json::Value;

use super::config;
use super::config::Config;

pub type MemStorage = Storage<JwkMemStore, KeyIdMemstore>;

/// Creates a DID Document and publishes it in a new Alias Output.
///
/// Its functionality is equivalent to the "create DID" example
/// and exists for convenient calling from the other examples.
pub async fn create_did(
    client: &Client,
    secret_manager: &mut SecretManager,
    storage: &MemStorage,
    faucet_endpoint: &str,
) -> anyhow::Result<(Address, IotaDocument, String)> {
    let address: Address = get_address_with_funds(client, secret_manager, faucet_endpoint)
        .await
        .context("failed to get address with funds")?;

    let network_name: NetworkName = client.network_name().await?;

    let (document, fragment): (IotaDocument, String) =
        create_did_document(&network_name, storage).await?;

    let alias_output: AliasOutput = client.new_did_output(address, document, None).await?;

    let document: IotaDocument = client
        .publish_did_output(secret_manager, alias_output)
        .await?;

    Ok((address, document, fragment))
}

/// Creates an example DID document with the given `network_name`.
///
/// Its functionality is equivalent to the "create DID" example
/// and exists for convenient calling from the other examples.
pub async fn create_did_document(
    network_name: &NetworkName,
    storage: &MemStorage,
) -> anyhow::Result<(IotaDocument, String)> {
    let mut document: IotaDocument = IotaDocument::new(network_name);

    let fragment: String = document
        .generate_method(
            storage,
            JwkMemStore::ED25519_KEY_TYPE,
            JwsAlgorithm::EdDSA,
            None,
            MethodScope::VerificationMethod,
        )
        .await?;

    Ok((document, fragment))
}

/// Generates an address from the given [`SecretManager`] and adds funds from the faucet.
pub async fn get_address_with_funds(
    client: &Client,
    stronghold: &SecretManager,
    faucet_endpoint: &str,
) -> anyhow::Result<Address> {
    let address: Bech32Address = get_address(client, stronghold).await?;

    request_faucet_funds(client, address, faucet_endpoint)
        .await
        .context("failed to request faucet funds")?;

    Ok(*address)
}

/// Initializes the [`SecretManager`] with a new mnemonic, if necessary,
/// and generates an address from the given [`SecretManager`].
pub async fn get_address(
    client: &Client,
    secret_manager: &SecretManager,
) -> anyhow::Result<Bech32Address> {
    let random: [u8; 32] = rand::random();
    let mnemonic = bip39::wordlist::encode(random.as_ref(), &bip39::wordlist::ENGLISH)
        .map_err(|err| anyhow::anyhow!(format!("{err:?}")))?;

    if let SecretManager::Stronghold(ref stronghold) = secret_manager {
        match stronghold.store_mnemonic(mnemonic).await {
            Ok(()) => (),
            Err(iota_sdk::client::stronghold::Error::MnemonicAlreadyStored) => (),
            Err(err) => anyhow::bail!(err),
        }
    } else {
        anyhow::bail!("expected a `StrongholdSecretManager`");
    }

    let bech32_hrp: Hrp = client.get_bech32_hrp().await?;
    let address: Bech32Address = secret_manager
        .generate_ed25519_addresses(
            GetAddressesOptions::default()
                .with_range(0..1)
                .with_bech32_hrp(bech32_hrp),
        )
        .await?[0];

    Ok(address)
}

/// Requests funds from the faucet for the given `address`.
pub async fn request_faucet_funds(
    client: &Client,
    address: Bech32Address,
    faucet_endpoint: &str,
) -> anyhow::Result<()> {
    iota_sdk::client::request_funds_from_faucet(faucet_endpoint, &address).await?;

    tokio::time::timeout(std::time::Duration::from_secs(45), async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let balance = get_address_balance(client, &address)
                .await
                .context("failed to get address balance")?;
            if balance > 0 {
                break;
            }
        }
        Ok::<(), anyhow::Error>(())
    })
    .await
    .context("maximum timeout exceeded")??;

    Ok(())
}

/// Returns the balance of the given Bech32-encoded `address`.
async fn get_address_balance(client: &Client, address: &Bech32Address) -> anyhow::Result<u64> {
    let output_ids = client
        .basic_output_ids(vec![
            QueryParameter::Address(address.to_owned()),
            QueryParameter::HasExpiration(false),
            QueryParameter::HasTimelock(false),
            QueryParameter::HasStorageDepositReturn(false),
        ])
        .await?;

    let outputs = client.get_outputs(&output_ids).await?;

    let mut total_amount = 0;
    for output_response in outputs {
        total_amount += output_response.output().amount();
    }

    Ok(total_amount)
}

/// Creates a random stronghold path in the temporary directory, whose exact location is OS-dependent.
pub fn random_stronghold_path(config: &config::VariablesConfig) -> PathBuf {
    let mut file = std::path::PathBuf::from(config.get_value("stronghold_path"));
    file.push("test_strongholds");
    file.push(rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 32));
    file.set_extension("stronghold");
    file.to_owned()
}

pub fn random_credential_path() -> PathBuf {
    let mut file = std::env::temp_dir();
    file.push("credentials");
    create_directory_is_not_exists(&file).unwrap();
    file.push(rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 32));
    file.set_extension("json");
    file.to_owned()
}

/// Builds a JSON credential by reading a file, parsing its content, and adding the holder's DID.
///
/// # Arguments
///
/// * `holder_did` - An `IotaDocument` representing the holder's DID.
/// * `path` - A reference to a `String` containing the path to the JSON file.
///
/// # Returns
///
/// * `anyhow::Result<Value>` - A result containing the modified JSON value or an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The file cannot be opened or read.
/// * The file content is not a valid JSON object.
/// * The JSON parsing fails.
pub fn build_json_credential(holder_did: IotaDocument, path: &String) -> anyhow::Result<Value> {
    // Read file content
    let mut context = String::new();
    File::open(&path)?.read_to_string(&mut context)?;

    // Parse to JSON
    let mut json: Value = serde_json::from_str(&context)?;

    // Ensure it's a JSON object
    if let Value::Object(ref mut map) = json {
        // Add the `id` key with the holder's ID
        map.insert("id".to_string(), Value::String(holder_did.id().to_string()));
    } else {
        anyhow::bail!("File content is not a valid JSON object");
    }

    Ok(json)
}

fn create_directory_is_not_exists(path: &PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn pretty_print_json(label: &str, value: &str) {
    let data: Value = serde_json::from_str(value).unwrap();
    let pretty_json = serde_json::to_string_pretty(&data).unwrap();
    println!("--------------------------------------");
    println!("{}:", label);
    println!("--------------------------------------");
    println!("{} \n", pretty_json);
}

pub fn extract_kid(resolved_document: &IotaDocument) -> Result<String, anyhow::Error> {
    let binding = resolved_document.methods(Some(MethodScope::VerificationMethod));

    let method = binding
        .first()
        .ok_or(anyhow::anyhow!("Methods not Found"))?;

    let public_key_jwk = method
        .data()
        .public_key_jwk()
        .ok_or(anyhow::anyhow!("No JWK provided"))?;

    let kid = public_key_jwk
        .kid()
        .ok_or(anyhow::anyhow!("Kid not founded"))?;

    Ok(kid.to_string())
}

pub fn is_command_available(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", command))
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

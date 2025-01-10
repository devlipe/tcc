use anyhow::Context;
use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
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

use base64::engine::general_purpose;
use base64::Engine;

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



/// Insert the holder's DID into a JSON object.
/// This function is used to add the `id` field to a JSON object.
/// The `id` field is used to specify the holder's DID.
/// The function will return an error if the JSON object is not a map.
/// The function will return an error if the `id` field already exists in the JSON object.
/// The function will return an error if the holder's DID is not a string.
/// The function will return the modified JSON object with the `id` field.
pub fn insert_holder_did(json: &mut Value, holder_did: &str) -> anyhow::Result<Value> {
    if let Value::Object(map) = json {
        if map.contains_key("id") {
            anyhow::bail!("The JSON object already contains an 'id' field");
        }

        map.insert("id".to_string(), Value::String(holder_did.to_string()));
    } else {
        anyhow::bail!("The JSON object is not a map");
    }

    Ok(json.clone())
}

/// Recursively generates JSON paths from a given JSON object.
///
/// # Arguments
/// * `json` - The JSON object to traverse.
/// * `prefix` - The current path prefix (used for recursion).
///
/// # Returns
/// A vector of strings representing all the paths in the JSON object.
pub fn generate_json_paths(json: &Value, prefix: &str) -> Vec<String> {
    let mut paths = Vec::new();

    match json {
        Value::Object(map) => {
            for (key, value) in map {
                let new_prefix = if prefix.is_empty() {
                    format!("/{}", key)
                } else {
                    format!("{}/{}", prefix, key)
                };
                paths.extend(generate_json_paths(value, &new_prefix));
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                let new_prefix = format!("{}/{}", prefix, index);
                paths.extend(generate_json_paths(value, &new_prefix));
            }
        }
        _ => {
            paths.push(prefix.to_string());
        }
    }

    paths
}

/// Compares the structure of two JSON values by generating and comparing their paths.
///
/// # Arguments
///
/// * `json1` - A reference to the first JSON value.
/// * `json2` - A reference to the second JSON value.
///
/// # Returns
///
/// * `bool` - Returns `true` if both JSON values have the same structure, otherwise `false`.
pub fn have_same_structure(json1: &Value, json2: &Value) -> bool {
    let paths1: HashSet<String> = generate_json_paths(json1, "").into_iter().collect();
    let paths2: HashSet<String> = generate_json_paths(json2, "").into_iter().collect();

    paths1 == paths2
}

pub fn read_json_file(path: &str) -> anyhow::Result<Value> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;
    Ok(json)
}

pub fn remove_file_extension(file_name: &str) -> String {
    let file_name = file_name.split('.').collect::<Vec<&str>>()[0];
    file_name.to_string()
}

pub fn file_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

pub fn prepend_comment_to_file(file: &mut File) -> anyhow::Result<()> {
    let comment = r#"
    # This file was generated by the Petrus CLI.
    # You are free to modify it manually, but you are responsible for your actions
    # and the consequences of any changes.
    # This file support comments, so you can add your own notes.
    # Just use the '#' character at the beginning of the line.
    "#;

    file.write(comment.as_bytes())?;
    // add a new line
    writeln!(file)?;
    file.flush()?;
    Ok(())
}

pub fn write_vec_to_file(file: &mut File, data: &Vec<String>) -> anyhow::Result<()> {
    for line in data {
        writeln!(file, "{}", line)?;
    }
    file.flush()?; // Ensure the buffer is written to the file
    Ok(())
}

pub fn read_file_ignoring_comments(file_path: &str) -> anyhow::Result<Vec<String>> {
    // Open the file
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    // Process each line, ignoring comments and empty lines
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|line| match line {
            Ok(content) => {
                let trimmed = content.trim();
                if !trimmed.starts_with('#') && !trimmed.is_empty() {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect();

    Ok(lines)
}

pub(crate) fn edit_file(editor: String, path: &String) -> anyhow::Result<()> {
    if editor == "code" {
        let status = Command::new(editor)
            .arg("--wait")
            .arg(&path)
            .status()
            .expect("Failed to open editor");

        if !status.success() {
            eprintln!("Failed to open editor");
            return Err(anyhow::anyhow!("Failed to open editor"));
        }
    } else {
        let status = Command::new(editor)
            .arg(&path)
            .status()
            .expect("Failed to open editor");

        if !status.success() {
            eprintln!("Failed to open editor");
            return Err(anyhow::anyhow!("Failed to open editor"));
        }
    }
    Ok(())
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

pub fn add_base64_padding(encoded: &str) -> String {
    let padding_needed = (4 - (encoded.len() % 4)) % 4;
    let mut padded = encoded.to_string();
    padded.extend(std::iter::repeat('=').take(padding_needed));
    padded
}

/// Decodes the disclosures and extracts the second field (key).
///
/// # Arguments
/// - `disclosures`: A vector of base64url-encoded strings, each representing a disclosure.
///
/// # Returns
/// - A vector of strings containing the second field from each disclosure.
pub fn extract_disclosure_keys(disclosures: &Vec<String>) -> anyhow::Result<Vec<String>> {
    let mut extracted_fields = Vec::new();

    for disclosure in disclosures {
        let padded = add_base64_padding(disclosure);
        // Decode the base64url-encoded disclosure
        let decoded = general_purpose::URL_SAFE.decode(padded)?;
        // Parse the JSON object
        let json: Value = serde_json::from_slice(&decoded)?;

        // Ensure the JSON is an array and has at least two elements
        if let Some(array) = json.as_array() {
            if array.len() > 1 {
                if let Some(second_field) = array.get(1) {
                    if let Some(second_field_str) = second_field.as_str() {
                        extracted_fields.push(second_field_str.to_string());
                    }
                }
            }
        }
    }

    Ok(extracted_fields)
}

pub fn decode_base64(encoded: &str) -> anyhow::Result<String> {
    // Add padding if necessary
    let padded = add_base64_padding(encoded);

    // Decode the padded Base64 string
    let decoded = general_purpose::URL_SAFE.decode(padded)?;

    // Convert the decoded bytes to a String (assumes it's valid UTF-8)
    let decoded_string = String::from_utf8(decoded)?;

    Ok(decoded_string)
}

/// Extracts the issuer and subject from a JWT.
///
/// # Arguments
///
/// * `jwt` - A reference to a `String` containing the JWT.
///
/// # Returns
///
/// * `anyhow::Result<(String, String)>` - A result containing a tuple with the **issuer** and **subject** as strings, or an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The JWT does not have exactly three parts.
/// * The payload cannot be decoded from Base64.
/// * The payload is not valid JSON.
/// * The issuer (`iss`) or subject (`sub`) claims are missing or not strings.
pub fn get_entities_from_jwt(jwt: &String) -> anyhow::Result<(String, String)> {
    // Split the JWT into its three parts
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return Err(anyhow::anyhow!("Invalid JWT"));
    }
    let encoded_payload = parts[1]; // JWT payload is the second part

    // Decode the payload from Base64
    let decoded_payload = decode_base64(encoded_payload)?;
    // Parse the payload as JSON
    pretty_print_json("Payload", &decoded_payload);
    let payload_json: Value = serde_json::from_str(&decoded_payload)?;
    // Access specific claims
    let Some(issuer) = payload_json.get("iss").and_then(|v| v.as_str()) else {
        return Err(anyhow::anyhow!("Could not parse JWT, and get issuer"));
    };
    let Some(subject) = payload_json.get("sub").and_then(|v| v.as_str()) else {
        return Err(anyhow::anyhow!("Could not parse JWT, and get holder"));
    };

    // Return the issuer and subject as a tuple
    Ok((issuer.to_string(), subject.to_string()))
}
